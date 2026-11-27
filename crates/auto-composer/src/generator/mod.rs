//! The generators of the layers of a tune.
//!
//! Each layer draws from a generator of its own ([`crate::random::layer_rng`]), which
//! [`tune::generate`] derives from the seeds of the parameters. The steps take a
//! `&mut impl Rng`, so that tests can feed scripted values.

pub mod cadence;
pub mod chord;
pub mod note;
pub mod pitch_weight;
pub mod rhythm;
pub mod tune;
pub mod variant;

#[cfg(test)]
pub(crate) mod test_util {
    use std::convert::Infallible;

    use rand::{Rng, SeedableRng, TryRng};
    use rand_pcg::Pcg32;

    use crate::random::unit_to_u64;
    use crate::tune::{Resolution, TimeMeasure, Tune};

    use super::tune::Parameters;

    pub fn default_params() -> Parameters {
        Parameters::default()
    }

    pub fn simple_params() -> Parameters {
        serde_json::from_str(include_str!(
            "../../../../resources/TuneGenParams_simple.json"
        ))
        .unwrap()
    }

    /// A tune of `[beats per bar, bars]` with the default scale and no layers yet.
    pub fn empty_tune([beats_per_bar, bars]: [u32; 2], resolution: Resolution) -> Tune {
        let measure = TimeMeasure::new(beats_per_bar, bars);
        Tune::new(measure, default_params().scale, resolution)
    }

    pub const SEEDS: [u32; 5] = [4649, 0, 1, 13, u32::MAX];

    /// Counts the words drawn from `inner`.
    pub struct Counting<R> {
        pub inner: R,
        pub consumed: usize,
    }

    impl<R: Rng> TryRng for Counting<R> {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Infallible> {
            self.consumed += 1;
            Ok(self.inner.next_u32())
        }

        fn try_next_u64(&mut self) -> Result<u64, Infallible> {
            self.consumed += 1;
            Ok(self.inner.next_u64())
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
            self.consumed += 1;
            self.inner.fill_bytes(dst);
            Ok(())
        }
    }

    /// A generator for a seed of [`SEEDS`].
    pub fn seeded(seed: u32) -> Pcg32 {
        Pcg32::seed_from_u64(u64::from(seed))
    }

    /// Returns the same value in `[0, 1)` forever (see [`crate::random::ScriptedRng`]) and
    /// counts the words drawn.
    #[derive(Debug, Default)]
    pub struct ConstantRng {
        value: f64,
        pub consumed: usize,
    }

    impl ConstantRng {
        pub fn new(value: f64) -> Self {
            Self { value, consumed: 0 }
        }
    }

    impl TryRng for ConstantRng {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Infallible> {
            Ok((self.try_next_u64()? >> 32) as u32)
        }

        fn try_next_u64(&mut self) -> Result<u64, Infallible> {
            self.consumed += 1;
            Ok(unit_to_u64(self.value))
        }

        fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), Infallible> {
            unimplemented!("ConstantRng gives words only")
        }
    }
}
