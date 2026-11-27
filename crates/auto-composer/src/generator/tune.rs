//! A whole tune from the parameters.

use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::random::{Seed, layer_rng};
use crate::tune::{NoteNode, Resolution, Scale, TimeMeasure, Tune};

use super::{cadence, chord, note, pitch_weight, rhythm};

/// The parameters of a tune, as in the JSON files (`resources/TuneGenParams_*.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "tsify", tsify(rename = "TuneParameters"))]
#[cfg_attr(feature = "schemars", schemars(rename = "TuneParameters"))]
pub struct Parameters {
    /// The seed of every layer that has no seed of its own.
    pub seed: Seed,
    pub scale: Scale,
    pub time_measure: TimeMeasure,
    pub resolution: Resolution,
    #[cfg_attr(feature = "tsify", tsify(type = "CadenceParameters"))]
    pub cadence: cadence::Parameters,
    #[cfg_attr(feature = "tsify", tsify(type = "ChordParameters"))]
    pub chord: chord::Parameters,
    #[cfg_attr(feature = "tsify", tsify(type = "RhythmParameters"))]
    pub rhythm: rhythm::Parameters,
    #[cfg_attr(feature = "tsify", tsify(type = "PitchParameters"))]
    pub pitch: pitch_weight::Parameters,
}

/// `resources/TuneGenParams_default.json`, the single source of the default values.
impl Default for Parameters {
    fn default() -> Self {
        serde_json::from_str(include_str!(
            "../../../../resources/TuneGenParams_default.json"
        ))
        .expect("the default tune parameters are valid")
    }
}

impl Parameters {
    /// The generator of a layer: its own seed if it has one, or the seed of the tune.
    pub fn layer_rng(&self, layer_seed: Option<Seed>, label: &str) -> Pcg32 {
        layer_rng(layer_seed.unwrap_or(self.seed), label)
    }
}

/// Generates the cadences, the chords and the notes in this order.
pub fn generate(params: &Parameters) -> Result<Tune> {
    let mut tune = empty(params)?;
    tune.cadence = cadence::generate(
        &tune,
        &params.cadence,
        &mut params.layer_rng(params.cadence.seed, "cadence"),
    )?;
    tune.chord = chord::generate(
        &tune,
        &params.chord,
        &mut params.layer_rng(params.chord.seed, "chord"),
    )?;
    tune.notes = note::tree_to_timeline(&generate_notes(&tune, params)?)?;
    Ok(tune)
}

/// The rhythm tree of the tune with the pitches of its leaves.
pub fn generate_notes(tune: &Tune, params: &Parameters) -> Result<NoteNode> {
    let rhythm = rhythm::generate(
        tune,
        &params.rhythm,
        &mut params.layer_rng(params.rhythm.seed, "rhythm"),
    );
    note::pitch_tree(
        tune,
        rhythm,
        &params.pitch,
        &mut params.layer_rng(params.pitch.seed, "pitch"),
    )
}

/// A tune with the scale, the measure and the resolution of `params`, and no cadences,
/// chords or notes yet.
pub fn empty(params: &Parameters) -> Result<Tune> {
    // The degrees of the chords would divide by zero.
    if params.scale.tones.is_empty() {
        return Err(Error::InvalidParameter("scale.tones"));
    }
    Ok(Tune::new(
        params.time_measure,
        params.scale.clone(),
        params.resolution,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::default_params;

    #[test]
    fn reads_the_resource_files() {
        let p = default_params();
        assert_eq!(p.time_measure, TimeMeasure::new(4, 4));
        assert_eq!(p.resolution, Resolution::Half);
        assert_eq!(p.rhythm.max_beat_division_depth, 3);
        // Serializing and reading back gives the same parameters.
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Parameters>(&json).unwrap(), p);
        // The seeds of the layers are omitted when they are not set.
        assert!(!json.contains("null"), "{json}");
    }

    #[test]
    fn a_seed_of_a_layer_changes_only_that_layer() {
        let p = default_params();
        let base = generate(&p).unwrap();
        let mut q = p.clone();
        q.rhythm.seed = Some(p.seed + 1);
        let other = generate(&q).unwrap();
        assert_eq!((&other.cadence, &other.chord), (&base.cadence, &base.chord));
        assert_ne!(other.notes, base.notes);
        // A layer seed equal to the seed of the tune changes nothing.
        q.rhythm.seed = Some(p.seed);
        assert_eq!(generate(&q).unwrap(), base);
    }

    #[test]
    fn an_empty_scale_is_an_error() {
        let mut p = default_params();
        p.scale.tones.clear();
        assert_eq!(generate(&p), Err(Error::InvalidParameter("scale.tones")));
    }
}
