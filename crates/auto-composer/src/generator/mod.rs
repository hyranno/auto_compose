//! The generators of the layers of a tune (`src/core/{cadence,chord,...}.ts`).
//!
//! Each `generate` takes the random number sources from a [`RngFactory`](crate::random::RngFactory)
//! with the seed of its parameters, as the TypeScript version does with its `GenerateContext`.
//! The steps inside take a `&mut impl Rng` so that tests can feed scripted values.
//! The random numbers are consumed in the same order as in TypeScript.

pub mod cadence;
pub mod chord;
pub mod note;
pub mod pitch_weight;
pub mod rhythm;
pub mod tune;
pub mod variant;

#[cfg(test)]
pub(crate) mod test_util {
    use crate::random::Rng;
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

    /// Returns the same value forever and counts the draws.
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

    impl Rng for ConstantRng {
        fn random(&mut self) -> f64 {
            self.consumed += 1;
            self.value
        }
    }
}
