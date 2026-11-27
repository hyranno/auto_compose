//! The weights of the pitch candidates of a note.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::random::Seed;
use crate::tune::{Notenum, Rhythm, Tune};
use crate::util::SmoothstepParams;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "tsify", tsify(rename = "PitchParameters"))]
#[cfg_attr(feature = "schemars", schemars(rename = "PitchParameters"))]
pub struct Parameters {
    /// Replaces the seed of the tune for this layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "tsify", tsify(optional))]
    pub seed: Option<Seed>,
    /// Lowers the weight by the distance from the key of the scale.
    pub abs_pitch_factor: SmoothstepParams,
    /// Lowers the weight by the distance from the previous pitch.
    pub rel_pitch_factor: SmoothstepParams,
    pub factor_in_scale: f64,
    pub factor_in_chord: f64,
    /// Longer notes sharpen the distribution: the weight is raised to
    /// `1 + duration * rhythm_exponent_factor`.
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

/// Applies the chord at the time of `rhythm` and its duration to a weight of
/// [`calc_time_homo_weight`].
pub fn calc_with_time_homo_weight(
    tune: &Tune,
    time_homo_weight: f64,
    candidate: Notenum,
    rhythm: &Rhythm,
    params: &Parameters,
) -> Result<f64> {
    let chord = &tune
        .chord
        .get(rhythm.t)
        .ok_or(Error::NoItemAt { timeline: "chord" })?
        .value;
    let factor_in_chord = if chord.includes(&tune.scale, candidate) {
        params.factor_in_chord
    } else {
        1.0
    };
    let rhythm_exponent = rhythm.duration * params.rhythm_exponent_factor;
    Ok((time_homo_weight * factor_in_chord).powf(1.0 + rhythm_exponent))
}

/// The weight of `candidate` for a note of `rhythm` after `prev`.
pub fn calc(
    tune: &Tune,
    prev: Notenum,
    candidate: Notenum,
    rhythm: &Rhythm,
    params: &Parameters,
) -> Result<f64> {
    let w = calc_time_homo_weight(tune, prev, candidate, params);
    calc_with_time_homo_weight(tune, w, candidate, rhythm, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::{default_params, empty_tune};
    use crate::tune::{Chord, Resolution, Scale};
    use crate::util::{Timeline, TimelineItem};

    fn tune() -> Tune {
        let mut tune = empty_tune([4, 4], Resolution::Half);
        tune.scale = Scale::major(69); // A major: 69 71 73 74 76 78 80
        tune.chord = Timeline::from_items([
            TimelineItem::new(0.0, Chord::new(0, vec![0, 2, 4])), // 69 73 76
            TimelineItem::new(8.0, Chord::new(4, vec![0, 2, 4])), // 76 80 83
        ]);
        tune
    }

    #[test]
    fn time_homo_weight() {
        let p = default_params().pitch;
        let tune = tune();
        // (prev, candidate, expected) = abs_pitch_factor * rel_pitch_factor * factor_in_scale
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
        let p = default_params().pitch;
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
            let rhythm = Rhythm {
                is_note_on: true,
                t,
                duration,
            };
            assert_eq!(
                calc(&tune, prev, pitch, &rhythm, &p),
                Ok(expected),
                "{prev} -> {pitch}"
            );
        }
    }

    #[test]
    fn no_chord_yet_is_an_error() {
        let p = default_params().pitch;
        let mut tune = tune();
        tune.chord = Timeline::new();
        let rhythm = Rhythm {
            is_note_on: true,
            t: 0.0,
            duration: 1.0,
        };
        assert_eq!(
            calc(&tune, 69, 69, &rhythm, &p),
            Err(Error::NoItemAt { timeline: "chord" })
        );
    }
}
