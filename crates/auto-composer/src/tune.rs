//! Tunes, scales, chords and the note tree.

use serde::{Deserialize, Serialize};

use crate::util::{Timeline, TimelineItem};

/// MIDI note number on 12 equal temperament.
#[cfg_attr(feature = "tsify", tsify::declare)]
pub type Notenum = i32;
/// Index on a scale or a chord.
#[cfg_attr(feature = "tsify", tsify::declare)]
pub type Degree = i32;

/// The length of a tune.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TimeMeasure {
    pub beats_per_bar: u32,
    pub bars: u32,
}

impl TimeMeasure {
    pub fn new(beats_per_bar: u32, bars: u32) -> Self {
        Self {
            beats_per_bar,
            bars,
        }
    }

    /// The number of beats.
    pub fn beats(self) -> u32 {
        self.beats_per_bar * self.bars
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tune {
    /// In beats. A merged tune is longer than its time measure.
    pub length: f64,
    pub time_measure: TimeMeasure,
    pub scale: Scale,
    /// The cadence at the end of the tune.
    pub resolution: Resolution,
    pub cadence: Timeline<Cadence>,
    pub chord: Timeline<Chord>,
    pub notes: Timeline<Note>,
}

impl Tune {
    /// A tune with no cadences, chords or notes yet.
    pub fn new(time_measure: TimeMeasure, scale: Scale, resolution: Resolution) -> Self {
        Self {
            length: f64::from(time_measure.beats()),
            time_measure,
            scale,
            resolution,
            cadence: Timeline::new(),
            chord: Timeline::new(),
            notes: Timeline::new(),
        }
    }

    /// `self` followed by `src`. The time measure and the scale are taken from `self`,
    /// and the resolution from `src`, which ends the merged tune.
    pub fn merge(&self, src: &Tune) -> Tune {
        let mut dest = Tune {
            length: self.length + src.length,
            resolution: src.resolution,
            ..Tune::new(self.time_measure, self.scale.clone(), src.resolution)
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
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    PerfectAuthentic,
    ImperfectAuthentic,
    Deceptive,
    Plagal,
    Half,
}

impl Resolution {
    pub const ALL: [Resolution; 5] = [
        Self::PerfectAuthentic,
        Self::ImperfectAuthentic,
        Self::Deceptive,
        Self::Plagal,
        Self::Half,
    ];

    /// The cadences at the end of a tune.
    pub fn postfix(self) -> &'static [Cadence] {
        use Cadence::*;
        match self {
            Self::PerfectAuthentic | Self::ImperfectAuthentic | Self::Deceptive => {
                &[Dominant, Tonic]
            }
            Self::Plagal => &[Subdominant, Tonic],
            Self::Half => &[Dominant],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Cadence {
    Tonic,
    Dominant,
    Subdominant,
}

impl Cadence {
    pub const ALL: [Cadence; 3] = [Self::Tonic, Self::Dominant, Self::Subdominant];
}

/// A value for each cadence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct PerCadence<T> {
    pub tonic: T,
    pub dominant: T,
    pub subdominant: T,
}

impl<T> PerCadence<T> {
    pub fn get(&self, cadence: Cadence) -> &T {
        match cadence {
            Cadence::Tonic => &self.tonic,
            Cadence::Dominant => &self.dominant,
            Cadence::Subdominant => &self.subdominant,
        }
    }
}

/// A note or a rest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    /// `None` for a rest.
    pub pitch: Option<Notenum>,
    pub duration: f64,
}

impl Note {
    pub fn is_note_on(&self) -> bool {
        self.pitch.is_some()
    }
}

/// The span of a node of the rhythm tree, in beats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rhythm {
    pub t: f64,
    pub duration: f64,
}

impl Rhythm {
    pub fn new(t: f64, duration: f64) -> Self {
        Self { t, duration }
    }
}

/// A node of the rhythm tree. The leaves are the notes and the rests, and carry an `L`:
/// whether the note is on in the tree of the rhythm ([`RhythmNode`]), and the pitch, `None` for
/// a rest, once the pitches are drawn ([`NoteNode`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Node<L> {
    pub rhythm: Rhythm,
    pub kind: NodeKind<L>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind<L> {
    Leaf(L),
    Branch(Vec<Node<L>>),
}

/// A rhythm tree whose leaves tell whether the note is on.
pub type RhythmNode = Node<bool>;
/// A rhythm tree with the pitches of its leaves, `None` for the rests.
pub type NoteNode = Node<Option<Notenum>>;

impl<L> Node<L> {
    pub fn leaf(rhythm: Rhythm, value: L) -> Self {
        Self {
            rhythm,
            kind: NodeKind::Leaf(value),
        }
    }

    pub fn branch(rhythm: Rhythm, children: Vec<Node<L>>) -> Self {
        Self {
            rhythm,
            kind: NodeKind::Branch(children),
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self.kind, NodeKind::Leaf(_))
    }

    /// Empty for a leaf.
    pub fn children(&self) -> &[Node<L>] {
        match &self.kind {
            NodeKind::Leaf(_) => &[],
            NodeKind::Branch(children) => children,
        }
    }

    /// Empty for a leaf.
    pub fn children_mut(&mut self) -> &mut [Node<L>] {
        match &mut self.kind {
            NodeKind::Leaf(_) => &mut [],
            NodeKind::Branch(children) => children,
        }
    }

    /// The rhythms and the values of the leaves, depth first from the left.
    pub fn leaves(&self) -> Vec<(Rhythm, &L)> {
        fn collect<'a, L>(n: &'a Node<L>, res: &mut Vec<(Rhythm, &'a L)>) {
            match &n.kind {
                NodeKind::Leaf(value) => res.push((n.rhythm, value)),
                NodeKind::Branch(children) => children.iter().for_each(|c| collect(c, res)),
            }
        }
        let mut res = Vec::new();
        collect(self, &mut res);
        res
    }

    /// The leaves in the same order as [`Node::leaves`].
    pub fn leaves_mut(&mut self) -> Vec<&mut Node<L>> {
        fn collect<'a, L>(n: &'a mut Node<L>, res: &mut Vec<&'a mut Node<L>>) {
            if n.is_leaf() {
                res.push(n);
            } else {
                n.children_mut().iter_mut().for_each(|c| collect(c, res));
            }
        }
        let mut res = Vec::new();
        collect(self, &mut res);
        res
    }

    /// The rhythms of the leaves.
    pub fn rhythms(&self) -> Vec<Rhythm> {
        self.leaves().into_iter().map(|(r, _)| r).collect()
    }

    /// The same tree with the values of the leaves replaced by `f` of their rhythms and values,
    /// called on the leaves in the order of [`Node::leaves`].
    pub fn try_map_leaves<M, E>(
        self,
        f: &mut impl FnMut(Rhythm, L) -> std::result::Result<M, E>,
    ) -> std::result::Result<Node<M>, E> {
        let kind = match self.kind {
            NodeKind::Leaf(value) => NodeKind::Leaf(f(self.rhythm, value)?),
            NodeKind::Branch(children) => NodeKind::Branch(
                children
                    .into_iter()
                    .map(|c| c.try_map_leaves(f))
                    .collect::<std::result::Result<_, _>>()?,
            ),
        };
        Ok(Node {
            rhythm: self.rhythm,
            kind,
        })
    }
}

impl NoteNode {
    /// The pitches of the leaves, `None` for the rests.
    pub fn pitches(&self) -> Vec<Option<Notenum>> {
        self.leaves().into_iter().map(|(_, &p)| p).collect()
    }

    /// The leaves as a timeline.
    pub fn to_timeline(&self) -> Timeline<Note> {
        Timeline::from_items(self.leaves().into_iter().map(|(r, &pitch)| {
            TimelineItem::new(
                r.t,
                Note {
                    pitch,
                    duration: r.duration,
                },
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let note = |pitch, duration| Note {
            pitch: Some(pitch),
            duration,
        };
        use Cadence::*;
        let a = Tune {
            cadence: tl(vec![(0.0, Tonic), (2.0, Dominant)]),
            chord: tl(vec![(0.0, Chord::new(0, vec![0, 2, 4]))]),
            notes: tl(vec![(0.0, note(60, 4.0))]),
            ..Tune::new(
                TimeMeasure::new(4, 1),
                scale.clone(),
                Resolution::PerfectAuthentic,
            )
        };
        let b = Tune {
            cadence: tl(vec![(0.0, Subdominant)]),
            chord: tl(vec![(0.0, Chord::new(3, vec![0, 2, 4]))]),
            notes: tl(vec![(1.0, note(65, 1.0))]),
            ..Tune::new(TimeMeasure::new(2, 1), Scale::minor(60), Resolution::Half)
        };

        let m = a.merge(&b);
        assert_eq!(m.length, 6.0);
        assert_eq!(m.scale, a.scale);
        assert_eq!(
            m.cadence,
            tl(vec![(0.0, Tonic), (2.0, Dominant), (4.0, Subdominant)])
        );
        let roots: Vec<_> = m.chord.list().iter().map(|e| (e.t, e.value.root)).collect();
        assert_eq!(roots, [(0.0, 0), (4.0, 3)]);
        let pitches: Vec<_> = m
            .notes
            .list()
            .iter()
            .map(|e| (e.t, e.value.pitch.unwrap()))
            .collect();
        assert_eq!(pitches, [(0.0, 60), (5.0, 65)]);
        assert_eq!(m.time_measure, a.time_measure);
        assert_eq!(m.resolution, Resolution::Half);
    }

    #[test]
    fn resolution_postfix() {
        let names: Vec<_> = Resolution::ALL
            .iter()
            .map(|r| (serde_json::to_string(r).unwrap(), r.postfix()))
            .collect();
        use Cadence::*;
        assert_eq!(
            names,
            [
                (r#""perfect_authentic""#.into(), &[Dominant, Tonic][..]),
                (r#""imperfect_authentic""#.into(), &[Dominant, Tonic][..]),
                (r#""deceptive""#.into(), &[Dominant, Tonic][..]),
                (r#""plagal""#.into(), &[Subdominant, Tonic][..]),
                (r#""half""#.into(), &[Dominant][..]),
            ]
        );
    }

    // A tree of nodes named by their times:
    //      r
    //    /   \
    //   a     b
    //  / \
    // c   d
    // The leaf d is a rest.
    fn tree() -> RhythmNode {
        let rhythm = |t: f64| Rhythm::new(t, 1.0);
        // The times are names: r = 0, a = 1, b = 2, c = 3, d = 4.
        Node::branch(
            rhythm(0.0),
            vec![
                Node::branch(
                    rhythm(1.0),
                    vec![
                        Node::leaf(rhythm(3.0), true),
                        Node::leaf(rhythm(4.0), false),
                    ],
                ),
                Node::leaf(rhythm(2.0), true),
            ],
        )
    }

    #[test]
    fn node_leaves() {
        let tree = tree();
        let ts: Vec<_> = tree.rhythms().iter().map(|r| r.t).collect();
        assert_eq!(ts, [3.0, 4.0, 2.0]);
        let on: Vec<_> = tree.leaves().iter().map(|&(_, &on)| on).collect();
        assert_eq!(on, [true, false, true]);
        let single = Node::leaf(tree.rhythm, true);
        assert_eq!(single.rhythms(), [tree.rhythm]);

        let mut next = 60;
        let mut tree = tree
            .try_map_leaves(&mut |_, on| {
                next += 1;
                Ok::<_, ()>(on.then_some(next - 1))
            })
            .unwrap();
        assert_eq!(tree.pitches(), [Some(60), None, Some(62)]);
        for leaf in tree.leaves_mut() {
            leaf.kind = NodeKind::Leaf(None);
        }
        assert_eq!(tree.pitches(), [None; 3]);
        assert_eq!(tree.try_map_leaves(&mut |_, _| Err::<(), _>("e")), Err("e"));
    }

    #[test]
    fn node_is_leaf() {
        let tree = tree();
        assert_eq!(
            [tree.is_leaf(), tree.children()[1].is_leaf()],
            [false, true]
        );
        assert!(tree.children()[1].children().is_empty());
    }

    #[test]
    fn note_node_to_timeline() {
        let tree = Node::branch(
            Rhythm::new(1.0, 1.0),
            vec![
                Node::leaf(Rhythm::new(1.0, 0.5), Some(64)),
                Node::leaf(Rhythm::new(1.5, 0.5), None),
            ],
        );
        let note = |pitch| Note {
            pitch,
            duration: 0.5,
        };
        assert_eq!(
            tree.to_timeline(),
            Timeline::from_items([
                TimelineItem::new(1.0, note(Some(64))),
                TimelineItem::new(1.5, note(None))
            ])
        );
        assert!(note(Some(64)).is_note_on());
        assert!(!note(None).is_note_on());
    }
}
