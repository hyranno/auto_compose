//! Harmonic functions by a Markov chain.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::markov::{Condition, MarkovChain};
use crate::random::{Rng, Seed};
use crate::tune::{Cadence, PerCadence, Tune};
use crate::util::{Timeline, TimelineItem, WeightedItem, range_step};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "tsify", tsify(rename = "CadenceParameters"))]
#[cfg_attr(feature = "schemars", schemars(rename = "CadenceParameters"))]
pub struct Parameters {
    /// Replaces the seed of the tune for this layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "tsify", tsify(optional))]
    pub seed: Option<Seed>,
    /// The length of a cadence in beats.
    pub duration: f64,
    /// The transition weights from each cadence to each cadence.
    pub transitions: PerCadence<PerCadence<f64>>,
}

impl Parameters {
    /// Checks the duration and that each row of the transitions can be made into probabilities.
    pub fn validate(&self) -> Result<()> {
        if !(self.duration > 0.0 && self.duration.is_finite()) {
            return Err(Error::InvalidParameter("cadence.duration"));
        }
        self.chain().map(|_| ()).map_err(|e| match e {
            Error::InvalidTransition { from } => {
                let names = PerCadence {
                    tonic: "cadence.transitions.tonic",
                    dominant: "cadence.transitions.dominant",
                    subdominant: "cadence.transitions.subdominant",
                };
                Error::InvalidParameter(names.get(Cadence::ALL[from]))
            }
            _ => Error::InvalidParameter("cadence.transitions"),
        })
    }

    /// The Markov chain of [`Parameters::transitions`].
    pub fn chain(&self) -> Result<MarkovChain<Cadence>> {
        let transition: Vec<_> = Cadence::ALL
            .iter()
            .map(|&from| {
                let row = self.transitions.get(from);
                let items = Cadence::ALL
                    .iter()
                    .map(|&to| WeightedItem::new(*row.get(to), to))
                    .collect();
                (from, items)
            })
            .collect();
        MarkovChain::new(&transition)
    }
}

/// Starts with the tonic, ends with the postfix of the resolution, and draws once for each
/// slot between them, conditioned on reaching the first cadence of the postfix.
/// When there are fewer slots than the tonic and the postfix, the start of the postfix is cut.
///
/// Panics if the duration is invalid ([`Parameters::validate`]).
pub fn generate(tune: &Tune, params: &Parameters, rng: &mut impl Rng) -> Result<Timeline<Cadence>> {
    let length = f64::from(tune.time_measure.beats());
    let times: Vec<f64> = range_step(0.0, length, params.duration).collect();
    let chain = params.chain()?;
    let start = Cadence::Tonic;
    let rest = times.len().saturating_sub(1);
    let postfix = tune.resolution.postfix();
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
    use crate::generator::test_util::{ConstantRng, SEEDS, default_params, empty_tune, seeded};
    use crate::random::ScriptedRng;
    use crate::tune::{Resolution, TimeMeasure};

    #[test]
    fn structure() {
        let base = default_params().cadence;
        for resolution in Resolution::ALL {
            for measure in [[4, 4], [3, 4], [2, 4]] {
                for duration in [1.0, 2.0] {
                    let tune = empty_tune(measure, resolution);
                    let postfix = resolution.postfix();
                    for seed in SEEDS {
                        let params = Parameters {
                            duration,
                            ..base.clone()
                        };
                        let tl = generate(&tune, &params, &mut seeded(seed)).unwrap();
                        let list = tl.list();
                        let what = format!("{resolution:?} {measure:?} {duration} {seed}");
                        assert_eq!(
                            list.len() as f64,
                            f64::from(measure[0] * measure[1]) / duration,
                            "{what}"
                        );
                        for (i, e) in list.iter().enumerate() {
                            assert_eq!(e.t, i as f64 * duration, "{what}");
                        }
                        assert_eq!(list[0].value, Cadence::Tonic, "{what}");
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
            (Resolution::PerfectAuthentic, 5),
            (Resolution::Plagal, 5),
        ];
        for (resolution, draws) in cases {
            let mut rng = ConstantRng::new(0.0);
            let tl = generate(&empty_tune([4, 4], resolution), &params, &mut rng).unwrap();
            assert_eq!(rng.consumed, draws);
            assert_eq!(tl.len(), 8);
        }
    }

    #[test]
    fn the_postfix_is_cut_when_there_are_fewer_slots_than_the_tonic_and_the_postfix() {
        let tune = empty_tune([2, 2], Resolution::PerfectAuthentic); // 2 slots, postfix [D, T]
        let mut rng = ScriptedRng::new([]);
        let tl = generate(&tune, &default_params().cadence, &mut rng).unwrap();
        let values: Vec<Cadence> = tl.list().iter().map(|e| e.value).collect();
        assert_eq!(values, [Cadence::Tonic, Cadence::Tonic]);
        let tune = Tune {
            time_measure: TimeMeasure::new(1, 1),
            ..tune
        };
        let tl = generate(&tune, &default_params().cadence, &mut rng).unwrap();
        assert_eq!(tl.len(), 1);
    }

    #[test]
    fn the_slot_before_the_postfix_never_transitions_to_an_impossible_state() {
        // Dominant -> Subdominant has probability 0, so the slot just before a Plagal postfix
        // [S, T] is never Dominant.
        let tune = empty_tune([4, 4], Resolution::Plagal);
        let params = default_params().cadence;
        for s in 0..50 {
            let tl = generate(&tune, &params, &mut seeded(s)).unwrap();
            let list = tl.list();
            assert_ne!(list[list.len() - 3].value, Cadence::Dominant, "seed {s}");
        }
    }

    #[test]
    fn rejects_a_duration_that_is_not_positive_and_finite() {
        for duration in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let params = Parameters {
                duration,
                ..default_params().cadence
            };
            assert_eq!(
                params.validate(),
                Err(Error::InvalidParameter("cadence.duration")),
                "{duration}"
            );
        }
        assert_eq!(default_params().cadence.validate(), Ok(()));
    }

    #[test]
    fn rejects_transitions_that_are_not_probabilities() {
        for weight in [-1.0, f64::NAN, f64::INFINITY] {
            let mut params = default_params().cadence;
            params.transitions.dominant.subdominant = weight;
            assert_eq!(
                params.validate(),
                Err(Error::InvalidParameter("cadence.transitions.dominant")),
                "{weight}"
            );
        }
        let mut params = default_params().cadence;
        params.transitions.subdominant = PerCadence {
            tonic: 0.0,
            dominant: 0.0,
            subdominant: 0.0,
        };
        assert_eq!(
            params.validate(),
            Err(Error::InvalidParameter("cadence.transitions.subdominant"))
        );
    }
}
