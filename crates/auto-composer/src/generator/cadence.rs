//! Harmonic functions by a Markov chain (`src/core/cadence.ts`).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::markov::{Condition, MarkovChain};
use crate::random::{RandomSeed, Rng, RngFactory};
use crate::tune::{Cadence, Tune};
use crate::util::{Timeline, TimelineItem, WeightedItem, range_step};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "tsify", tsify(rename = "CadenceParameters"))]
#[cfg_attr(feature = "schemars", schemars(rename = "CadenceParameters"))]
pub struct Parameters {
    pub seed: RandomSeed,
    /// The transition weights from each cadence.
    pub probabilities: Vec<(Cadence, Vec<WeightedItem<Cadence>>)>,
    /// The length of a cadence in beats.
    pub duration: f64,
}

pub fn generate(
    tune: &Tune,
    params: &Parameters,
    factory: &dyn RngFactory,
) -> Result<Timeline<Cadence>> {
    generate_with(tune, params, &mut factory.create(params.seed, "cadence"))
}

/// Starts with the tonic, ends with the postfix of the resolution, and draws once for each
/// slot between them, conditioned on reaching the first cadence of the postfix.
/// When there are fewer slots than the tonic and the postfix, the start of the postfix is cut.
pub fn generate_with(
    tune: &Tune,
    params: &Parameters,
    rng: &mut impl Rng,
) -> Result<Timeline<Cadence>> {
    if !(params.duration > 0.0 && params.duration.is_finite()) {
        return Err(Error::InvalidParameter("cadence.duration"));
    }
    let length = f64::from(tune.time_measure[0] * tune.time_measure[1]);
    let times: Vec<f64> = range_step(0.0, length, params.duration).collect();
    let resolution = tune
        .resolution
        .get(0.0)
        .ok_or(Error::NoItemAt {
            timeline: "resolution",
        })?
        .value;
    let chain = MarkovChain::new(&params.probabilities)?;
    let start = Cadence::T;
    let rest = times.len().saturating_sub(1);
    let postfix = resolution.postfix();
    let postfix = &postfix[postfix.len().saturating_sub(rest)..];
    let free = rest - postfix.len();
    let mut values = Vec::with_capacity(times.len());
    let mut prev = start;
    for i in 0..free {
        // `free - i` steps from the previous cadence to the postfix.
        prev = chain.sample_next(&prev, &Condition::new(free - i, postfix[0]), rng)?;
        values.push(prev);
    }
    let values = std::iter::once(start)
        .chain(values)
        .chain(postfix.iter().copied());
    Ok(Timeline::from_items(
        times
            .into_iter()
            .zip(values)
            .map(|(t, value)| TimelineItem::new(t, value)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::seeded;
    use crate::generator::test_util::{ConstantRng, SEEDS, default_params, empty_tune};
    use crate::random::ScriptedRng;
    use crate::tune::Resolution;

    // Ported from src/core/generators.test.ts.

    #[test]
    fn structure() {
        let base = default_params().cadence;
        for resolution in Resolution::ALL {
            for measure in [[4, 4], [3, 4], [2, 4]] {
                for duration in [1.0, 2.0] {
                    let tune = empty_tune(measure, resolution, 3);
                    let postfix = resolution.postfix();
                    for (state, sequence) in SEEDS {
                        let params = Parameters {
                            duration,
                            ..base.clone()
                        };
                        let mut rng = seeded(state, sequence);
                        let tl = generate_with(&tune, &params, &mut rng).unwrap();
                        let list = tl.list();
                        let what = format!("{resolution:?} {measure:?} {duration} {state}");
                        assert_eq!(
                            list.len() as f64,
                            f64::from(measure[0] * measure[1]) / duration,
                            "{what}"
                        );
                        for (i, e) in list.iter().enumerate() {
                            assert_eq!(e.t, i as f64 * duration, "{what}");
                        }
                        assert_eq!(list[0].value, Cadence::T, "{what}");
                        let tail: Vec<Cadence> = list[list.len() - postfix.len()..]
                            .iter()
                            .map(|e| e.value)
                            .collect();
                        assert_eq!(tail, postfix, "{what}");
                    }
                }
            }
        }
    }

    #[test]
    fn draws_once_per_slot_between_the_tonic_and_the_postfix() {
        // (resolution, expected draws) for 4/4 with duration 2 (8 slots, the tonic first)
        let params = default_params().cadence;
        let cases = [
            (Resolution::Half, 6),
            (Resolution::Perfect, 5),
            (Resolution::Plagal, 5),
        ];
        for (resolution, draws) in cases {
            let mut rng = ConstantRng::new(0.0);
            let tl = generate_with(&empty_tune([4, 4], resolution, 3), &params, &mut rng).unwrap();
            assert_eq!(rng.consumed, draws);
            assert_eq!(tl.len(), 8);
        }
    }

    #[test]
    fn the_postfix_is_cut_when_there_are_fewer_slots_than_the_tonic_and_the_postfix() {
        let tune = empty_tune([2, 2], Resolution::Perfect, 3); // 2 slots, postfix [D, T]
        let mut rng = ScriptedRng::new([]);
        let tl = generate_with(&tune, &default_params().cadence, &mut rng).unwrap();
        let values: Vec<Cadence> = tl.list().iter().map(|e| e.value).collect();
        assert_eq!(values, [Cadence::T, Cadence::T]);
    }

    #[test]
    fn the_slot_before_the_postfix_never_transitions_to_an_impossible_state() {
        // Dominant -> Subdominant has probability 0, so the slot just before a Plagal postfix
        // [S, T] is never Dominant.
        let tune = empty_tune([4, 4], Resolution::Plagal, 3);
        let params = default_params().cadence;
        for s in 0..50 {
            let mut rng = seeded(s, 1);
            let tl = generate_with(&tune, &params, &mut rng).unwrap();
            let list = tl.list();
            assert_ne!(list[list.len() - 3].value, Cadence::D, "seed {s}");
        }
    }

    #[test]
    fn rejects_a_duration_that_is_not_positive_and_finite() {
        let tune = empty_tune([4, 4], Resolution::Half, 3);
        for duration in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let params = Parameters {
                duration,
                ..default_params().cadence
            };
            assert_eq!(
                generate_with(&tune, &params, &mut ConstantRng::new(0.0)),
                Err(Error::InvalidParameter("cadence.duration")),
                "{duration}"
            );
        }
    }
}
