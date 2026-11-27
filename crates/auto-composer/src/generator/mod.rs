//! The generators of the layers of a tune (`src/core/{cadence,chord,...}.ts`).
//!
//! Each `generate` takes the random number sources from a [`RngFactory`](crate::random::RngFactory)
//! with the seed of its parameters, as the TypeScript version does with its `GenerateContext`.
//! The steps inside take a `&mut impl Rng` so that tests can feed scripted values.

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

    use rand::{Rng, TryRng};
    use rand_pcg::Pcg32;

    use crate::random::unit_to_u64;
    use crate::tune::{Resolution, Scale, Tune};
    use crate::util::{Timeline, TimelineItem};

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

    /// `emptyTune` of generators.test.ts.
    pub fn empty_tune(time_measure: [u32; 2], resolution: Resolution, depth: u32) -> Tune {
        let p = default_params();
        Tune {
            length: f64::from(time_measure[0] * time_measure[1]),
            time_measure,
            max_beat_division_depth: depth,
            scale: Scale::new(p.scale.key, p.scale.tones),
            resolution: Timeline::from_items([TimelineItem::new(0.0, resolution)]),
            ..Tune::default()
        }
    }

    pub const SEEDS: [(i64, i64); 5] = [(4649, 459), (1, 0), (-1, -7), (13, 21), (100, 3)];

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
    pub fn seeded(state: i64, sequence: i64) -> Pcg32 {
        Pcg32::new(state as u64, sequence as u64)
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
