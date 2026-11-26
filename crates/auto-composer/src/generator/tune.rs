//! A whole tune from the parameters (`src/core/tune_generator.ts`).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::random::RngFactory;
use crate::tune::{Resolution, Scale, Tune};
use crate::util::{Timeline, TimelineItem};

use super::{cadence, chord, note};

/// The parameters of a tune, read from the same JSON as the TypeScript version
/// (`resources/TuneGenParams_*.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub scale: Scale,
    /// Beats per bar and bars.
    pub time_measure: [u32; 2],
    pub resolution: Resolution,
    pub cadence: cadence::Parameters,
    pub chord: chord::Parameters,
    pub note: note::Parameters,
}

/// Generates the cadences, the chords and the notes in this order.
pub fn generate(params: &Parameters, factory: &dyn RngFactory) -> Result<Tune> {
    // The TypeScript version gives NaN pitches for an empty scale; here the degrees of the
    // chords would divide by zero.
    if params.scale.tones.is_empty() {
        return Err(Error::InvalidParameter("scale.tones"));
    }
    let mut tune = Tune {
        length: f64::from(params.time_measure[0] * params.time_measure[1]),
        time_measure: params.time_measure,
        max_beat_division_depth: params.note.rhythm.max_beat_division_depth,
        scale: params.scale.clone(),
        resolution: Timeline::from_items([TimelineItem::new(0.0, params.resolution)]),
        ..Tune::default()
    };
    tune.cadence = cadence::generate(&tune, &params.cadence, factory)?;
    tune.chord = chord::generate(&tune, &params.chord, factory)?;
    tune.notes = note::generate(&tune, &params.note, factory)?;
    Ok(tune)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::default_params;
    use crate::random::ReliableRandomFactory;

    #[test]
    fn reads_the_resource_files() {
        let p = default_params();
        assert_eq!(p.time_measure, [4, 4]);
        assert_eq!(p.resolution, Resolution::Half);
        assert_eq!(p.note.rhythm.max_beat_division_depth, 3);
        // Serializing and reading back gives the same parameters.
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Parameters>(&json).unwrap(), p);
    }

    #[test]
    fn an_empty_scale_is_an_error() {
        let mut p = default_params();
        p.scale.tones.clear();
        assert_eq!(
            generate(&p, &ReliableRandomFactory),
            Err(Error::InvalidParameter("scale.tones"))
        );
    }
}
