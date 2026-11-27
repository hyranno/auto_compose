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

/// Whether two notes are in the same pitch class.
fn same_pitch_class(a: Notenum, b: Notenum) -> bool {
    a.rem_euclid(12) == b.rem_euclid(12)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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

    pub fn notenums(&self) -> impl Iterator<Item = Notenum> + '_ {
        self.tones.iter().map(|h| self.key + h)
    }

    /// Whether `note` is on the scale, ignoring the octave.
    pub fn includes(&self, note: Notenum) -> bool {
        self.notenums().any(|v| same_pitch_class(v, note))
    }

    /// The notenum of a degree, wrapping into octaves.
    ///
    /// Panics if there are no tones.
    pub fn get(&self, degree: Degree) -> Notenum {
        let len = self.tones.len() as Degree;
        self.key + self.tones[degree.rem_euclid(len) as usize] + 12 * degree.div_euclid(len)
    }
}

/// A chord on the scale of the tune.
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    /// Degree of the root on the scale.
    pub root: Degree,
    /// Degrees from the root on the scale.
    pub tones: Vec<Degree>,
}

impl Chord {
    pub fn new(root: Degree, tones: Vec<Degree>) -> Self {
        Self { root, tones }
    }

    /// The notenums of the tones on `scale`.
    pub fn notenums<'a>(&'a self, scale: &'a Scale) -> impl Iterator<Item = Notenum> + 'a {
        self.tones.iter().map(move |h| scale.get(self.root + h))
    }

    /// Whether `note` is one of the tones on `scale`, ignoring the octave.
    pub fn includes(&self, scale: &Scale, note: Notenum) -> bool {
        self.notenums(scale).any(|v| same_pitch_class(v, note))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
        let notenums: Vec<_> = Scale::minor(57).notenums().collect();
        assert_eq!(notenums, [57, 59, 60, 62, 64, 65, 67]);
    }

    #[test]
    fn scale_includes_ignores_the_octave() {
        let scale = Scale::major(60);
        let notes = [-1, 0, 1, 48, 60, 61, 71, 72, 73, 83];
        assert_eq!(
            notes.map(|n| scale.includes(n)),
            [
                true, true, false, true, true, false, true, true, false, true
            ]
        );
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
            let chord = Chord::new(root, tones.to_vec());
            let notenums: Vec<_> = chord.notenums(&scale).collect();
            assert_eq!(notenums, expected, "root {root}");
        }
    }

    #[test]
    fn chord_includes_ignores_the_octave() {
        let scale = Scale::major(60);
        let chord = Chord::new(5, vec![0, 2, 4]); // A minor: 69, 72, 76
        assert_eq!(
            [-3, 57, 60, 64, 65].map(|n| chord.includes(&scale, n)),
            [true, true, true, true, false]
        );
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
            chord: tl(vec![(0.0, Chord::new(0, vec![0, 2, 4]))]),
            notes: tl(vec![(0.0, note(60, 4.0, true))]),
            ..Tune::default()
        };
        let b = Tune {
            length: 2.0,
            cadence: tl(vec![(0.0, Cadence::S)]),
            chord: tl(vec![(0.0, Chord::new(3, vec![0, 2, 4]))]),
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
