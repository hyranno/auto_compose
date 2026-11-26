//! Plain-JSON form of a [`Tune`], the format of the golden fixtures
//! (the same as `src/core/snapshot.ts` of the TypeScript implementation).

use serde::{Deserialize, Serialize};

use crate::tune::{Cadence, Degree, Notenum, Resolution, RootedTones, Scale, Tune};
use crate::util::TimelineItem;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TuneSnapshot {
    pub length: f64,
    pub time_measure: [u32; 2],
    pub max_beat_division_depth: u32,
    pub scale: Scale,
    pub resolution: Vec<TimelineItem<Resolution>>,
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
    /// Redundant with `root` and `tones`; kept to cross-check [`RootedTones::notenums`].
    pub notenums: Vec<Notenum>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct NoteSnapshot {
    pub t: f64,
    pub pitch: Notenum,
    pub duration: f64,
    #[serde(rename = "isNoteOn")]
    pub is_note_on: bool,
}

impl From<&Tune> for TuneSnapshot {
    fn from(tune: &Tune) -> Self {
        Self {
            length: tune.length,
            time_measure: tune.time_measure,
            max_beat_division_depth: tune.max_beat_division_depth,
            scale: tune.scale.clone(),
            resolution: tune.resolution.list().to_vec(),
            cadence: tune.cadence.list().to_vec(),
            chord: tune
                .chord
                .list()
                .iter()
                .map(|e| ChordSnapshot {
                    t: e.t,
                    root: e.value.root,
                    tones: e.value.tones.clone(),
                    notenums: e.value.notenums(),
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
                    is_note_on: e.value.is_note_on,
                })
                .collect(),
        }
    }
}
