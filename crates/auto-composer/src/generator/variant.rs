//! A variant of a tune: one layer regenerated within a timespan (`src/core/variant.ts`).
//!
//! The variant draws from its own random number source (label `"variant"`): first the timespan,
//! then the layer, then everything the variant of that layer needs. The original tune is
//! generated in between with the seeds of the base parameters, and the layers below the chosen
//! one are generated on top of the variant, as in TypeScript.
//!
//! Known bugs of the TypeScript version are reproduced on purpose (see the notes on each step).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::markov::{Condition, MarkovChain};
use crate::random::{RandomSeed, Rng, RngFactory};
use crate::tune::{Cadence, Chord, Note, NoteNode, Notenum, Resolution, Tune};
use crate::util::{Timeline, TimelineItem, WeightedItem, weighted_pick};

use super::{cadence, chord, note, pitch_weight, rhythm, tune as tune_generator};

/// The layer to vary. It is a number (0 to 3) in the JSON, as in TypeScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum Layer {
    Cadence = 0,
    Chord = 1,
    Pitch = 2,
    Rhythm = 3,
}

impl Layer {
    pub const ALL: [Layer; 4] = [Self::Cadence, Self::Chord, Self::Pitch, Self::Rhythm];
}

impl TryFrom<u8> for Layer {
    type Error = String;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(v))
            .copied()
            .ok_or_else(|| format!("invalid layer {v}, expected 0 to 3"))
    }
}

impl From<Layer> for u8 {
    fn from(layer: Layer) -> u8 {
        layer as u8
    }
}

/// The parameters of a variant, read from the same JSON as the TypeScript version
/// (`resources/VariantGenParams_default.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub seed: RandomSeed,
    /// Replaces the resolution of the base parameters.
    pub resolution: Resolution,
    /// The weights of the timespans `[start, end]` in beats. Only the items strictly inside are
    /// changed.
    pub timespan: Vec<WeightedItem<[f64; 2]>>,
    pub layer: Vec<WeightedItem<Layer>>,
}

/// Generates the tune of `base` (with the resolution of `params`) and varies one layer of it.
pub fn generate(
    params: &Parameters,
    base: &tune_generator::Parameters,
    factory: &dyn RngFactory,
) -> Result<Tune> {
    let mut rng = factory.create(params.seed, "variant");
    let timespan = *weighted_pick(&params.timespan, &mut rng).ok_or(Error::NoItems)?;
    let layer = *weighted_pick(&params.layer, &mut rng).ok_or(Error::NoItems)?;
    let tune_params = tune_generator::Parameters {
        resolution: params.resolution,
        ..base.clone()
    };
    let mut tune = tune_generator::empty(&tune_params)?;
    tune.cadence = cadence::generate(&tune, &tune_params.cadence, factory)?;
    if layer == Layer::Cadence {
        tune.cadence = vary_cadence(&tune, timespan, &mut rng, &tune_params.cadence)?;
    }
    tune.chord = chord::generate(&tune, &tune_params.chord, factory)?;
    if layer == Layer::Chord {
        tune.chord = vary_chord(&tune, timespan, &mut rng, &tune_params.chord)?;
    }
    tune.notes = note::generate(&tune, &tune_params.note, factory)?;
    match layer {
        Layer::Pitch => {
            tune.notes = vary_pitch(&tune, timespan, &mut rng, &tune_params.note, factory)?;
        }
        Layer::Rhythm => {
            tune.notes = vary_rhythm(&tune, timespan, &mut rng, &tune_params.note, factory)?;
        }
        Layer::Cadence | Layer::Chord => {}
    }
    Ok(tune)
}

/// The indices of `timeline` at both ends of `timespan` ([`Timeline::get_index`]).
fn span_indices<T>(timeline: &Timeline<T>, timespan: [f64; 2]) -> [usize; 2] {
    timespan.map(|t| timeline.get_index(t))
}

/// Redraws the cadences strictly inside the timespan by the Markov chain, starting from the
/// cadence at the start and conditioned on reaching the cadence at the end.
pub fn vary_cadence(
    tune: &Tune,
    timespan: [f64; 2],
    rng: &mut impl Rng,
    params: &cadence::Parameters,
) -> Result<Timeline<Cadence>> {
    let [start, end] = span_indices(&tune.cadence, timespan);
    let list = tune.cadence.list();
    if end <= start + 1 {
        return Ok(tune.cadence.clone());
    }
    let at = |i: usize| {
        list.get(i).map(|e| e.value).ok_or(Error::NoItemAt {
            timeline: "cadence",
        })
    };
    let mut chain = MarkovChain::new(at(start)?, &params.probabilities)?;
    let goal = at(end)?;
    let items = list
        .iter()
        .enumerate()
        .map(|(i, e)| {
            if !(start < i && i < end) {
                return Ok(e.clone());
            }
            let value = chain.next_conditional(rng, &Condition::new(end - i, goal), false)?;
            Ok(TimelineItem::new(e.t, value))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(items))
}

/// Redraws the root and the tones of the chords strictly inside the timespan.
///
/// The indices are those of the cadences (bug 5 of the notes); it works because the cadences
/// and the chords have the same times.
pub fn vary_chord(
    tune: &Tune,
    timespan: [f64; 2],
    rng: &mut impl Rng,
    params: &chord::Parameters,
) -> Result<Timeline<Chord>> {
    let [start, end] = span_indices(&tune.cadence, timespan);
    let items = tune
        .chord
        .list()
        .iter()
        .enumerate()
        .map(|(i, e)| {
            if !(start < i && i < end) {
                return Ok(e.clone());
            }
            let cadence = tune
                .cadence
                .get(e.t)
                .ok_or(Error::NoItemAt {
                    timeline: "cadence",
                })?
                .value;
            let items_root = chord::root_items(&params.root, cadence)?;
            let root = *weighted_pick(items_root, rng).ok_or(Error::NoItems)?;
            let tones = weighted_pick(&params.tone.rand_tones.items, rng).ok_or(Error::NoItems)?;
            Ok(TimelineItem::new(
                e.t,
                Chord::new(tune.scale.clone(), root, tones.clone()),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(items))
}

/// Draws the pitches of `dest` so that they lead from `start` to `end`.
///
/// The pitches move by a Markov chain on `key ± (ceil(absPitchFactor.edge1) - 1)` whose
/// transition weights are [`pitch_weight::calc_time_homo_weight`] (made into probabilities by
/// [`MarkovChain::new`]).
///
/// Each note is drawn from the distribution of the chain one step after the previous pitch,
/// conditioned on reaching `end` one step after the last note, and weighted further by the
/// chord and the duration of the note. `start` and `end` outside the candidates are moved to
/// the nearest candidate for the chain. When `end` cannot be reached in the remaining steps,
/// the condition on `end` is dropped for that note.
pub fn interpolate(
    tune: &Tune,
    start: Notenum,
    end: Notenum,
    rng: &mut impl Rng,
    dest: &mut [&mut NoteNode],
    params: &pitch_weight::Parameters,
) -> Result<()> {
    let max_length_to_key = params.abs_pitch_factor.edge1.ceil() as Notenum;
    let key = tune.scale.key;
    let candidates: Vec<Notenum> = (key - max_length_to_key + 1..key + max_length_to_key).collect();
    if candidates.is_empty() {
        // TypeScript fails to build the Markov chain of no states.
        return Err(Error::InvalidParameter("pitchWeight.absPitchFactor.edge1"));
    }
    let transition: Vec<(Notenum, Vec<WeightedItem<Notenum>>)> = candidates
        .iter()
        .map(|&p| {
            let row = candidates
                .iter()
                .map(|&c| {
                    WeightedItem::new(pitch_weight::calc_time_homo_weight(tune, p, c, params), c)
                })
                .collect();
            (p, row)
        })
        .collect();
    let chain = MarkovChain::new(0, &transition)?;
    let nearest = |p: Notenum| p.clamp(candidates[0], candidates[candidates.len() - 1]);
    let end = nearest(end);
    let mut prev = nearest(start);
    let len = dest.len();
    for (i, note) in dest.iter_mut().enumerate() {
        // From the note `i`, `len - 1 - i` notes and then `end` follow.
        let mut dist = chain.calc_distribution_conditional(
            &Condition::new(1, prev),
            &Condition::new(len - i, end),
        )?;
        if !is_distribution(&dist) {
            dist = chain.calc_distribution(&chain.state_to_vector(&prev), false);
        }
        let items = chain
            .vector_to_weighted_items(&dist)
            .into_iter()
            .map(|item| {
                let candidate = Note {
                    pitch: item.value,
                    is_note_on: note.rhythm.is_note_on,
                    duration: note.rhythm.duration,
                };
                let weight = pitch_weight::calc_with_time_homo_weight(
                    tune,
                    item.weight,
                    &candidate,
                    note.rhythm.t,
                    params,
                )?;
                Ok(WeightedItem::new(weight, item.value))
            })
            .collect::<Result<Vec<_>>>()?;
        let pitch = *weighted_pick(&items, rng).ok_or(Error::NoItems)?;
        note.pitch = Some(pitch);
        prev = pitch;
    }
    Ok(())
}

/// Whether the weights can be drawn from: finite, non-negative and not all 0.
fn is_distribution(v: &[f64]) -> bool {
    v.iter().all(|w| (0.0..f64::INFINITY).contains(w)) && v.iter().any(|&w| w > 0.0)
}

/// Regenerates the notes of the base parameters and redraws the pitches of the notes strictly
/// inside the timespan (the indices are taken from `tune.notes`).
pub fn vary_pitch(
    tune: &Tune,
    timespan: [f64; 2],
    rng: &mut impl Rng,
    params: &note::Parameters,
    factory: &dyn RngFactory,
) -> Result<Timeline<Note>> {
    let [start, end] = span_indices(&tune.notes, timespan);
    let mut tree = note::generate_tree(tune, params, factory)?;
    let mut leaves = tree.leaves_mut();
    let pitch_at = |leaves: &[&mut NoteNode], i: usize| {
        leaves
            .get(i)
            .and_then(|l| l.pitch)
            .ok_or(Error::MissingPitch)
    };
    let start_pitch = pitch_at(&leaves, start)?;
    let end_pitch = pitch_at(&leaves, end)?;
    // `slice(start + 1, end)`: empty when the range is reversed.
    let dest = leaves
        .get_mut(start + 1..end.max(start + 1))
        .unwrap_or_default();
    interpolate(
        tune,
        start_pitch,
        end_pitch,
        rng,
        dest,
        &params.pitch_weight,
    )?;
    note::tree_to_timeline(&tree)
}

/// Regenerates the note tree of the base parameters, then splits, merges and toggles the notes
/// inside the timespan.
pub fn vary_rhythm(
    tune: &Tune,
    timespan: [f64; 2],
    rng: &mut impl Rng,
    params: &note::Parameters,
    factory: &dyn RngFactory,
) -> Result<Timeline<Note>> {
    let mut tree = note::generate_tree(tune, params, factory)?;
    split_rhythm(tune, &mut tree, timespan, rng, params)?;
    merge_rhythm(tune, &mut tree, timespan, rng, params)?;
    toggle_note_on(&mut tree, timespan, rng, &params.rhythm);
    note::tree_to_timeline(&tree)
}

fn inside(timespan: [f64; 2], t: f64) -> bool {
    timespan[0] < t && t < timespan[1]
}

const NUM_CHILDREN: usize = 2;

/// The pitch of the leaf `neighbour`, or of the leaf `own` when there is no such leaf
/// (`neighbour` is out of range at either end of the tune).
fn neighbour_pitch(
    pitches: &[Option<Notenum>],
    neighbour: Option<usize>,
    own: usize,
) -> Result<Notenum> {
    neighbour
        .and_then(|i| pitches.get(i))
        .unwrap_or(&pitches[own])
        .ok_or(Error::MissingPitch)
}

/// Splits each leaf inside the timespan into halves with the probability `pBranch` of its
/// duration, drawing once per leaf, and interpolates the pitches of the halves between the
/// neighbour leaves. The first and the last leaf use their own pitch on the side without a
/// neighbour.
pub fn split_rhythm(
    tune: &Tune,
    root: &mut NoteNode,
    timespan: [f64; 2],
    rng: &mut impl Rng,
    params: &note::Parameters,
) -> Result<()> {
    let mut leaves = root.leaves_mut();
    // Splitting a leaf does not change the pitches of the leaves.
    let pitches: Vec<Option<Notenum>> = leaves.iter().map(|l| l.pitch).collect();
    for (i, leaf) in leaves.iter_mut().enumerate() {
        let rhythm = leaf.rhythm;
        if !inside(timespan, rhythm.t) {
            continue;
        }
        let p_split = params.rhythm.p_branch.eval(rhythm.duration);
        if rng.random() < p_split {
            let sub_duration = rhythm.duration / NUM_CHILDREN as f64;
            leaf.children.extend(
                (0..NUM_CHILDREN)
                    .map(|k| rhythm::new_node(rhythm.t + k as f64 * sub_duration, sub_duration)),
            );
            let start = neighbour_pitch(&pitches, i.checked_sub(1), i)?;
            let end = neighbour_pitch(&pitches, Some(i + 1), i)?;
            let mut children: Vec<&mut NoteNode> = leaf.children.iter_mut().collect();
            interpolate(tune, start, end, rng, &mut children, &params.pitch_weight)?;
        }
    }
    Ok(())
}

/// The paths (indices of the children from `node`) of the nodes with `NUM_CHILDREN` children,
/// in pre-order.
fn branch_paths(node: &NoteNode, path: &mut Vec<usize>, res: &mut Vec<Vec<usize>>) {
    if node.children.len() == NUM_CHILDREN {
        res.push(path.clone());
    }
    for (i, child) in node.children.iter().enumerate() {
        path.push(i);
        branch_paths(child, path, res);
        path.pop();
    }
}

fn node_at<'a>(root: &'a NoteNode, path: &[usize]) -> Option<&'a NoteNode> {
    path.iter().try_fold(root, |n, &i| n.children.get(i))
}

fn node_at_mut<'a>(root: &'a mut NoteNode, path: &[usize]) -> Option<&'a mut NoteNode> {
    path.iter().try_fold(root, |n, &i| n.children.get_mut(i))
}

/// Merges each branch of two children inside the timespan into one note with the probability
/// `1 - pBranch` of its duration, drawing once per branch, and interpolates its pitch between
/// the leaves next to its children.
///
/// As in TypeScript, the branches and the leaves are listed before any merge. Reproduced bugs
/// (bug 4 of the notes):
/// - The leaves next to a child are found by the time of the child, which is the time of its
///   first leaf when the child has children.
/// - A branch under a merged branch is still visited and draws, although it is no longer in
///   the tree.
/// - The merged note keeps the `isNoteOn` of the branch.
pub fn merge_rhythm(
    tune: &Tune,
    root: &mut NoteNode,
    timespan: [f64; 2],
    rng: &mut impl Rng,
    params: &note::Parameters,
) -> Result<()> {
    let leaves: Vec<(f64, Option<Notenum>)> = root
        .leaves()
        .iter()
        .map(|l| (l.rhythm.t, l.pitch))
        .collect();
    let pitches: Vec<Option<Notenum>> = leaves.iter().map(|l| l.1).collect();
    let mut paths = Vec::new();
    branch_paths(root, &mut Vec::new(), &mut paths);
    // What each branch looks like before the merges; a merge changes only the merged branch.
    let targets: Vec<(Vec<usize>, NoteNode)> = paths
        .into_iter()
        .map(|path| {
            let node = node_at(root, &path).expect("a path just collected");
            (path, node.clone())
        })
        .collect();
    for (path, branch) in targets {
        if !inside(timespan, branch.rhythm.t) {
            continue;
        }
        let p_merge = 1.0 - params.rhythm.p_branch.eval(branch.rhythm.duration);
        if rng.random() < p_merge {
            let [i0, i1] = [0, 1].map(|k| {
                let t = branch.children[k].rhythm.t;
                leaves
                    .iter()
                    .position(|l| l.0 == t)
                    .ok_or(Error::NoNeighbour)
            });
            let (i0, i1) = (i0?, i1?);
            let start = neighbour_pitch(&pitches, i0.checked_sub(1), i0)?;
            let end = neighbour_pitch(&pitches, Some(i1 + 1), i1)?;
            let mut merged = NoteNode::new(branch.rhythm);
            merged.pitch = branch.pitch;
            interpolate(
                tune,
                start,
                end,
                rng,
                &mut [&mut merged],
                &params.pitch_weight,
            )?;
            // A branch under a merged one is gone from the tree; its result is discarded.
            if let Some(node) = node_at_mut(root, &path) {
                node.children.clear();
                node.pitch = merged.pitch;
            }
        }
    }
    Ok(())
}

/// Redraws whether each leaf inside the timespan is a note or a rest, with the probability
/// `pNoteOn` of its duration.
pub fn toggle_note_on(
    root: &mut NoteNode,
    timespan: [f64; 2],
    rng: &mut impl Rng,
    params: &rhythm::Parameters,
) {
    for leaf in root.leaves_mut() {
        if !inside(timespan, leaf.rhythm.t) {
            continue;
        }
        let p_note_on = params.p_note_on.eval(leaf.rhythm.duration);
        leaf.rhythm.is_note_on = rng.random() < p_note_on;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::generator::test_util::{ConstantRng, SEEDS, default_params, empty_tune};
    use crate::random::{ReliableRandom, ReliableRandomFactory, ScriptedRng};
    use crate::tune::{Rhythm, Scale};

    // Ported from src/core/generators.test.ts.

    const TIMESPANS: [[f64; 2]; 5] = [
        [6.0, 11.0],
        [2.0, 8.0],
        [9.0, 15.0],
        [0.0, 16.0],
        [3.0, 5.0],
    ];

    fn variant_params(
        layer: Layer,
        timespan: [f64; 2],
        (state, sequence): (i64, i64),
    ) -> Parameters {
        Parameters {
            seed: RandomSeed::new(state, sequence),
            resolution: Resolution::Half,
            timespan: vec![WeightedItem::new(1.0, timespan)],
            layer: vec![WeightedItem::new(1.0, layer)],
        }
    }

    fn base() -> tune_generator::Parameters {
        tune_generator::Parameters {
            resolution: Resolution::Half,
            ..default_params()
        }
    }

    fn outside<T: Clone>(timeline: &Timeline<T>, [t0, t1]: [f64; 2]) -> Vec<TimelineItem<T>> {
        timeline
            .list()
            .iter()
            .filter(|e| e.t <= t0 || t1 <= e.t)
            .cloned()
            .collect()
    }

    /// Gives `rng` to the variant and the usual Rngs to the generators of the original tune.
    struct VariantRngFactory<R> {
        rng: RefCell<R>,
    }

    struct Borrowed<'a, R>(&'a RefCell<R>);

    impl<R: Rng> Rng for Borrowed<'_, R> {
        fn random(&mut self) -> f64 {
            self.0.borrow_mut().random()
        }
    }

    impl<R: Rng> RngFactory for VariantRngFactory<R> {
        fn create(&self, seed: RandomSeed, label: &str) -> Box<dyn Rng + '_> {
            if label == "variant" {
                Box::new(Borrowed(&self.rng))
            } else {
                ReliableRandomFactory.create(seed, label)
            }
        }
    }

    #[test]
    fn reads_the_resource_file() {
        let p: Parameters = serde_json::from_str(include_str!(
            "../../../../resources/VariantGenParams_default.json"
        ))
        .unwrap();
        assert_eq!(p.resolution, Resolution::Perfect);
        assert_eq!(p.timespan, [WeightedItem::new(1.0, [6.0, 11.0])]);
        assert_eq!(p.layer, [WeightedItem::new(1.0, Layer::Rhythm)]);
        let json = serde_json::to_string(&p).unwrap();
        assert!(
            json.contains(r#""layer":[{"weight":1.0,"value":3}]"#),
            "{json}"
        );
        assert_eq!(serde_json::from_str::<Parameters>(&json).unwrap(), p);
        assert!(serde_json::from_str::<Layer>("4").is_err());
    }

    #[test]
    fn layers_0_to_2_keep_the_original_outside_the_timespan() {
        let original = tune_generator::generate(&base(), &ReliableRandomFactory).unwrap();
        for layer in [Layer::Cadence, Layer::Chord, Layer::Pitch] {
            for ts in TIMESPANS {
                for seed in SEEDS {
                    let params = variant_params(layer, ts, seed);
                    let tune = generate(&params, &base(), &ReliableRandomFactory).unwrap();
                    let what = format!("{layer:?} {ts:?} {seed:?}");
                    if layer == Layer::Cadence {
                        assert_eq!(
                            outside(&tune.cadence, ts),
                            outside(&original.cadence, ts),
                            "{what}"
                        );
                    } else {
                        assert_eq!(tune.cadence, original.cadence, "{what}");
                    }
                    if layer == Layer::Pitch {
                        assert_eq!(tune.chord, original.chord, "{what}");
                    } else {
                        assert_eq!(
                            outside(&tune.chord, ts),
                            outside(&original.chord, ts),
                            "{what}"
                        );
                    }
                    if layer == Layer::Pitch {
                        assert_eq!(
                            outside(&tune.notes, ts),
                            outside(&original.notes, ts),
                            "{what}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn layer_3_keeps_the_notes_before_the_timespan_and_the_upper_layers() {
        let original = tune_generator::generate(&base(), &ReliableRandomFactory).unwrap();
        for ts in TIMESPANS {
            for seed in SEEDS {
                let params = variant_params(Layer::Rhythm, ts, seed);
                let tune = generate(&params, &base(), &ReliableRandomFactory).unwrap();
                let before = |tl: &Timeline<Note>| -> Vec<TimelineItem<Note>> {
                    tl.list().iter().filter(|e| e.t <= ts[0]).cloned().collect()
                };
                let what = format!("{ts:?} {seed:?}");
                assert_eq!(before(&tune.notes), before(&original.notes), "{what}");
                assert_eq!(tune.cadence, original.cadence, "{what}");
                assert_eq!(tune.chord, original.chord, "{what}");
            }
        }
    }

    #[test]
    fn the_variant_rng_picks_the_timespan_first_and_then_the_layer() {
        let params = Parameters {
            seed: RandomSeed::new(0, 0),
            resolution: Resolution::Half,
            timespan: vec![
                WeightedItem::new(1.0, [2.0, 6.0]),
                WeightedItem::new(1.0, [6.0, 11.0]),
            ],
            layer: vec![
                WeightedItem::new(1.0, Layer::Cadence),
                WeightedItem::new(1.0, Layer::Chord),
            ],
        };
        let mut values = vec![0.75, 0.25];
        values.extend([0.5; 16]);
        let factory = VariantRngFactory {
            rng: RefCell::new(ScriptedRng::new(values)),
        };
        let tune = generate(&params, &base(), &factory).unwrap();
        let original = tune_generator::generate(&base(), &ReliableRandomFactory).unwrap();
        // Timespan [6, 11] and layer Cadence; the Cadence variant then draws for the slots
        // strictly inside: the cadences last 2 beats, so only the one at t = 8.
        let ts = [6.0, 11.0];
        assert_eq!(outside(&tune.cadence, ts), outside(&original.cadence, ts));
        assert_eq!(factory.rng.borrow().consumed(), 3);
    }

    #[test]
    fn splitting_the_whole_tune_succeeds() {
        // The last leaf has no neighbour after it; TypeScript threw here.
        let factory = VariantRngFactory {
            rng: RefCell::new(ConstantRng::new(0.0)), // every split happens
        };
        let params = variant_params(Layer::Rhythm, [8.0, 16.0], (0, 0));
        assert!(generate(&params, &base(), &factory).is_ok());
    }

    fn node(t: f64, duration: f64, pitch: Option<Notenum>, children: Vec<NoteNode>) -> NoteNode {
        NoteNode {
            rhythm: Rhythm {
                is_note_on: true,
                t,
                duration,
            },
            pitch,
            children,
        }
    }

    fn leaf(t: f64, duration: f64, pitch: Notenum) -> NoteNode {
        node(t, duration, Some(pitch), Vec::new())
    }

    fn tune_with_a_chord() -> Tune {
        let mut tune = empty_tune([4, 4], Resolution::Half, 3);
        tune.scale = Scale::major(69);
        tune.chord = Timeline::from_items([TimelineItem::new(
            0.0,
            Chord::new(tune.scale.clone(), 0, vec![0, 2, 4]),
        )]);
        tune
    }

    #[test]
    fn a_branch_under_a_merged_branch_still_draws() {
        // root: [L0, B, L3], B: [C, l2], C: [l1a, l1b]. B and C are the branches of two children.
        let c = node(1.0, 1.0, None, vec![leaf(1.0, 0.5, 70), leaf(1.5, 0.5, 71)]);
        let b = node(1.0, 2.0, None, vec![c, leaf(2.0, 1.0, 72)]);
        let mut root = node(
            0.0,
            4.0,
            None,
            vec![leaf(0.0, 1.0, 69), b, leaf(3.0, 1.0, 73)],
        );
        let mut rng = ConstantRng::new(0.0); // every merge happens
        let params = default_params().note;
        merge_rhythm(
            &tune_with_a_chord(),
            &mut root,
            [0.5, 3.0],
            &mut rng,
            &params,
        )
        .unwrap();
        // B: one draw to merge and one for its pitch; C, no longer in the tree: the same.
        assert_eq!(rng.consumed, 4);
        let leaves: Vec<_> = root
            .leaves()
            .iter()
            .map(|l| (l.rhythm.t, l.rhythm.duration))
            .collect();
        assert_eq!(leaves, [(0.0, 1.0), (1.0, 2.0), (3.0, 1.0)]);
        assert!(root.children[1].pitch.is_some());
    }

    #[test]
    fn split_interpolates_between_the_neighbours() {
        let mut root = node(
            0.0,
            3.0,
            None,
            vec![leaf(0.0, 1.0, 69), leaf(1.0, 1.0, 71), leaf(2.0, 1.0, 73)],
        );
        let mut rng = ConstantRng::new(0.0);
        let params = default_params().note;
        split_rhythm(
            &tune_with_a_chord(),
            &mut root,
            [0.5, 1.5],
            &mut rng,
            &params,
        )
        .unwrap();
        // One draw to split and one per half.
        assert_eq!(rng.consumed, 3);
        let middle = &root.children[1];
        assert_eq!(middle.pitch, Some(71));
        let halves: Vec<_> = middle
            .children
            .iter()
            .map(|n| (n.rhythm.t, n.rhythm.duration, n.pitch.is_some()))
            .collect();
        assert_eq!(halves, [(1.0, 0.5, true), (1.5, 0.5, true)]);
    }

    #[test]
    fn split_uses_the_own_pitch_at_the_end_of_the_tune() {
        let mut root = node(
            0.0,
            3.0,
            None,
            vec![leaf(0.0, 1.0, 62), leaf(1.0, 1.0, 62), leaf(2.0, 1.0, 76)],
        );
        // Split, then draw the pitches of the halves from the middle of their distributions.
        let mut rng = ScriptedRng::new(vec![0.0, 0.5, 0.5]);
        let params = default_params().note;
        split_rhythm(
            &tune_with_a_chord(),
            &mut root,
            [1.5, 3.0],
            &mut rng,
            &params,
        )
        .unwrap();
        // The halves lead from 62 to the own pitch 76 in 3 steps of less than 6 semitones.
        let halves: Vec<_> = root.children[2]
            .children
            .iter()
            .map(|n| n.pitch.unwrap())
            .collect();
        assert_eq!(rng.consumed(), 3);
        let path = [62, halves[0], halves[1], 76];
        assert!(path.windows(2).all(|w| (w[1] - w[0]).abs() < 6), "{path:?}");
    }

    #[test]
    fn interpolate_moves_step_by_step_to_the_end() {
        // relPitchFactor.edge1 = 6: the chain never moves by 6 semitones or more, so each
        // pitch must follow the previous one and still reach the end.
        let tune = tune_with_a_chord();
        let params = default_params().note.pitch_weight;
        let (start, end) = (63, 75);
        for (state, sequence) in SEEDS {
            let mut rng = ReliableRandom::new(state, sequence);
            let mut notes: Vec<NoteNode> = (0..3).map(|i| leaf(f64::from(i), 1.0, 0)).collect();
            let mut dest: Vec<&mut NoteNode> = notes.iter_mut().collect();
            interpolate(&tune, start, end, &mut rng, &mut dest, &params).unwrap();
            let mut path = vec![start];
            path.extend(notes.iter().map(|n| n.pitch.unwrap()));
            path.push(end);
            assert!(
                path.windows(2).all(|w| (w[1] - w[0]).abs() < 6),
                "{state} {sequence}: {path:?}"
            );
        }
    }

    #[test]
    fn interpolate_moves_out_of_range_ends_into_the_candidates() {
        // The candidates are 62..=76 (key 69, absPitchFactor.edge1 = 8): 100 is taken as 76.
        let tune = tune_with_a_chord();
        let params = default_params().note.pitch_weight;
        for (state, sequence) in SEEDS {
            let run = |start, end| {
                let mut notes = [leaf(0.0, 1.0, 0), leaf(1.0, 1.0, 0)];
                let [a, b] = &mut notes;
                let mut rng = ReliableRandom::new(state, sequence);
                interpolate(&tune, start, end, &mut rng, &mut [a, b], &params).unwrap();
                notes
            };
            assert_eq!(run(30, 100), run(62, 76));
        }
    }

    #[test]
    fn interpolate_drops_an_unreachable_end() {
        // One note cannot bridge 62 -> 76 when each step is less than 6 semitones.
        let tune = tune_with_a_chord();
        let params = default_params().note.pitch_weight;
        let pitches: Vec<Notenum> = SEEDS
            .iter()
            .map(|&(state, sequence)| {
                let mut note = leaf(0.0, 1.0, 0);
                let mut rng = ReliableRandom::new(state, sequence);
                interpolate(&tune, 62, 76, &mut rng, &mut [&mut note], &params).unwrap();
                note.pitch.unwrap()
            })
            .collect();
        // Drawn one step from 62, not stuck at the lowest candidate as with NaN weights.
        assert!(pitches.iter().all(|p| (62..68).contains(p)), "{pitches:?}");
        assert!(pitches.iter().any(|&p| p != 62), "{pitches:?}");
    }
}
