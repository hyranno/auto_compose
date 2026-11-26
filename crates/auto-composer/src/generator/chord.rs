//! Chords: a root for each cadence, then the tones (`src/core/chord.ts`).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::random::{RandomSeed, Rng, RngFactory};
use crate::tune::{Cadence, Chord, Degree, Tune};
use crate::util::{Timeline, TimelineItem, WeightedItem, weighted_pick};

/// `RootGenerator.Parameters` in TypeScript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootParameters {
    pub seed: RandomSeed,
    /// The weights of the root degrees for each cadence.
    pub probabilities: Vec<(Cadence, Vec<WeightedItem<Degree>>)>,
}

/// `WeightedRandomParameters` in TypeScript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightedRandomParameters<T> {
    pub items: Vec<WeightedItem<T>>,
    pub seed: RandomSeed,
}

/// `ToneGenerator.Parameters` in TypeScript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToneParameters {
    /// The weights of the tones (degrees from the root).
    pub rand_tones: WeightedRandomParameters<Vec<Degree>>,
}

/// `ToneModifier.Parameters` in TypeScript. It has no fields yet.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModifierParameters {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub root: RootParameters,
    pub tone: ToneParameters,
    #[serde(default)]
    pub modifier: ModifierParameters,
}

pub fn generate(
    tune: &Tune,
    params: &Parameters,
    factory: &dyn RngFactory,
) -> Result<Timeline<Chord>> {
    let roots = generate_roots(
        tune,
        &params.root,
        &mut factory.create(params.root.seed, "chord.root"),
    )?;
    generate_tones(
        tune,
        &roots,
        &params.tone,
        &mut factory.create(params.tone.rand_tones.seed, "chord.tone"),
    )
}

/// The weights of the roots for `cadence`. Like `new Map(...).get`, a later entry for the same
/// cadence wins.
pub(crate) fn root_items(
    params: &RootParameters,
    cadence: Cadence,
) -> Result<&[WeightedItem<Degree>]> {
    params
        .probabilities
        .iter()
        .rev()
        .find(|(k, _)| *k == cadence)
        .map(|(_, items)| items.as_slice())
        .ok_or(Error::MissingRootProbabilities(cadence))
}

/// Draws a root for each cadence of the tune (`RootGenerator.generate`).
pub fn generate_roots(
    tune: &Tune,
    params: &RootParameters,
    rng: &mut impl Rng,
) -> Result<Timeline<Degree>> {
    let events = tune
        .cadence
        .list()
        .iter()
        .map(|c| {
            let items = root_items(params, c.value)?;
            let root = *weighted_pick(items, rng).ok_or(Error::NoItems)?;
            Ok(TimelineItem::new(c.t, root))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(events))
}

/// Draws the tones for each root (`ToneGenerator.generate`).
pub fn generate_tones(
    tune: &Tune,
    roots: &Timeline<Degree>,
    params: &ToneParameters,
    rng: &mut impl Rng,
) -> Result<Timeline<Chord>> {
    let events = roots
        .list()
        .iter()
        .map(|r| {
            let tones = weighted_pick(&params.rand_tones.items, rng).ok_or(Error::NoItems)?;
            Ok(TimelineItem::new(
                r.t,
                Chord::new(tune.scale.clone(), r.value, tones.clone()),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::{default_params, empty_tune};
    use crate::random::ScriptedRng;
    use crate::tune::Resolution;

    // Ported from src/core/generators.test.ts.

    #[test]
    fn scripted_draws_pick_root_and_tones_from_the_tables() {
        let mut tune = empty_tune([4, 4], Resolution::Half, 3);
        tune.cadence = Timeline::from_items(
            [Cadence::T, Cadence::S, Cadence::D, Cadence::T]
                .into_iter()
                .zip(0..)
                .map(|(c, i)| TimelineItem::new(f64::from(i * 4), c)),
        );
        let params = default_params().chord;
        // Root draws (one per cadence): T 0.7 -> 0 (ratio exactly 0.7), S 0.95 -> 1
        // (0.6 + 0.35 == 0.95), D 0.9 -> 6, T 1-2^-32 -> 2
        let mut root_rng = ScriptedRng::new([0.7, 0.95, 0.9, 1.0 - 2f64.powi(-32)]);
        // Tone draws (one per chord): 0.8 -> [0,2,4], 0.81 -> [0,2,4,6],
        // 0.95 -> [0,2,4,6] (0.8 + 0.15 == 0.9500000000000001), 0.96 -> [0,4]
        let mut tone_rng = ScriptedRng::new([0.8, 0.81, 0.95, 0.96]);
        let roots = generate_roots(&tune, &params.root, &mut root_rng).unwrap();
        let chords = generate_tones(&tune, &roots, &params.tone, &mut tone_rng).unwrap();
        let actual: Vec<(f64, Degree, Vec<Degree>)> = chords
            .list()
            .iter()
            .map(|e| (e.t, e.value.root, e.value.tones.clone()))
            .collect();
        assert_eq!(
            actual,
            [
                (0.0, 0, vec![0, 2, 4]),
                (4.0, 1, vec![0, 2, 4, 6]),
                (8.0, 6, vec![0, 2, 4, 6]),
                (12.0, 2, vec![0, 4]),
            ]
        );
        assert_eq!([root_rng.consumed(), tone_rng.consumed()], [4, 4]);
    }

    #[test]
    fn a_cadence_without_probabilities_is_an_error() {
        let mut tune = empty_tune([4, 4], Resolution::Half, 3);
        tune.cadence = Timeline::from_items([TimelineItem::new(0.0, Cadence::S)]);
        let mut params = default_params().chord.root;
        params.probabilities.retain(|(c, _)| *c != Cadence::S);
        assert_eq!(
            generate_roots(&tune, &params, &mut ScriptedRng::new([0.0])),
            Err(Error::MissingRootProbabilities(Cadence::S))
        );
    }
}
