//! Random number sources.
//!
//! Generators take their random numbers from [`Rng`], so that tests can feed scripted values
//! ([`ScriptedRng`]) and record the consumption order ([`RecordingRng`]) to compare it with the
//! TypeScript implementation.

use std::cell::RefCell;

use serde::{Deserialize, Deserializer, Serialize};

/// A source of uniform random numbers in `[0, 1)`.
pub trait Rng {
    fn random(&mut self) -> f64;
}

impl<R: Rng + ?Sized> Rng for &mut R {
    fn random(&mut self) -> f64 {
        (**self).random()
    }
}

impl<R: Rng + ?Sized> Rng for Box<R> {
    fn random(&mut self) -> f64 {
        (**self).random()
    }
}

/// Seed of a [`ReliableRandom`].
///
/// The parts are read from JSON like JavaScript reads them: the number is taken
/// as a `f64` (as `JSON.parse` does) and then converted like `BigInt(x) & U64_MASK`, i.e. to the
/// two's complement of the integer modulo 2^64. A number that is not an integer is an error
/// (`BigInt` throws).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RandomSeed {
    #[serde(deserialize_with = "deserialize_seed_part")]
    pub state: i64,
    #[serde(deserialize_with = "deserialize_seed_part")]
    pub sequence: i64,
}

impl RandomSeed {
    pub fn new(state: i64, sequence: i64) -> Self {
        Self { state, sequence }
    }

    /// A seed of its own for each `label`, so that consumers given the same seed draw
    /// independent sequences. Both parts are mixed with a hash of the label, so the generators
    /// differ in their state as well as in their PCG stream.
    pub fn for_label(self, label: &str) -> Self {
        let h = fnv1a(label.as_bytes());
        Self {
            state: splitmix64(self.state as u64 ^ h) as i64,
            sequence: splitmix64(self.sequence as u64 ^ splitmix64(h)) as i64,
        }
    }
}

/// 64-bit FNV-1a.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

/// The output function of SplitMix64: a bijective mix of the bits.
fn splitmix64(x: u64) -> u64 {
    let x = x.wrapping_add(0x9e3779b97f4a7c15);
    let x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Converts a JavaScript number to a seed part like `BigInt(x) & U64_MASK`
/// (reinterpreted as `i64`). `None` if `x` is not an integer.
pub fn seed_part_from_f64(x: f64) -> Option<i64> {
    const TWO_63: f64 = 9223372036854775808.0;
    const TWO_64: f64 = 18446744073709551616.0;
    if !x.is_finite() || x.fract() != 0.0 {
        return None;
    }
    // `%` is exact, and so are the subtractions below (Sterbenz lemma).
    let r = x % TWO_64;
    let r = if r >= TWO_63 {
        r - TWO_64
    } else if r < -TWO_63 {
        r + TWO_64
    } else {
        r
    };
    Some(r as i64)
}

fn deserialize_seed_part<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    let x = f64::deserialize(d)?;
    seed_part_from_f64(x)
        .ok_or_else(|| serde::de::Error::custom(format!("a seed must be an integer: {x}")))
}

const MULTIPLIER: u64 = 6364136223846793005;

/// The generator of the npm package reliable-random 0.0.1, reproduced bit for bit.
///
/// It is a PCG32 variant whose output function differs from the standard one
/// (so it does not match `rand_pcg::Pcg32`): the xorshifted value is not masked to 32 bits,
/// and the left rotation is `rot ^ 31` (= `31 - rot`) instead of `(-rot) & 31`.
/// Negative seeds are taken as two's complement, like `BigInt(x) & U64_MASK` in JavaScript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReliableRandom {
    state: u64,
    inc: u64,
}

impl ReliableRandom {
    pub fn new(state: i64, sequence: i64) -> Self {
        let mut r = Self {
            state: 0,
            inc: ((sequence as u64) << 1) | 1,
        };
        r.next_u32();
        r.state = r.state.wrapping_add(state as u64);
        r.next_u32();
        r
    }

    pub fn from_seed(seed: RandomSeed) -> Self {
        Self::new(seed.state, seed.sequence)
    }

    /// The raw 32-bit output (`_random_b` in reliable-random).
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
        // 37 bits wide; bits shifted beyond 64 are discarded by the final mask anyway.
        let xorshifted = ((old >> 18) ^ old) >> 27;
        let rot = (old >> 59) as u32;
        ((xorshifted >> rot) | (xorshifted << (rot ^ 31))) as u32
    }
}

impl Rng for ReliableRandom {
    fn random(&mut self) -> f64 {
        f64::from(self.next_u32()) / 4294967296.0
    }
}

/// Returns the given values in order. Panics when exhausted (it is meant for tests).
#[derive(Debug, Clone)]
pub struct ScriptedRng {
    values: Vec<f64>,
    index: usize,
}

impl ScriptedRng {
    pub fn new(values: impl Into<Vec<f64>>) -> Self {
        Self {
            values: values.into(),
            index: 0,
        }
    }

    /// Number of values returned so far.
    pub fn consumed(&self) -> usize {
        self.index
    }
}

impl Rng for ScriptedRng {
    fn random(&mut self) -> f64 {
        let Some(&v) = self.values.get(self.index) else {
            panic!("ScriptedRng exhausted after {} values", self.values.len());
        };
        self.index += 1;
        v
    }
}

/// One value drawn from a [`RecordingRng`].
#[derive(Debug, Clone, PartialEq)]
pub struct RngRecord {
    pub label: String,
    pub value: f64,
}

/// Delegates to `inner` and appends each value with `label` to `log`.
#[derive(Debug)]
pub struct RecordingRng<'a, R> {
    inner: R,
    label: String,
    log: &'a RefCell<Vec<RngRecord>>,
}

impl<'a, R: Rng> RecordingRng<'a, R> {
    pub fn new(inner: R, label: impl Into<String>, log: &'a RefCell<Vec<RngRecord>>) -> Self {
        Self {
            inner,
            label: label.into(),
            log,
        }
    }
}

impl<R: Rng> Rng for RecordingRng<'_, R> {
    fn random(&mut self) -> f64 {
        let value = self.inner.random();
        self.log.borrow_mut().push(RngRecord {
            label: self.label.clone(),
            value,
        });
        value
    }
}

/// Creates the random number sources of the generators (`RngFactory` in TypeScript).
///
/// `label` names the consumer (`"cadence"`, `"chord.root"`, `"chord.tone"`, `"rhythm"`,
/// `"note.pitch"`, `"variant"`). [`ReliableRandomFactory`] derives the seed from it, and
/// diagnostic factories such as [`RecordingRngFactory`] record it.
pub trait RngFactory {
    fn create(&self, seed: RandomSeed, label: &str) -> Box<dyn Rng + '_>;
}

/// Creates a [`ReliableRandom`] from each seed combined with the label
/// ([`RandomSeed::for_label`]). This is what the generators normally use.
///
/// The parameters give every layer the same seed by default; without the label, all the
/// layers would draw the same sequence and be correlated.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReliableRandomFactory;

impl RngFactory for ReliableRandomFactory {
    fn create(&self, seed: RandomSeed, label: &str) -> Box<dyn Rng + '_> {
        Box::new(ReliableRandom::from_seed(seed.for_label(label)))
    }
}

/// Wraps every Rng created by `inner` with a [`RecordingRng`] writing to `log`.
#[derive(Debug)]
pub struct RecordingRngFactory<'a, F> {
    inner: F,
    log: &'a RefCell<Vec<RngRecord>>,
}

impl<'a, F: RngFactory> RecordingRngFactory<'a, F> {
    pub fn new(inner: F, log: &'a RefCell<Vec<RngRecord>>) -> Self {
        Self { inner, log }
    }
}

impl<F: RngFactory> RngFactory for RecordingRngFactory<'_, F> {
    fn create(&self, seed: RandomSeed, label: &str) -> Box<dyn Rng + '_> {
        Box::new(RecordingRng::new(
            self.inner.create(seed, label),
            label,
            self.log,
        ))
    }
}

/// Ignores the seeds: every Rng it creates draws from the same `rng` (for tests).
#[derive(Debug)]
pub struct SharedRngFactory<R> {
    rng: RefCell<R>,
}

impl<R: Rng> SharedRngFactory<R> {
    pub fn new(rng: R) -> Self {
        Self {
            rng: RefCell::new(rng),
        }
    }

    pub fn into_inner(self) -> R {
        self.rng.into_inner()
    }
}

struct SharedRng<'a, R>(&'a RefCell<R>);

impl<R: Rng> Rng for SharedRng<'_, R> {
    fn random(&mut self) -> f64 {
        self.0.borrow_mut().random()
    }
}

impl<R: Rng> RngFactory for SharedRngFactory<R> {
    fn create(&self, _seed: RandomSeed, _label: &str) -> Box<dyn Rng + '_> {
        Box::new(SharedRng(&self.rng))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_match_their_references() {
        // FNV-1a test vectors, and the first output of SplitMix64 seeded with 0.
        assert_eq!(fnv1a(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(splitmix64(0), 0xe220a8397b1dcdaf);
    }

    #[test]
    fn the_factory_gives_each_label_its_own_sequence() {
        let seed = RandomSeed::new(4649, 459);
        let draw = |label| {
            let mut r = ReliableRandomFactory.create(seed, label);
            (0..4).map(|_| r.random()).collect::<Vec<_>>()
        };
        let labels = [
            "cadence",
            "chord.root",
            "chord.tone",
            "rhythm",
            "note.pitch",
            "variant",
        ];
        let seqs: Vec<_> = labels.iter().map(|l| draw(l)).collect();
        for (i, a) in seqs.iter().enumerate() {
            assert_eq!(*a, draw(labels[i]), "deterministic");
            for b in &seqs[i + 1..] {
                assert_ne!(a, b);
            }
        }
        let mut plain = ReliableRandom::from_seed(seed.for_label("rhythm"));
        assert_eq!(
            draw("rhythm"),
            (0..4).map(|_| plain.random()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn seed_part_like_bigint() {
        let two_53 = 9007199254740992.0;
        let cases = [
            (0.0, Some(0)),
            (-1.0, Some(-1)),
            (9007199254740991.0, Some(9007199254740991)),
            (-two_53, Some(-9007199254740992)),
            // 2^63 wraps to i64::MIN, 2^64 to 0, 2^64 + 2^12 to 2^12 and -(2^63 + 2^11) to 2^63 - 2^11.
            (9223372036854775808.0, Some(i64::MIN)),
            (18446744073709551616.0, Some(0)),
            (18446744073709555712.0, Some(4096)),
            (-9223372036854777856.0, Some(9223372036854773760)),
            (0.5, None),
            (f64::NAN, None),
            (f64::INFINITY, None),
        ];
        for (x, expected) in cases {
            assert_eq!(seed_part_from_f64(x), expected, "x = {x}");
        }
    }

    #[test]
    fn seed_from_json() {
        let seed: RandomSeed =
            serde_json::from_str(r#"{"state": 9007199254740991, "sequence": -7}"#).unwrap();
        assert_eq!(seed, RandomSeed::new(9007199254740991, -7));
        // JSON.parse rounds to f64 first: 2^64 - 1 becomes 2^64, i.e. 0.
        let seed: RandomSeed =
            serde_json::from_str(r#"{"state": 18446744073709551615, "sequence": 1e3}"#).unwrap();
        assert_eq!(seed, RandomSeed::new(0, 1000));
        assert!(serde_json::from_str::<RandomSeed>(r#"{"state": 1.5, "sequence": 0}"#).is_err());
    }

    #[test]
    fn factories() {
        let seed = RandomSeed::new(754, 489);
        let mut expected = ReliableRandom::from_seed(seed.for_label("a"));
        let log = RefCell::new(Vec::new());
        let factory = RecordingRngFactory::new(ReliableRandomFactory, &log);
        let mut r = factory.create(seed, "a");
        let v = r.random();
        assert_eq!(v, expected.random());
        drop(r);
        assert_eq!(
            *log.borrow(),
            vec![RngRecord {
                label: "a".into(),
                value: v
            }]
        );

        let shared = SharedRngFactory::new(ScriptedRng::new([0.1, 0.2]));
        assert_eq!(shared.create(seed, "a").random(), 0.1);
        assert_eq!(shared.create(seed, "b").random(), 0.2);
        assert_eq!(shared.into_inner().consumed(), 2);
    }

    #[test]
    fn scripted_returns_values_in_order() {
        let mut r = ScriptedRng::new([0.0, 0.5, 0.999]);
        assert_eq!([r.random(), r.random(), r.random()], [0.0, 0.5, 0.999]);
        assert_eq!(r.consumed(), 3);
    }

    #[test]
    #[should_panic(expected = "exhausted")]
    fn scripted_panics_when_exhausted() {
        let mut r = ScriptedRng::new([0.0]);
        r.random();
        r.random();
    }

    #[test]
    fn recording_passes_values_through() {
        let log = RefCell::new(Vec::new());
        let mut r = RecordingRng::new(ScriptedRng::new([0.25, 0.75]), "x", &log);
        assert_eq!([r.random(), r.random()], [0.25, 0.75]);
        let rec = |value| RngRecord {
            label: "x".into(),
            value,
        };
        assert_eq!(log.into_inner(), vec![rec(0.25), rec(0.75)]);
    }

    // Values from src/core/util.test.ts (compared with toBeCloseTo there).
    #[test]
    fn random_values_from_ts_tests() {
        let cases: [((i64, i64), [f64; 4]); 2] = [
            (
                (754, 489),
                [
                    0.7883008599746972,
                    0.30768447695299983,
                    0.00584357138723135,
                    0.6822985948529094,
                ],
            ),
            (
                (222, 9643),
                [
                    0.3066857950761914,
                    0.43649681424722075,
                    0.9942313127685338,
                    0.10204491158947349,
                ],
            ),
        ];
        for ((state, sequence), expected) in cases {
            let mut r = ReliableRandom::new(state, sequence);
            for e in expected {
                assert_eq!(r.random(), e, "seed ({state}, {sequence})");
            }
        }
    }
}
