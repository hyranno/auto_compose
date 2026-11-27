//! Rhythm: beats divided recursively into halves.

use serde::{Deserialize, Serialize};

use crate::random::{Rng, Seed, chance};
use crate::tune::{Node, NodeKind, Rhythm, RhythmNode, Tune};
use crate::util::SmoothstepParams;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "tsify", tsify(rename = "RhythmParameters"))]
#[cfg_attr(feature = "schemars", schemars(rename = "RhythmParameters"))]
pub struct Parameters {
    /// Replaces the seed of the tune for this layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "tsify", tsify(optional))]
    pub seed: Option<Seed>,
    /// How many times a beat can be divided into halves.
    pub max_beat_division_depth: u32,
    /// The probability to divide a node, by its duration.
    pub p_branch: SmoothstepParams,
    /// The probability that a leaf is a note rather than a rest, by its duration.
    pub p_note_on: SmoothstepParams,
}

const NUM_CHILDREN: u32 = 2;

/// Divides a node of `rhythm` into two halves with the probability `p_branch`, recursively
/// while `depthcount` is positive. At depth 0 no number is drawn for the branch.
/// A leaf draws once more for `is_note_on`.
pub fn generate_sub_tree(
    rhythm: Rhythm,
    rng: &mut impl Rng,
    depthcount: u32,
    params: &Parameters,
) -> RhythmNode {
    let p_branch = params.p_branch.eval(rhythm.duration);
    let is_branch = 0 < depthcount && chance(rng, p_branch);
    if is_branch {
        let sub_duration = rhythm.duration / f64::from(NUM_CHILDREN);
        let children = (0..NUM_CHILDREN)
            .map(|i| {
                let t = rhythm.t + f64::from(i) * sub_duration;
                generate_sub_tree(Rhythm::new(t, sub_duration), rng, depthcount - 1, params)
            })
            .collect();
        Node::branch(rhythm, children)
    } else {
        let p_note_on = params.p_note_on.eval(rhythm.duration);
        Node::leaf(rhythm, chance(rng, p_note_on))
    }
}

/// Merges adjacent nodes with the probability `1 - p_branch` of the merged duration, drawing
/// once per pair. A merge skips the next pair. The merged node is a leaf that is on if the
/// first node is on or is a branch.
pub fn merge_top_branches(
    nodes: Vec<RhythmNode>,
    rng: &mut impl Rng,
    params: &Parameters,
) -> Vec<RhythmNode> {
    let mut res = Vec::with_capacity(nodes.len());
    let mut nodes = nodes.into_iter().peekable();
    while let Some(node) = nodes.next() {
        if let Some(next) = nodes.peek() {
            let duration = node.rhythm.duration + next.rhythm.duration;
            let p_merge = 1.0 - params.p_branch.eval(duration);
            if chance(rng, p_merge) {
                nodes.next();
                let is_note_on = match node.kind {
                    NodeKind::Leaf(on) => on,
                    NodeKind::Branch(_) => true,
                };
                res.push(Node::leaf(Rhythm::new(node.rhythm.t, duration), is_note_on));
                continue;
            }
        }
        res.push(node);
    }
    res
}

/// A phrase of bars of beats. Each beat is divided by [`generate_sub_tree`], and then the
/// beats of the bar are merged by [`merge_top_branches`].
pub fn generate(tune: &Tune, params: &Parameters, rng: &mut impl Rng) -> RhythmNode {
    let num_beats = tune.time_measure.beats_per_bar;
    let num_bars = tune.time_measure.bars;
    let bars = (0..num_bars)
        .map(|i| {
            let t = f64::from(i * num_beats);
            let beats = (0..num_beats)
                .map(|j| {
                    let beat = Rhythm::new(t + f64::from(j), 1.0);
                    generate_sub_tree(beat, rng, params.max_beat_division_depth, params)
                })
                .collect();
            let beats = merge_top_branches(beats, rng, params);
            Node::branch(Rhythm::new(t, f64::from(num_beats)), beats)
        })
        .collect();
    Node::branch(Rhythm::new(0.0, f64::from(num_beats * num_bars)), bars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::{ConstantRng, SEEDS, default_params, empty_tune, seeded};
    use crate::tune::Resolution;

    #[test]
    fn structure() {
        let base = default_params().rhythm;
        for measure in [[4, 4], [3, 4]] {
            for depth in 0..4 {
                let tune = empty_tune(measure, Resolution::Half);
                let params = Parameters {
                    max_beat_division_depth: depth,
                    ..base.clone()
                };
                for seed in SEEDS {
                    let what = format!("{measure:?} depth={depth} seed={seed}");
                    let root = generate(&tune, &params, &mut seeded(seed));
                    assert_eq!(root.rhythm.duration, tune.length, "{what}");
                    assert_eq!(root.children().len(), measure[1] as usize, "{what}");
                    for (i, bar) in root.children().iter().enumerate() {
                        assert_eq!(bar.rhythm.t, (i as u32 * measure[0]) as f64, "{what}");
                        let sum: f64 = bar.children().iter().map(|n| n.rhythm.duration).sum();
                        assert_eq!(sum, f64::from(measure[0]), "{what}");
                    }
                    // Leaves tile the whole phrase without gaps or overlaps.
                    let mut t = 0.0;
                    for leaf in root.rhythms() {
                        assert_eq!(leaf.t, t, "{what}");
                        assert!(leaf.duration >= 2f64.powi(-(depth as i32)), "{what}");
                        t += leaf.duration;
                    }
                    assert_eq!(t, tune.length, "{what}");
                }
            }
        }
    }

    fn with_depth(max_beat_division_depth: u32) -> Parameters {
        Parameters {
            max_beat_division_depth,
            ..default_params().rhythm
        }
    }

    fn rhythms(root: &RhythmNode) -> Vec<(f64, f64, bool)> {
        root.leaves()
            .iter()
            .map(|&(r, &on)| (r.t, r.duration, on))
            .collect()
    }

    #[test]
    fn depth_0_with_draws_of_0_every_beat_is_on_and_pairs_of_beats_merge() {
        // Per bar: 4 is_note_on draws (no branch draw at depth 0), then merge draws for the pairs
        // (0,1) and (2,3) (a merge skips the next pair). p_merge(2) = 0.3006... > 0.
        let mut rng = ConstantRng::new(0.0);
        let tune = empty_tune([4, 4], Resolution::Half);
        let root = generate(&tune, &with_depth(0), &mut rng);
        assert_eq!(rng.consumed, 4 * (4 + 2));
        let expected: Vec<_> = (0..8).map(|i| (f64::from(i * 2), 2.0, true)).collect();
        assert_eq!(rhythms(&root), expected);
    }

    #[test]
    fn depth_0_with_draws_close_to_1_every_beat_is_off_and_nothing_merges() {
        let mut rng = ConstantRng::new(1.0 - 2f64.powi(-53));
        let tune = empty_tune([4, 4], Resolution::Half);
        let root = generate(&tune, &with_depth(0), &mut rng);
        assert_eq!(rng.consumed, 4 * (4 + 3));
        let expected: Vec<_> = (0..16).map(|t| (f64::from(t), 1.0, false)).collect();
        assert_eq!(rhythms(&root), expected);
    }

    #[test]
    fn depth_3_with_draws_of_0_branching_stops_where_p_branch_is_0() {
        // p_branch(1) > 0 and p_branch(0.5) > 0 but p_branch(0.25) = 0 (edge0 = 0.25), so each beat
        // makes 1 + 2 * (1 + 2 * 2) = 11 draws. The merges then drop the subtrees of the merged
        // beats.
        let mut rng = ConstantRng::new(0.0);
        let tune = empty_tune([4, 4], Resolution::Half);
        let root = generate(&tune, &with_depth(3), &mut rng);
        assert_eq!(rng.consumed, 4 * (4 * 11 + 2));
        let actual: Vec<_> = rhythms(&root).iter().map(|&(t, d, _)| (t, d)).collect();
        let expected: Vec<_> = (0..8).map(|i| (f64::from(i * 2), 2.0)).collect();
        assert_eq!(actual, expected);
    }
}
