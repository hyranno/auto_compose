//! Chords: a root for each cadence, then the tones.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::random::{Rng, Seed};
use crate::tune::{Cadence, Chord, Degree, PerCadence, Tune};
use crate::util::{Timeline, TimelineItem, WeightedItem, weighted_pick};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "tsify", tsify(rename = "ChordParameters"))]
#[cfg_attr(feature = "schemars", schemars(rename = "ChordParameters"))]
pub struct Parameters {
    /// Replaces the seed of the tune for this layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "tsify", tsify(optional))]
    pub seed: Option<Seed>,
    /// The weights of the root degrees for each cadence.
    pub roots: PerCadence<Vec<WeightedItem<Degree>>>,
    /// The weights of the tones (degrees from the root).
    pub tones: Vec<WeightedItem<Vec<Degree>>>,
}

impl Parameters {
    /// Draws a root for `cadence` and then the tones.
    pub fn draw(&self, cadence: Cadence, rng: &mut impl Rng) -> Result<Chord> {
        let root = *weighted_pick(self.roots.get(cadence), rng)?;
        let tones = weighted_pick(&self.tones, rng)?;
        Ok(Chord::new(root, tones.clone()))
    }
}

/// Draws a chord for each cadence of the tune.
pub fn generate(tune: &Tune, params: &Parameters, rng: &mut impl Rng) -> Result<Timeline<Chord>> {
    let events = tune
        .cadence
        .list()
        .iter()
        .map(|c| Ok(TimelineItem::new(c.t, params.draw(c.value, rng)?)))
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::{default_params, empty_tune};
    use crate::random::ScriptedRng;
    use crate::tune::Resolution;

    #[test]
    fn scripted_draws_pick_root_and_tones_from_the_tables() {
        use Cadence::*;
        let mut tune = empty_tune([4, 4], Resolution::Half);
        tune.cadence = Timeline::from_items(
            [Tonic, Subdominant, Dominant, Tonic]
                .into_iter()
                .zip(0..)
                .map(|(c, i)| TimelineItem::new(f64::from(i * 4), c)),
        );
        let params = default_params().chord;
        // A root and then the tones for each cadence.
        // Roots: T 0.69 -> 0 (ratio 0.7), S 0.9499 -> 1 (0.6 + 0.35 == 0.95), D 0.9 -> 6,
        // T 0.999 -> 2.
        // Tones: 0.79 -> [0,2,4] (ratio 0.8), 0.8 -> [0,2,4,6], 0.94 -> [0,2,4,6] (0.95),
        // 0.96 -> [0,4].
        let mut rng = ScriptedRng::new([0.69, 0.79, 0.9499, 0.8, 0.9, 0.94, 0.999, 0.96]);
        let chords = generate(&tune, &params, &mut rng).unwrap();
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
        assert_eq!(rng.consumed(), 8);
    }
}
