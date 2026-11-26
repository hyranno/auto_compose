//! Tunes, scales, chords and the note tree (`src/core/tune.ts`).

use serde::{Deserialize, Serialize};

use crate::util::{Timeline, TimelineItem};

/// MIDI note number on 12 equal temperament.
#[cfg_attr(feature = "tsify", tsify::declare)]
pub type Notenum = i32;
/// Index on a scale or a chord.
#[cfg_attr(feature = "tsify", tsify::declare)]
pub type Degree = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Tune {
    pub length: f64,
    pub time_measure: [u32; 2],
    pub max_beat_division_depth: u32,
    pub scale: Scale,
    pub resolution: Timeline<Resolution>,
    pub cadence: Timeline<Cadence>,
    pub chord: Timeline<Chord>,
    pub notes: Timeline<Note>,
}

impl Default for Tune {
    fn default() -> Self {
        Self {
            length: 0.0,
            time_measure: [4, 4],
            max_beat_division_depth: 3,
            scale: Scale::major(64),
            resolution: Timeline::new(),
            cadence: Timeline::new(),
            chord: Timeline::new(),
            notes: Timeline::new(),
        }
    }
}

impl Tune {
    /// `self` followed by `src`. Like the TypeScript version, the resolution is not merged,
    /// and the time measure, the depth and the scale are taken from `self`.
    pub fn merge(&self, src: &Tune) -> Tune {
        let mut dest = Tune {
            length: self.length + src.length,
            time_measure: self.time_measure,
            max_beat_division_depth: self.max_beat_division_depth,
            scale: self.scale.clone(),
            ..Tune::default()
        };
        dest.cadence.merge(0.0, &self.cadence);
        dest.cadence.merge(self.length, &src.cadence);
        dest.chord.merge(0.0, &self.chord);
        dest.chord.merge(self.length, &src.chord);
        dest.notes.merge(0.0, &self.notes);
        dest.notes.merge(self.length, &src.notes);
        dest
    }
}

/// Operations shared by [`Scale`] and [`Chord`] (`RootedTones` in TypeScript).
pub trait RootedTones {
    fn notenums(&self) -> Vec<Notenum>;

    /// Whether `note` is one of the notenums, ignoring the octave.
    fn includes(&self, note: Notenum) -> bool {
        self.notenums().iter().any(|v| v % 12 == note % 12)
    }

    /// Whether `note` is the first notenum, ignoring the octave.
    fn is_root(&self, note: Notenum) -> bool {
        self.notenums().first().is_some_and(|v| v % 12 == note % 12)
    }

    /// The notenum of a degree, wrapping into octaves.
    ///
    /// Panics if there are no tones (the TypeScript version returns `NaN`).
    fn get(&self, pitch: Degree) -> Notenum {
        let notenums = self.notenums();
        let len = notenums.len() as i32;
        notenums[pitch.rem_euclid(len) as usize] + 12 * pitch.div_euclid(len)
    }

    /// The mean of the notenums. `NaN` if there are none (the TypeScript version throws).
    fn center(&self) -> f64 {
        let notenums = self.notenums();
        let Some((first, rest)) = notenums.split_first() else {
            return f64::NAN;
        };
        let sum = rest
            .iter()
            .fold(f64::from(*first), |acc, &v| acc + f64::from(v));
        sum / notenums.len() as f64
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Scale {
    pub key: Notenum,
    /// Offsets from the key in semitones.
    pub tones: Vec<Notenum>,
}

impl Scale {
    pub fn new(key: Notenum, tones: Vec<Notenum>) -> Self {
        Self { key, tones }
    }

    pub fn major(key: Notenum) -> Self {
        Self::new(key, vec![0, 2, 4, 5, 7, 9, 11])
    }

    pub fn minor(key: Notenum) -> Self {
        Self::new(key, vec![0, 2, 3, 5, 7, 8, 10])
    }
}

impl RootedTones for Scale {
    fn notenums(&self) -> Vec<Notenum> {
        self.tones.iter().map(|h| self.key + h).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    pub scale: Scale,
    /// Degree of the root on the scale.
    pub root: Degree,
    /// Degrees from the root on the scale.
    pub tones: Vec<Degree>,
}

impl Chord {
    pub fn new(scale: Scale, root: Degree, tones: Vec<Degree>) -> Self {
        Self { scale, root, tones }
    }

    /// Inversion: raises the lowest tone by an octave `times` times (lowers if negative).
    pub fn rot(&self, scale: &Scale, times: i32) -> Chord {
        let len = self.tones.len() as i32;
        let octave = times.div_euclid(len);
        let rest = times.rem_euclid(len);
        let n = scale.tones.len() as i32;
        let tones = self
            .tones
            .iter()
            .zip(0..)
            .map(|(h, i)| h + (octave + i32::from(i < rest)) * n)
            .collect();
        Chord::new(scale.clone(), self.root, tones)
    }
}

impl RootedTones for Chord {
    fn notenums(&self) -> Vec<Notenum> {
        self.tones
            .iter()
            .map(|h| self.scale.get(self.root + h))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub enum Resolution {
    #[serde(rename = "Perfect authentic")]
    Perfect,
    #[serde(rename = "Imperfect authentic")]
    Imperfect,
    Deceptive,
    Plagal,
    Half,
}

impl Resolution {
    pub const ALL: [Resolution; 5] = [
        Self::Perfect,
        Self::Imperfect,
        Self::Deceptive,
        Self::Plagal,
        Self::Half,
    ];

    /// The name used in the parameters and the fixtures.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Perfect => "Perfect authentic",
            Self::Imperfect => "Imperfect authentic",
            Self::Deceptive => "Deceptive",
            Self::Plagal => "Plagal",
            Self::Half => "Half",
        }
    }

    /// The cadences at the end of a tune (`CadencePostfix` in TypeScript).
    pub fn postfix(self) -> &'static [Cadence] {
        match self {
            Self::Perfect | Self::Imperfect | Self::Deceptive => &[Cadence::D, Cadence::T],
            Self::Plagal => &[Cadence::S, Cadence::T],
            Self::Half => &[Cadence::D],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub enum Cadence {
    #[serde(rename = "Tonic")]
    T,
    #[serde(rename = "Dominant")]
    D,
    #[serde(rename = "Subdominant")]
    S,
}

impl Cadence {
    /// The name used in the parameters and the fixtures.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::T => "Tonic",
            Self::D => "Dominant",
            Self::S => "Subdominant",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    pub pitch: Notenum,
    pub duration: f64,
    pub is_note_on: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rhythm {
    pub is_note_on: bool,
    pub t: f64,
    pub duration: f64,
}

/// A node of the rhythm tree (`NoteTreeNode` in TypeScript). The leaves are the notes.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteNode {
    pub rhythm: Rhythm,
    pub pitch: Option<Notenum>,
    pub children: Vec<NoteNode>,
}

impl NoteNode {
    pub fn new(rhythm: Rhythm) -> Self {
        Self {
            rhythm,
            pitch: None,
            children: Vec::new(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// All nodes in pre-order (`flat` in TypeScript).
    pub fn flat(&self) -> Vec<&NoteNode> {
        let mut res = Vec::new();
        self.visit(&mut |n| res.push(n));
        res
    }

    fn visit<'a>(&'a self, f: &mut impl FnMut(&'a NoteNode)) {
        f(self);
        for c in &self.children {
            c.visit(f);
        }
    }

    /// The leaves, depth first from the left.
    pub fn leaves(&self) -> Vec<&NoteNode> {
        let mut res = Vec::new();
        self.visit(&mut |n| {
            if n.is_leaf() {
                res.push(n);
            }
        });
        res
    }

    /// The leaves in the same order as [`NoteNode::leaves`].
    pub fn leaves_mut(&mut self) -> Vec<&mut NoteNode> {
        fn collect<'a>(n: &'a mut NoteNode, res: &mut Vec<&'a mut NoteNode>) {
            if n.is_leaf() {
                res.push(n);
            } else {
                for c in &mut n.children {
                    collect(c, res);
                }
            }
        }
        let mut res = Vec::new();
        collect(self, &mut res);
        res
    }

    /// The rhythms of the leaves.
    pub fn rhythms(&self) -> Vec<Rhythm> {
        self.leaves().into_iter().map(|n| n.rhythm).collect()
    }

    /// `None` if the pitch is not set (the TypeScript version throws).
    pub fn to_timeline_item(&self) -> Option<TimelineItem<Note>> {
        let pitch = self.pitch?;
        Some(TimelineItem::new(
            self.rhythm.t,
            Note {
                pitch,
                duration: self.rhythm.duration,
                is_note_on: self.rhythm.is_note_on,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from src/core/tune.test.ts.

    #[test]
    fn chord_rot() {
        let scale = Scale::major(69);
        let org = Chord::new(scale.clone(), 4, vec![0, 2, 4]);
        assert_eq!(org.notenums(), [76, 80, 83]);
        let cases: [(i32, [Notenum; 3]); 12] = [
            (1, [88, 80, 83]),
            (2, [88, 92, 83]),
            (3, [88, 92, 95]),
            (4, [100, 92, 95]),
            (5, [100, 104, 95]),
            (6, [100, 104, 107]),
            (-1, [76, 80, 71]),
            (-2, [76, 68, 71]),
            (-3, [64, 68, 71]),
            (-4, [64, 68, 59]),
            (-5, [64, 56, 59]),
            (-6, [52, 56, 59]),
        ];
        for (times, expected) in cases {
            assert_eq!(org.rot(&scale, times).notenums(), expected, "rot {times}");
        }
    }

    #[test]
    fn scale_get_wraps_degrees_into_octaves() {
        let scale = Scale::major(60);
        let cases = [
            (0, 60),
            (1, 62),
            (6, 71),
            (7, 72),
            (8, 74),
            (14, 84),
            (-1, 59),
            (-7, 48),
            (-8, 47),
        ];
        for (d, expected) in cases {
            assert_eq!(scale.get(d), expected, "degree {d}");
        }
    }

    #[test]
    fn scale_minor() {
        assert_eq!(Scale::minor(57).notenums(), [57, 59, 60, 62, 64, 65, 67]);
    }

    #[test]
    fn scale_includes_and_is_root_ignore_the_octave() {
        let scale = Scale::major(60);
        let notes = [48, 60, 61, 71, 72, 73, 83];
        assert_eq!(
            notes.map(|n| scale.includes(n)),
            [true, true, false, true, true, false, true]
        );
        assert_eq!(
            notes.map(|n| scale.is_root(n)),
            [true, true, false, false, true, false, false]
        );
    }

    #[test]
    fn scale_center_is_the_mean() {
        assert_eq!(Scale::major(60).center(), 458.0 / 7.0);
    }

    #[test]
    fn chord_notenums_take_the_degrees_on_the_scale() {
        let scale = Scale::major(60);
        let cases: [(Degree, &[Degree], &[Notenum]); 5] = [
            (0, &[0, 2, 4], &[60, 64, 67]),
            (4, &[0, 2, 4, 6], &[67, 71, 74, 77]),
            (5, &[0, 2, 4], &[69, 72, 76]),
            (6, &[0, 4], &[71, 77]),
            (-1, &[0, 2, 4], &[59, 62, 65]),
        ];
        for (root, tones, expected) in cases {
            let chord = Chord::new(scale.clone(), root, tones.to_vec());
            assert_eq!(chord.notenums(), expected, "root {root}");
        }
    }

    #[test]
    fn chord_includes_is_root_and_center() {
        let chord = Chord::new(Scale::major(60), 5, vec![0, 2, 4]); // A minor: 69, 72, 76
        assert_eq!(
            [57, 60, 64, 65].map(|n| chord.includes(n)),
            [true, true, true, false]
        );
        assert_eq!([57, 81, 60].map(|n| chord.is_root(n)), [true, true, false]);
        assert_eq!(chord.center(), 217.0 / 3.0);
    }

    #[test]
    fn tune_merge_appends_the_source_shifted_by_the_length() {
        fn tl<T>(items: Vec<(f64, T)>) -> Timeline<T> {
            Timeline::from_items(items.into_iter().map(|(t, v)| TimelineItem::new(t, v)))
        }
        let scale = Scale::major(60);
        let note = |pitch, duration, is_note_on| Note {
            pitch,
            duration,
            is_note_on,
        };
        let a = Tune {
            length: 4.0,
            scale: scale.clone(),
            cadence: tl(vec![(0.0, Cadence::T), (2.0, Cadence::D)]),
            chord: tl(vec![(0.0, Chord::new(scale.clone(), 0, vec![0, 2, 4]))]),
            notes: tl(vec![(0.0, note(60, 4.0, true))]),
            ..Tune::default()
        };
        let b = Tune {
            length: 2.0,
            cadence: tl(vec![(0.0, Cadence::S)]),
            chord: tl(vec![(0.0, Chord::new(scale.clone(), 3, vec![0, 2, 4]))]),
            notes: tl(vec![(1.0, note(65, 1.0, false))]),
            resolution: tl(vec![(0.0, Resolution::Half)]),
            ..Tune::default()
        };

        let m = a.merge(&b);
        assert_eq!(m.length, 6.0);
        assert_eq!(m.scale, a.scale);
        assert_eq!(
            m.cadence,
            tl(vec![
                (0.0, Cadence::T),
                (2.0, Cadence::D),
                (4.0, Cadence::S)
            ])
        );
        let roots: Vec<_> = m.chord.list().iter().map(|e| (e.t, e.value.root)).collect();
        assert_eq!(roots, [(0.0, 0), (4.0, 3)]);
        let pitches: Vec<_> = m
            .notes
            .list()
            .iter()
            .map(|e| (e.t, e.value.pitch))
            .collect();
        assert_eq!(pitches, [(0.0, 60), (5.0, 65)]);
        // The resolution is not merged.
        assert!(m.resolution.is_empty());
    }

    #[test]
    fn resolution_postfix() {
        let names: Vec<_> = Resolution::ALL
            .iter()
            .map(|r| (r.as_str(), r.postfix()))
            .collect();
        use Cadence::*;
        assert_eq!(
            names,
            [
                ("Perfect authentic", &[D, T][..]),
                ("Imperfect authentic", &[D, T][..]),
                ("Deceptive", &[D, T][..]),
                ("Plagal", &[S, T][..]),
                ("Half", &[D][..]),
            ]
        );
    }

    // The TreeNode tests of src/core/util.test.ts, on NoteNode.
    //      r
    //    /   \
    //   a     b
    //  / \
    // c   d
    fn tree() -> NoteNode {
        let node = |t: f64, children: Vec<NoteNode>| NoteNode {
            children,
            ..NoteNode::new(Rhythm {
                is_note_on: true,
                t,
                duration: 1.0,
            })
        };
        // The times are names: r = 0, a = 1, b = 2, c = 3, d = 4.
        node(
            0.0,
            vec![
                node(1.0, vec![node(3.0, vec![]), node(4.0, vec![])]),
                node(2.0, vec![]),
            ],
        )
    }

    fn names(nodes: &[&NoteNode]) -> Vec<f64> {
        nodes.iter().map(|n| n.rhythm.t).collect()
    }

    #[test]
    fn note_node_flat_is_pre_order() {
        assert_eq!(names(&tree().flat()), [0.0, 1.0, 3.0, 4.0, 2.0]);
    }

    #[test]
    fn note_node_leaves() {
        let mut tree = tree();
        assert_eq!(names(&tree.leaves()), [3.0, 4.0, 2.0]);
        let single = NoteNode::new(tree.rhythm);
        assert_eq!(names(&single.leaves()), [0.0]);

        for (i, leaf) in tree.leaves_mut().into_iter().enumerate() {
            leaf.pitch = Some(60 + i as Notenum);
        }
        let pitches: Vec<_> = tree.leaves().iter().map(|n| n.pitch).collect();
        assert_eq!(pitches, [Some(60), Some(61), Some(62)]);
        let ts: Vec<_> = tree.rhythms().iter().map(|r| r.t).collect();
        assert_eq!(ts, [3.0, 4.0, 2.0]);
    }

    #[test]
    fn note_node_is_leaf() {
        let tree = tree();
        assert_eq!([tree.is_leaf(), tree.children[1].is_leaf()], [false, true]);
    }

    #[test]
    fn note_node_to_timeline_item() {
        let mut n = NoteNode::new(Rhythm {
            is_note_on: false,
            t: 1.5,
            duration: 0.5,
        });
        assert_eq!(n.to_timeline_item(), None);
        n.pitch = Some(64);
        let expected = Note {
            pitch: 64,
            duration: 0.5,
            is_note_on: false,
        };
        assert_eq!(n.to_timeline_item(), Some(TimelineItem::new(1.5, expected)));
    }
}
