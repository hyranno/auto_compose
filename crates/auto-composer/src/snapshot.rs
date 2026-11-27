//! Plain-JSON form of a [`Tune`], the format of the golden fixtures and of the tunes given to
//! the web page.

use serde::{Deserialize, Serialize};

use crate::tune::{Cadence, Degree, Notenum, Resolution, Scale, TimeMeasure, Tune};
use crate::util::TimelineItem;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TuneSnapshot {
    /// In beats.
    pub length: f64,
    pub time_measure: TimeMeasure,
    pub scale: Scale,
    pub resolution: Resolution,
    pub cadence: Vec<TimelineItem<Cadence>>,
    pub chord: Vec<ChordSnapshot>,
    pub notes: Vec<NoteSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ChordSnapshot {
    pub t: f64,
    pub root: Degree,
    pub tones: Vec<Degree>,
    /// The tones on the scale, from [`Chord::notenums`](crate::tune::Chord::notenums).
    pub notenums: Vec<Notenum>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct NoteSnapshot {
    pub t: f64,
    /// Omitted for a rest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "tsify", tsify(optional))]
    pub pitch: Option<Notenum>,
    pub duration: f64,
    /// Whether this is a note rather than a rest: whether it has a pitch.
    pub is_note_on: bool,
}

impl From<&Tune> for TuneSnapshot {
    fn from(tune: &Tune) -> Self {
        Self {
            length: tune.length,
            time_measure: tune.time_measure,
            scale: tune.scale.clone(),
            resolution: tune.resolution,
            cadence: tune.cadence.list().to_vec(),
            chord: tune
                .chord
                .list()
                .iter()
                .map(|e| ChordSnapshot {
                    t: e.t,
                    root: e.value.root,
                    tones: e.value.tones.clone(),
                    notenums: e.value.notenums(&tune.scale).collect(),
                })
                .collect(),
            notes: tune
                .notes
                .list()
                .iter()
                .map(|e| NoteSnapshot {
                    t: e.t,
                    pitch: e.value.pitch,
                    duration: e.value.duration,
                    is_note_on: e.value.is_note_on(),
                })
                .collect(),
        }
    }
}
