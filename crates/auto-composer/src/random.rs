//! Random number sources.
//!
//! The generators draw from a [`rand::Rng`]. Each layer of a tune draws from a generator of its
//! own ([`layer_rng`]), seeded by the seed of the tune or the seed of the layer if it has one.

use std::convert::Infallible;

pub use rand::Rng;
use rand::{RngExt, SeedableRng, TryRng};
use rand_pcg::Pcg32;

/// The seed of a tune, of a layer or of a variant.
#[cfg_attr(feature = "tsify", tsify::declare)]
pub type Seed = u32;

/// `true` with the probability `p`, drawing one number. `p` is effectively clamped to `[0, 1]`.
pub fn chance(rng: &mut (impl Rng + ?Sized), p: f64) -> bool {
    rng.random::<f64>() < p
}

/// The generator of the layer `label` (such as `"cadence"`) for `seed`. Each label has its own
/// sequence, so layers given the same seed draw independent numbers.
pub fn layer_rng(seed: Seed, label: &str) -> Pcg32 {
    Pcg32::seed_from_u64(fnv1a(label.as_bytes()) ^ u64::from(seed))
}

/// 64-bit FNV-1a.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
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
    fn fnv1a_matches_its_reference() {
        assert_eq!(fnv1a(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a(b"a"), 0xaf63dc4c8601ec8c);
    }

    #[test]
    fn each_label_has_its_own_sequence() {
        let draw = |seed, label| {
            let mut r = layer_rng(seed, label);
            (0..4).map(|_| r.next_u64()).collect::<Vec<_>>()
        };
        let labels = ["cadence", "chord", "rhythm", "pitch", "variant"];
        let seqs: Vec<_> = labels.iter().map(|l| draw(4649, l)).collect();
        for (i, a) in seqs.iter().enumerate() {
            assert_eq!(*a, draw(4649, labels[i]), "deterministic");
            assert_ne!(*a, draw(4650, labels[i]));
            for b in &seqs[i + 1..] {
                assert_ne!(a, b);
            }
        }
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
