//! Random number sources.
//!
//! Generators take their random numbers from [`Rng`], so that tests can feed scripted values
//! ([`ScriptedRng`]) and record the consumption order ([`RecordingRng`]) to compare it with the
//! TypeScript implementation.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomSeed {
    pub state: i64,
    pub sequence: i64,
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
    log: &'a std::cell::RefCell<Vec<RngRecord>>,
}

impl<'a, R: Rng> RecordingRng<'a, R> {
    pub fn new(
        inner: R,
        label: impl Into<String>,
        log: &'a std::cell::RefCell<Vec<RngRecord>>,
    ) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

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
