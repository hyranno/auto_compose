//! Random number sources.
//!
//! The generators draw from a [`rand::Rng`]. Each layer of a tune takes its own generator from
//! an [`RngFactory`], by the seed in its parameters and a label of its own.

use std::convert::Infallible;

pub use rand::Rng;
use rand::{RngExt, TryRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Deserializer, Serialize};

/// `true` with the probability `p`, drawing one number. `p` is effectively clamped to `[0, 1]`.
pub fn chance(rng: &mut (impl Rng + ?Sized), p: f64) -> bool {
    rng.random::<f64>() < p
}

/// Seed of a random number generator: the state and the stream of a [`Pcg32`].
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

pub trait RngFactory {
    fn create(&self, seed: RandomSeed, label: &str) -> Box<dyn Rng + '_>;
}

/// Gives a [`Pcg32`] seeded by [`RandomSeed::for_label`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SeededRngFactory;

impl SeededRngFactory {
    pub fn rng(seed: RandomSeed, label: &str) -> Pcg32 {
        let seed = seed.for_label(label);
        Pcg32::new(seed.state as u64, seed.sequence as u64)
    }
}

impl RngFactory for SeededRngFactory {
    fn create(&self, seed: RandomSeed, label: &str) -> Box<dyn Rng + '_> {
        Box::new(Self::rng(seed, label))
    }
}

/// The 64-bit word that [`RngExt::random`] turns into `x` (rounded down to a multiple of
/// 2^-53) when `x` is in `[0, 1)`.
pub fn unit_to_u64(x: f64) -> u64 {
    (x * 18446744073709551616.0) as u64
}

/// Returns the given numbers in `[0, 1)` in order, for tests: [`RngExt::random`] gives them
/// back (rounded down to a multiple of 2^-53), and a draw that takes one word compares with them.
/// Panics when they run out.
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

    /// The number of words drawn.
    pub fn consumed(&self) -> usize {
        self.index
    }
}

impl TryRng for ScriptedRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok((self.try_next_u64()? >> 32) as u32)
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        let Some(&v) = self.values.get(self.index) else {
            panic!("ScriptedRng exhausted after {} values", self.values.len());
        };
        self.index += 1;
        Ok(unit_to_u64(v))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        for chunk in dst.chunks_mut(8) {
            let bytes = self.try_next_u64()?.to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
        Ok(())
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
            let mut r = SeededRngFactory.create(seed, label);
            (0..4).map(|_| r.next_u64()).collect::<Vec<_>>()
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
    fn scripted_returns_values_in_order() {
        let mut r = ScriptedRng::new([0.0, 0.5, 0.75, 1.0 - 2f64.powi(-53)]);
        let got: [f64; 4] = std::array::from_fn(|_| r.random());
        assert_eq!(got, [0.0, 0.5, 0.75, 1.0 - 2f64.powi(-53)]);
        assert_eq!(r.consumed(), 4);
    }

    #[test]
    fn chance_compares_one_draw() {
        let mut r = ScriptedRng::new([0.25, 0.25, 0.0]);
        assert_eq!(
            [
                chance(&mut r, 0.5),
                chance(&mut r, 0.25),
                chance(&mut r, 0.0)
            ],
            [true, false, false]
        );
        assert_eq!(r.consumed(), 3);
    }

    #[test]
    #[should_panic(expected = "exhausted")]
    fn scripted_panics_when_exhausted() {
        let mut r = ScriptedRng::new([0.0]);
        r.next_u64();
        r.next_u64();
    }
}
