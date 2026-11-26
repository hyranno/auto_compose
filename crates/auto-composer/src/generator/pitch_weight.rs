//! The weights of the pitch candidates of a note (`src/core/pitch_weight.ts`).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::tune::{Note, Notenum, RootedTones, Tune};
use crate::util::SmoothstepParams;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    /// Lowers the weight by the distance from the key of the scale.
    #[serde(rename = "absPitchFactor")]
    pub abs_pitch_factor: SmoothstepParams,
    /// Lowers the weight by the distance from the previous pitch.
    #[serde(rename = "relPitchFactor")]
    pub rel_pitch_factor: SmoothstepParams,
    #[serde(rename = "factorInScale")]
    pub factor_in_scale: f64,
    #[serde(rename = "factorInChord")]
    pub factor_in_chord: f64,
    /// Longer notes sharpen the distribution: the weight is raised to
    /// `1 + duration * rhythmExponentFactor`.
    #[serde(rename = "rhythmExponentFactor")]
    pub rhythm_exponent_factor: f64,
}

fn distance_factor(edges: &SmoothstepParams, from: Notenum, to: Notenum) -> f64 {
    1.0 - edges.eval(f64::from((to - from).abs()))
}

/// The factors that do not depend on the time: the distances from the key and from the
/// previous pitch, and whether the candidate is on the scale.
pub fn calc_time_homo_weight(
    tune: &Tune,
    prev: Notenum,
    candidate: Notenum,
    params: &Parameters,
) -> f64 {
    let abs_pitch_factor = distance_factor(&params.abs_pitch_factor, tune.scale.key, candidate);
    let rel_pitch_factor = distance_factor(&params.rel_pitch_factor, prev, candidate);
    let factor_in_scale = if tune.scale.includes(candidate) {
        params.factor_in_scale
    } else {
        1.0
    };
    abs_pitch_factor * rel_pitch_factor * factor_in_scale
}

/// Applies the chord at `t` and the duration to a weight of [`calc_time_homo_weight`].
pub fn calc_with_time_homo_weight(
    tune: &Tune,
    time_homo_weight: f64,
    candidate: &Note,
    t: f64,
    params: &Parameters,
) -> Result<f64> {
    let chord = &tune
        .chord
        .get(t)
        .ok_or(Error::NoItemAt { timeline: "chord" })?
        .value;
    let factor_in_chord = if chord.includes(candidate.pitch) {
        params.factor_in_chord
    } else {
        1.0
    };
    let rhythm_exponent = candidate.duration * params.rhythm_exponent_factor;
    Ok((time_homo_weight * factor_in_chord).powf(1.0 + rhythm_exponent))
}

pub fn calc(
    tune: &Tune,
    prev: Notenum,
    candidate: &Note,
    t: f64,
    params: &Parameters,
) -> Result<f64> {
    let w = calc_time_homo_weight(tune, prev, candidate.pitch, params);
    calc_with_time_homo_weight(tune, w, candidate, t, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::{default_params, empty_tune};
    use crate::tune::{Chord, Resolution, Scale};
    use crate::util::{Timeline, TimelineItem};

    // Ported from src/core/generators.test.ts.

    fn tune() -> Tune {
        let mut tune = empty_tune([4, 4], Resolution::Half, 3);
        tune.scale = Scale::major(69); // A major: 69 71 73 74 76 78 80
        tune.chord = Timeline::from_items([
            TimelineItem::new(0.0, Chord::new(tune.scale.clone(), 0, vec![0, 2, 4])), // 69 73 76
            TimelineItem::new(8.0, Chord::new(tune.scale.clone(), 4, vec![0, 2, 4])), // 76 80 83
        ]);
        tune
    }

    #[test]
    fn time_homo_weight() {
        let p = default_params().note.pitch_weight;
        let tune = tune();
        // (prev, candidate, expected) = absPitchFactor * relPitchFactor * factorInScale
        let cases = [
            (69, 69, 8.0), // 1 * 1 * 8
            (69, 70, 1.0), // not in scale
            (69, 73, 8.0), // distance 4: both factors still 1
            (69, 74, 4.0), // relative distance 5: 1 - smoothstep(4, 6, 5) = 0.5
            (69, 76, 0.0), // relative distance 7 > 6
            (74, 76, 4.0), // absolute distance 7: 1 - smoothstep(6, 8, 7) = 0.5
            (62, 61, 0.0), // absolute distance 8
            (68, 63, 0.5), // absolute distance 6 -> 1, relative 5 -> 0.5, not in scale
            (75, 64, 0.0), // relative distance 11
        ];
        for (prev, candidate, expected) in cases {
            assert_eq!(
                calc_time_homo_weight(&tune, prev, candidate, &p),
                expected,
                "{prev} -> {candidate}"
            );
        }
    }

    #[test]
    fn calc_raises_the_weight_with_the_chord_factor() {
        let p = default_params().note.pitch_weight;
        let tune = tune();
        // (prev, pitch, duration, t, expected)
        let cases = [
            (69, 69, 0.5, 0.0, (8.0f64 * 4.0).powi(2)), // in chord at t = 0
            (69, 69, 0.5, 8.0, 8.0f64.powi(2)),         // not in the chord at t = 8
            (69, 74, 1.0, 0.0, 4.0f64.powi(3)),
            (72, 76, 0.25, 9.0, (0.5f64 * 8.0 * 4.0).powf(1.5)), // absolute distance 7 halves it
            (69, 70, 2.0, 0.0, 1.0),
            (69, 76, 1.0, 0.0, 0.0),
        ];
        for (prev, pitch, duration, t, expected) in cases {
            let note = Note {
                pitch,
                duration,
                is_note_on: true,
            };
            assert_eq!(
                calc(&tune, prev, &note, t, &p),
                Ok(expected),
                "{prev} -> {pitch}"
            );
        }
    }

    #[test]
    fn no_chord_yet_is_an_error() {
        let p = default_params().note.pitch_weight;
        let mut tune = tune();
        tune.chord = Timeline::new();
        let note = Note {
            pitch: 69,
            duration: 1.0,
            is_note_on: true,
        };
        assert_eq!(
            calc(&tune, 69, &note, 0.0, &p),
            Err(Error::NoItemAt { timeline: "chord" })
        );
    }
}
