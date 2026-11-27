//! A variant of a tune: one layer regenerated within a timespan.
//!
//! The variant draws from its own generator (seeded by the seed of the variant): first the
//! timespan, then the layer, then everything the variant of that layer needs. The original tune
//! is generated in between with the seeds of the base parameters, and the layers below the
//! chosen one are generated on top of the variant.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::markov::{Condition, MarkovChain};
use crate::random::{Rng, Seed, chance, layer_rng};
use crate::tune::{Cadence, Chord, Note, NoteNode, Notenum, Resolution, Rhythm, Tune};
use crate::util::{Timeline, TimelineItem, WeightedItem, weighted_pick};

use super::{cadence, chord, note, pitch_weight, rhythm, tune as tune_generator};

/// The layer to vary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    Cadence,
    Chord,
    Pitch,
    Rhythm,
}

impl Layer {
    pub const ALL: [Layer; 4] = [Self::Cadence, Self::Chord, Self::Pitch, Self::Rhythm];
}

/// A span of time in beats. The items strictly inside it are varied.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Timespan {
    pub start: f64,
    pub end: f64,
}

impl Timespan {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    /// Whether `t` is strictly inside.
    pub fn contains(self, t: f64) -> bool {
        self.start < t && t < self.end
    }
}

/// The parameters of a variant, as in the JSON file
/// (`resources/VariantGenParams_default.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "tsify", tsify(rename = "VariantParameters"))]
#[cfg_attr(feature = "schemars", schemars(rename = "VariantParameters"))]
pub struct Parameters {
    pub seed: Seed,
    /// Replaces the resolution of the base parameters.
    pub resolution: Resolution,
    /// The weights of the timespans.
    pub timespan: Vec<WeightedItem<Timespan>>,
    /// The weights of the layers.
    pub layer: Vec<WeightedItem<Layer>>,
}

/// `resources/VariantGenParams_default.json`, the single source of the default values.
impl Default for Parameters {
    fn default() -> Self {
        serde_json::from_str(include_str!(
            "../../../../resources/VariantGenParams_default.json"
        ))
        .expect("the default variant parameters are valid")
    }
}

/// Generates the tune of `base` (with the resolution of `params`) and varies one layer of it.
pub fn generate(params: &Parameters, base: &tune_generator::Parameters) -> Result<Tune> {
    generate_with(params, base, &mut layer_rng(params.seed, "variant"))
}

/// [`generate`] with the generator of the variant given.
pub fn generate_with(
    params: &Parameters,
    base: &tune_generator::Parameters,
    rng: &mut impl Rng,
) -> Result<Tune> {
    let timespan = *weighted_pick(&params.timespan, rng)?;
    let layer = *weighted_pick(&params.layer, rng)?;
    let tune_params = tune_generator::Parameters {
        resolution: params.resolution,
        ..base.clone()
    };
    let p = &tune_params;
    let mut tune = tune_generator::empty(p)?;
    tune.cadence = cadence::generate(
        &tune,
        &p.cadence,
        &mut p.layer_rng(p.cadence.seed, "cadence"),
    )?;
    if layer == Layer::Cadence {
        tune.cadence = vary_cadence(&tune, timespan, rng, &p.cadence)?;
    }
    tune.chord = chord::generate(&tune, &p.chord, &mut p.layer_rng(p.chord.seed, "chord"))?;
    if layer == Layer::Chord {
        tune.chord = vary_chord(&tune, timespan, rng, &p.chord)?;
    }
    let mut tree = tune_generator::generate_notes(&tune, p)?;
    match layer {
        Layer::Pitch => vary_pitch(&tune, &mut tree, timespan, rng, &p.pitch)?,
        Layer::Rhythm => vary_rhythm(&tune, &mut tree, timespan, rng, p)?,
        Layer::Cadence | Layer::Chord => {}
    }
    tune.notes = note::tree_to_timeline(&tree)?;
    Ok(tune)
}

/// The indices of `timeline` at both ends of `timespan` ([`Timeline::get_index`]).
/// A timespan that starts before the first item is an error.
fn span_indices<T>(timeline: &Timeline<T>, timespan: Timespan) -> Result<[usize; 2]> {
    match [timespan.start, timespan.end].map(|t| timeline.get_index(t)) {
        [Some(start), Some(end)] => Ok([start, end]),
        _ => Err(Error::InvalidParameter("timespan")),
    }
}

/// Redraws the cadences strictly inside the timespan by the Markov chain, starting from the
/// cadence at the start and conditioned on reaching the cadence at the end.
pub fn vary_cadence(
    tune: &Tune,
    timespan: Timespan,
    rng: &mut impl Rng,
    params: &cadence::Parameters,
) -> Result<Timeline<Cadence>> {
    let [start, end] = span_indices(&tune.cadence, timespan)?;
    let list = tune.cadence.list();
    if end <= start + 1 {
        return Ok(tune.cadence.clone());
    }
    let chain = params.chain()?;
    let goal = list[end].value;
    let mut prev = list[start].value;
    let items = list
        .iter()
        .enumerate()
        .map(|(i, e)| {
            if !(start < i && i < end) {
                return Ok(e.clone());
            }
            prev = chain.sample_next(&prev, &Condition::new(end - i, goal), rng)?;
            Ok(TimelineItem::new(e.t, prev))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(items))
}

/// Redraws the root and the tones of the chords strictly inside the timespan.
pub fn vary_chord(
    tune: &Tune,
    timespan: Timespan,
    rng: &mut impl Rng,
    params: &chord::Parameters,
) -> Result<Timeline<Chord>> {
    let [start, end] = span_indices(&tune.chord, timespan)?;
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
            Ok(TimelineItem::new(e.t, params.draw(cadence, rng)?))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(items))
}

/// Draws the pitches of `dest` so that they lead from `start` to `end`.
///
/// The pitches move by a Markov chain on `key ± (ceil(abs_pitch_factor.edge1) - 1)` whose
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
        // The Markov chain needs a state.
        return Err(Error::InvalidParameter("pitch.abs_pitch_factor.edge1"));
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
    let chain = MarkovChain::new(&transition)?;
    let nearest = |p: Notenum| p.clamp(candidates[0], candidates[candidates.len() - 1]);
    let end = nearest(end);
    let mut prev = nearest(start);
    let len = dest.len();
    for (i, note) in dest.iter_mut().enumerate() {
        // From the note `i`, `len - 1 - i` notes and then `end` follow.
        let dist = chain.next_distribution(&prev, &Condition::new(len - i + 1, end))?;
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
        let pitch = *weighted_pick(&items, rng)?;
        note.pitch = Some(pitch);
        prev = pitch;
    }
    Ok(())
}

/// Redraws the pitches of the leaves of `tree` strictly inside the timespan
/// (by the indices of the leaves at its ends).
pub fn vary_pitch(
    tune: &Tune,
    tree: &mut NoteNode,
    timespan: Timespan,
    rng: &mut impl Rng,
    params: &pitch_weight::Parameters,
) -> Result<()> {
    let [start, end] = span_indices(&note::tree_to_timeline(tree)?, timespan)?;
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
    interpolate(tune, start_pitch, end_pitch, rng, dest, params)
}

/// Splits, merges and toggles the leaves of `tree` inside the timespan.
pub fn vary_rhythm(
    tune: &Tune,
    tree: &mut NoteNode,
    timespan: Timespan,
    rng: &mut impl Rng,
    params: &tune_generator::Parameters,
) -> Result<()> {
    split_rhythm(tune, tree, timespan, rng, params)?;
    merge_rhythm(tune, tree, timespan, rng, params)?;
    toggle_note_on(tree, timespan, rng, &params.rhythm);
    Ok(())
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

/// Splits each leaf inside the timespan into halves with the probability `p_branch` of its
/// duration, drawing once per leaf, and interpolates the pitches of the halves between the
/// neighbour leaves. The first and the last leaf use their own pitch on the side without a
/// neighbour.
pub fn split_rhythm(
    tune: &Tune,
    root: &mut NoteNode,
    timespan: Timespan,
    rng: &mut impl Rng,
    params: &tune_generator::Parameters,
) -> Result<()> {
    let mut leaves = root.leaves_mut();
    // Splitting a leaf does not change the pitches of the leaves.
    let pitches: Vec<Option<Notenum>> = leaves.iter().map(|l| l.pitch).collect();
    for (i, leaf) in leaves.iter_mut().enumerate() {
        let rhythm = leaf.rhythm;
        if !timespan.contains(rhythm.t) {
            continue;
        }
        let p_split = params.rhythm.p_branch.eval(rhythm.duration);
        if chance(rng, p_split) {
            let sub_duration = rhythm.duration / NUM_CHILDREN as f64;
            leaf.children.extend(
                (0..NUM_CHILDREN)
                    .map(|k| rhythm::new_node(rhythm.t + k as f64 * sub_duration, sub_duration)),
            );
            let start = neighbour_pitch(&pitches, i.checked_sub(1), i)?;
            let end = neighbour_pitch(&pitches, Some(i + 1), i)?;
            let mut children: Vec<&mut NoteNode> = leaf.children.iter_mut().collect();
            interpolate(tune, start, end, rng, &mut children, &params.pitch)?;
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

fn node_at_mut<'a>(root: &'a mut NoteNode, path: &[usize]) -> Option<&'a mut NoteNode> {
    path.iter().try_fold(root, |n, &i| n.children.get_mut(i))
}

/// Merges each branch of two children inside the timespan into one note with the probability
/// `1 - p_branch` of its duration, drawing once per branch, and interpolates its pitch between
/// the leaves next to the leaves of the branch.
///
/// The branches are visited in pre-order, and a branch under a merged one is skipped without
/// drawing. The neighbour pitches are those before any merge. The merged note is on if any of
/// its leaves was on.
pub fn merge_rhythm(
    tune: &Tune,
    root: &mut NoteNode,
    timespan: Timespan,
    rng: &mut impl Rng,
    params: &tune_generator::Parameters,
) -> Result<()> {
    let leaves: Vec<Rhythm> = root.leaves().iter().map(|l| l.rhythm).collect();
    let pitches: Vec<Option<Notenum>> = root.leaves().iter().map(|l| l.pitch).collect();
    let mut paths = Vec::new();
    branch_paths(root, &mut Vec::new(), &mut paths);
    let mut merged_paths: Vec<Vec<usize>> = Vec::new();
    for path in paths {
        if merged_paths.iter().any(|m| path.starts_with(m)) {
            continue;
        }
        let branch = node_at_mut(root, &path).expect("a path just collected");
        let rhythm = branch.rhythm;
        if !timespan.contains(rhythm.t) {
            continue;
        }
        let p_merge = 1.0 - params.rhythm.p_branch.eval(rhythm.duration);
        if chance(rng, p_merge) {
            let end_t = rhythm.t + rhythm.duration;
            let under = |l: &Rhythm| rhythm.t <= l.t && l.t < end_t;
            let i0 = leaves.iter().position(under).expect("a branch has leaves");
            let i1 = leaves.iter().rposition(under).expect("a branch has leaves");
            let start = neighbour_pitch(&pitches, i0.checked_sub(1), i0)?;
            let end = neighbour_pitch(&pitches, Some(i1 + 1), i1)?;
            let mut merged = NoteNode::new(Rhythm {
                is_note_on: leaves[i0..=i1].iter().any(|l| l.is_note_on),
                ..rhythm
            });
            interpolate(tune, start, end, rng, &mut [&mut merged], &params.pitch)?;
            *branch = merged;
            merged_paths.push(path);
        }
    }
    Ok(())
}

/// Redraws whether each leaf inside the timespan is a note or a rest, with the probability
/// `p_note_on` of its duration.
pub fn toggle_note_on(
    root: &mut NoteNode,
    timespan: Timespan,
    rng: &mut impl Rng,
    params: &rhythm::Parameters,
) {
    for leaf in root.leaves_mut() {
        if !timespan.contains(leaf.rhythm.t) {
            continue;
        }
        let p_note_on = params.p_note_on.eval(leaf.rhythm.duration);
        leaf.rhythm.is_note_on = chance(rng, p_note_on);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::{ConstantRng, SEEDS, default_params, empty_tune, seeded};
    use crate::random::ScriptedRng;
    use crate::tune::Scale;

    const TIMESPANS: [[f64; 2]; 5] = [
        [6.0, 11.0],
        [2.0, 8.0],
        [9.0, 15.0],
        [0.0, 16.0],
        [3.0, 5.0],
    ];

    fn span([start, end]: [f64; 2]) -> Timespan {
        Timespan::new(start, end)
    }

    fn variant_params(layer: Layer, timespan: [f64; 2], seed: Seed) -> Parameters {
        Parameters {
            seed,
            resolution: Resolution::Half,
            timespan: vec![WeightedItem::new(1.0, span(timespan))],
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

    #[test]
    fn reads_the_resource_file() {
        let p = Parameters::default();
        assert_eq!(p.resolution, Resolution::PerfectAuthentic);
        assert_eq!(
            p.timespan,
            [WeightedItem::new(1.0, Timespan::new(6.0, 11.0))]
        );
        assert_eq!(p.layer, [WeightedItem::new(1.0, Layer::Rhythm)]);
        let json = serde_json::to_string(&p).unwrap();
        assert!(
            json.contains(r#""layer":[{"weight":1.0,"value":"rhythm"}]"#),
            "{json}"
        );
        assert_eq!(serde_json::from_str::<Parameters>(&json).unwrap(), p);
        assert!(serde_json::from_str::<Layer>("3").is_err());
    }

    #[test]
    fn layers_0_to_2_keep_the_original_outside_the_timespan() {
        let original = tune_generator::generate(&base()).unwrap();
        for layer in [Layer::Cadence, Layer::Chord, Layer::Pitch] {
            for ts in TIMESPANS {
                for seed in SEEDS {
                    let params = variant_params(layer, ts, seed);
                    let tune = generate(&params, &base()).unwrap();
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
        let original = tune_generator::generate(&base()).unwrap();
        for ts in TIMESPANS {
            for seed in SEEDS {
                let params = variant_params(Layer::Rhythm, ts, seed);
                let tune = generate(&params, &base()).unwrap();
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
            seed: 0,
            resolution: Resolution::Half,
            timespan: vec![
                WeightedItem::new(1.0, Timespan::new(2.0, 6.0)),
                WeightedItem::new(1.0, Timespan::new(6.0, 11.0)),
            ],
            layer: vec![
                WeightedItem::new(1.0, Layer::Cadence),
                WeightedItem::new(1.0, Layer::Chord),
            ],
        };
        let mut values = vec![0.75, 0.25];
        values.extend([0.5; 16]);
        let mut rng = ScriptedRng::new(values);
        let tune = generate_with(&params, &base(), &mut rng).unwrap();
        let original = tune_generator::generate(&base()).unwrap();
        // Timespan [6, 11] and layer Cadence; the Cadence variant then draws for the slots
        // strictly inside: the cadences last 2 beats, so only the one at t = 8.
        let ts = [6.0, 11.0];
        assert_eq!(outside(&tune.cadence, ts), outside(&original.cadence, ts));
        assert_eq!(rng.consumed(), 3);
    }

    #[test]
    fn splitting_the_whole_tune_succeeds() {
        // The last leaf has no neighbour after it.
        let mut rng = ConstantRng::new(0.0); // every split happens
        let params = variant_params(Layer::Rhythm, [8.0, 16.0], 0);
        assert!(generate_with(&params, &base(), &mut rng).is_ok());
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
        let mut tune = empty_tune([4, 4], Resolution::Half);
        tune.scale = Scale::major(69);
        tune.chord = Timeline::from_items([TimelineItem::new(0.0, Chord::new(0, vec![0, 2, 4]))]);
        tune
    }

    #[test]
    fn a_branch_under_a_merged_branch_is_skipped() {
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
        let params = default_params();
        merge_rhythm(
            &tune_with_a_chord(),
            &mut root,
            span([0.5, 3.0]),
            &mut rng,
            &params,
        )
        .unwrap();
        // B: one draw to merge and one for its pitch; C, no longer in the tree, draws nothing.
        assert_eq!(rng.consumed, 2);
        let leaves: Vec<_> = root
            .leaves()
            .iter()
            .map(|l| (l.rhythm.t, l.rhythm.duration))
            .collect();
        assert_eq!(leaves, [(0.0, 1.0), (1.0, 2.0), (3.0, 1.0)]);
        assert!(root.children[1].pitch.is_some());
    }

    #[test]
    fn a_timespan_before_the_tune_is_an_error() {
        // The rhythm variant does not look up the timelines and accepts it.
        for layer in [Layer::Cadence, Layer::Chord, Layer::Pitch] {
            let params = variant_params(layer, [-1.0, 5.0], 0);
            assert_eq!(
                generate(&params, &base()),
                Err(Error::InvalidParameter("timespan")),
                "{layer:?}"
            );
        }
    }

    #[test]
    fn vary_chord_takes_the_indices_of_the_chords() {
        // Two chords per cadence: the indices of the cadences would select nothing here.
        let mut tune = tune_with_a_chord();
        tune.cadence = Timeline::from_items([
            TimelineItem::new(0.0, Cadence::Tonic),
            TimelineItem::new(4.0, Cadence::Dominant),
        ]);
        tune.chord = Timeline::from_items(
            [0.0, 2.0, 4.0, 6.0].map(|t| TimelineItem::new(t, Chord::new(0, vec![0]))),
        );
        let mut rng = ConstantRng::new(0.5);
        let chord = vary_chord(&tune, span([1.0, 5.0]), &mut rng, &default_params().chord).unwrap();
        // The chord at t = 2 is strictly between the chords at the ends: a root and tones.
        assert_eq!(rng.consumed, 2);
        let changed: Vec<f64> = chord
            .list()
            .iter()
            .zip(tune.chord.list())
            .filter(|(a, b)| a != b)
            .map(|(a, _)| a.t)
            .collect();
        assert_eq!(changed, [2.0]);
    }

    #[test]
    fn merge_interpolates_between_the_leaves_next_to_the_branch() {
        // root: [L0, B, L3], B: [l1, C], C: [l2a, l2b]. The leaf after B is L3, not l2b.
        let off = |t, duration, pitch| {
            let mut n = leaf(t, duration, pitch);
            n.rhythm.is_note_on = false;
            n
        };
        let c = node(2.0, 1.0, None, vec![off(2.0, 0.5, 90), off(2.5, 0.5, 90)]);
        let b = node(1.0, 2.0, None, vec![off(1.0, 1.0, 90), c]);
        let mut root = node(
            0.0,
            4.0,
            None,
            vec![leaf(0.0, 1.0, 66), b, leaf(3.0, 1.0, 74)],
        );
        // Merge B, then draw its pitch from the middle of the distribution.
        let mut rng = ScriptedRng::new(vec![0.0, 0.5]);
        let params = default_params();
        merge_rhythm(
            &tune_with_a_chord(),
            &mut root,
            span([0.5, 3.0]),
            &mut rng,
            &params,
        )
        .unwrap();
        assert_eq!(rng.consumed(), 2);
        let merged = &root.children[1];
        assert!(merged.children.is_empty());
        // Less than 6 semitones from both 66 and 74.
        assert!((69..=70).contains(&merged.pitch.unwrap()), "{merged:?}");
        assert!(!merged.rhythm.is_note_on);
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
        let params = default_params();
        split_rhythm(
            &tune_with_a_chord(),
            &mut root,
            span([0.5, 1.5]),
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
        let params = default_params();
        split_rhythm(
            &tune_with_a_chord(),
            &mut root,
            span([1.5, 3.0]),
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
        // rel_pitch_factor.edge1 = 6: the chain never moves by 6 semitones or more, so each
        // pitch must follow the previous one and still reach the end.
        let tune = tune_with_a_chord();
        let params = default_params().pitch;
        let (start, end) = (63, 75);
        for seed in SEEDS {
            let mut rng = seeded(seed);
            let mut notes: Vec<NoteNode> = (0..3).map(|i| leaf(f64::from(i), 1.0, 0)).collect();
            let mut dest: Vec<&mut NoteNode> = notes.iter_mut().collect();
            interpolate(&tune, start, end, &mut rng, &mut dest, &params).unwrap();
            let mut path = vec![start];
            path.extend(notes.iter().map(|n| n.pitch.unwrap()));
            path.push(end);
            assert!(
                path.windows(2).all(|w| (w[1] - w[0]).abs() < 6),
                "{seed}: {path:?}"
            );
        }
    }

    #[test]
    fn interpolate_moves_out_of_range_ends_into_the_candidates() {
        // The candidates are 62..=76 (key 69, abs_pitch_factor.edge1 = 8): 100 is taken as 76.
        let tune = tune_with_a_chord();
        let params = default_params().pitch;
        for seed in SEEDS {
            let run = |start, end| {
                let mut notes = [leaf(0.0, 1.0, 0), leaf(1.0, 1.0, 0)];
                let [a, b] = &mut notes;
                let mut rng = seeded(seed);
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
        let params = default_params().pitch;
        let pitches: Vec<Notenum> = SEEDS
            .iter()
            .map(|&seed| {
                let mut note = leaf(0.0, 1.0, 0);
                let mut rng = seeded(seed);
                interpolate(&tune, 62, 76, &mut rng, &mut [&mut note], &params).unwrap();
                note.pitch.unwrap()
            })
            .collect();
        // Drawn one step from 62, not stuck at the lowest candidate as with NaN weights.
        assert!(pitches.iter().all(|p| (62..68).contains(p)), "{pitches:?}");
        assert!(pitches.iter().any(|&p| p != 62), "{pitches:?}");
    }
}
