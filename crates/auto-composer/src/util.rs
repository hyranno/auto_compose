//! Small numeric helpers, weighted random choice and timelines.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::random::Rng;

/// Hermite interpolation between `edge0` and `edge1`, clamped to `[0, 1]`.
///
/// `edge0 == edge1` gives a step function: 0 before the edge and 1 from it.
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    if edge0 == edge1 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let xn = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    xn * xn * (3.0 - 2.0 * xn)
}

/// The edges of a [`smoothstep`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SmoothstepParams {
    pub edge0: f64,
    pub edge1: f64,
}

impl SmoothstepParams {
    pub fn eval(&self, x: f64) -> f64 {
        smoothstep(self.edge0, self.edge1, x)
    }
}

/// `start, start + tick, ...` while the value is less than `end`. Each value is
/// `start + i * tick`.
///
/// Panics if `tick` is not positive and finite.
pub fn range_step(start: f64, end: f64, tick: f64) -> impl Iterator<Item = f64> {
    assert!(
        tick > 0.0 && tick.is_finite(),
        "the step must be positive and finite"
    );
    let count = ((end - start) / tick).ceil().max(0.0) as usize;
    (0..count).map(move |i| start + i as f64 * tick)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct WeightedItem<T> {
    pub weight: f64,
    pub value: T,
}

impl<T> WeightedItem<T> {
    pub fn new(weight: f64, value: T) -> Self {
        Self { weight, value }
    }
}

/// Picks an item with a probability proportional to its weight, drawing one random number.
///
/// An item of weight 0 is never chosen. It is an error if `items` is empty (nothing is drawn
/// then), or if a weight is negative or not finite, or the weights are all 0.
pub fn weighted_pick<'a, T>(items: &'a [WeightedItem<T>], rng: &mut impl Rng) -> Result<&'a T> {
    if items.is_empty() {
        return Err(Error::NoItems);
    }
    let cumulative: Vec<f64> = items
        .iter()
        .scan(0.0, |acc, item| {
            *acc += item.weight;
            Some(*acc)
        })
        .collect();
    let total = cumulative[cumulative.len() - 1];
    let valid = items
        .iter()
        .all(|item| item.weight >= 0.0 && item.weight.is_finite());
    if !valid || !(total > 0.0 && total.is_finite()) {
        return Err(Error::InvalidWeights);
    }
    let x = rng.random() * total;
    let index = cumulative.partition_point(|&c| c <= x);
    // `x` rounded up to the total: the last item that can be chosen.
    let index = if index < items.len() {
        index
    } else {
        items
            .iter()
            .rposition(|item| item.weight > 0.0)
            .expect("the total is positive")
    };
    Ok(&items[index].value)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct TimelineItem<T> {
    pub t: f64,
    pub value: T,
}

impl<T> TimelineItem<T> {
    pub fn new(t: f64, value: T) -> Self {
        Self { t, value }
    }
}

/// Items sorted by time. Items with equal times keep their insertion order.
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline<T> {
    items: Vec<TimelineItem<T>>,
}

impl<T> Default for Timeline<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> Timeline<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_items(items: impl IntoIterator<Item = TimelineItem<T>>) -> Self {
        let mut res = Self::new();
        res.add(items);
        res
    }

    /// Inserts each item after the items whose time is not after it.
    pub fn add(&mut self, items: impl IntoIterator<Item = TimelineItem<T>>) {
        for item in items {
            let i = self.upper_bound(item.t);
            self.items.insert(i, item);
        }
    }

    /// Adds the items of `src` shifted by `offset`.
    pub fn merge(&mut self, offset: f64, src: &Timeline<T>)
    where
        T: Clone,
    {
        self.add(
            src.items
                .iter()
                .map(|e| TimelineItem::new(e.t + offset, e.value.clone())),
        );
    }

    /// The last item whose time is not after `t`. `None` before the first item.
    pub fn get(&self, t: f64) -> Option<&TimelineItem<T>> {
        self.get_index(t).map(|i| &self.items[i])
    }

    /// The index of [`Timeline::get`]. `None` before the first item.
    pub fn get_index(&self, t: f64) -> Option<usize> {
        self.upper_bound(t).checked_sub(1)
    }

    /// The number of items whose time is not after `t`.
    fn upper_bound(&self, t: f64) -> usize {
        self.items
            .partition_point(|item| item.t.total_cmp(&t).is_le())
    }

    pub fn list(&self) -> &[TimelineItem<T>] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{ReliableRandom, ScriptedRng};

    #[test]
    fn smoothstep_basic() {
        let cases = [(-1.0, 0.0), (0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (2.0, 1.0)];
        for (x, expected) in cases {
            assert_eq!(smoothstep(0.0, 1.0, x), expected, "x = {x}");
        }
    }

    #[test]
    fn smoothstep_table() {
        // (edge0, edge1, x, expected)
        let cases = [
            (0.25, 3.0, 0.25, 0.0),
            (0.25, 3.0, 3.0, 1.0),
            (0.25, 3.0, 1.625, 0.5),
            (0.25, 3.0, 1.0, 0.18256949661908337),
            (-2.0, 1.4, 0.125, 0.68359375),
            // Reversed edges: the curve goes from 1 down to 0.
            (1.0, 0.0, 0.25, 0.84375),
        ];
        for (e0, e1, x, expected) in cases {
            assert_eq!(
                smoothstep(e0, e1, x),
                expected,
                "smoothstep({e0}, {e1}, {x})"
            );
        }
    }

    #[test]
    fn smoothstep_equal_edges_is_a_step() {
        assert_eq!(
            [0.0, 1.0, 2.0].map(|x| smoothstep(1.0, 1.0, x)),
            [0.0, 1.0, 1.0]
        );
    }

    #[test]
    fn range_step_multiplies() {
        assert_eq!(
            range_step(0.0, 4.0, 1.0).collect::<Vec<_>>(),
            [0.0, 1.0, 2.0, 3.0]
        );
        assert_eq!(range_step(0.0, 3.0, 2.0).collect::<Vec<_>>(), [0.0, 2.0]);
        assert_eq!(range_step(0.0, 0.0, 1.0).count(), 0);
        assert_eq!(range_step(1.0, 0.0, 1.0).count(), 0);
        assert_eq!(
            range_step(0.0, 0.35, 0.1).collect::<Vec<_>>(),
            [0.0, 0.1, 0.2, 3.0 * 0.1]
        );
        assert_eq!(range_step(2.0, 3.0, 0.5).collect::<Vec<_>>(), [2.0, 2.5]);
    }

    #[test]
    #[should_panic(expected = "positive and finite")]
    fn range_step_rejects_a_step_that_is_not_positive() {
        let _ = range_step(0.0, 1.0, 0.0);
    }

    fn uniform(n: usize) -> Vec<WeightedItem<usize>> {
        (0..n).map(|v| WeightedItem::new(1.0, v)).collect()
    }

    fn pick_with<T: Clone>(items: &[WeightedItem<T>], rng: &mut impl Rng, n: usize) -> Vec<T> {
        (0..n)
            .map(|_| weighted_pick(items, rng).unwrap().clone())
            .collect()
    }

    #[test]
    fn weighted_pick_with_seeds() {
        let items = uniform(5);
        let mut r = ReliableRandom::new(754, 489);
        assert_eq!(pick_with(&items, &mut r, 4), [3, 1, 0, 3]);
        let mut r = ReliableRandom::new(222, 9643);
        assert_eq!(pick_with(&items, &mut r, 4), [1, 2, 4, 0]);
    }

    fn pick_scripted<T: Clone>(items: &[WeightedItem<T>], values: &[f64]) -> Vec<T> {
        pick_with(items, &mut ScriptedRng::new(values), values.len())
    }

    #[test]
    fn weighted_pick_draw_equal_to_a_ratio_picks_the_upper_item() {
        // cumulative ratios: 0.25, 0.75, 1
        let items = [
            WeightedItem::new(1.0, 'a'),
            WeightedItem::new(2.0, 'b'),
            WeightedItem::new(1.0, 'c'),
        ];
        let draws = [0.0, 0.25, 0.2500001, 0.75, 0.7500001, 1.0 - 2f64.powi(-32)];
        assert_eq!(
            pick_scripted(&items, &draws),
            ['a', 'b', 'b', 'c', 'c', 'c']
        );
    }

    #[test]
    fn weighted_pick_never_picks_zero_weight() {
        let items = [
            WeightedItem::new(0.0, 'a'),
            WeightedItem::new(1.0, 'b'),
            WeightedItem::new(0.0, 'c'),
        ];
        let draws = [0.0, 2f64.powi(-32), 0.5, 1.0 - 2f64.powi(-32)];
        assert_eq!(pick_scripted(&items, &draws), ['b', 'b', 'b', 'b']);
    }

    #[test]
    fn weighted_pick_a_draw_rounded_up_to_the_total_picks_the_last_possible_item() {
        let items = [
            WeightedItem::new(1.0, 'a'),
            WeightedItem::new(2.0, 'b'),
            WeightedItem::new(0.0, 'c'),
        ];
        assert_eq!(pick_scripted(&items, &[1.0]), ['b']);
    }

    #[test]
    fn weighted_pick_invalid_weights() {
        for bad in [
            [0.0, 0.0],
            [-1.0, 2.0],
            [f64::NAN, 1.0],
            [f64::INFINITY, 1.0],
        ] {
            let items = bad.map(|w| WeightedItem::new(w, ()));
            let mut rng = ScriptedRng::new([0.5]);
            assert_eq!(
                weighted_pick(&items, &mut rng),
                Err(Error::InvalidWeights),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn weighted_pick_consumes_one_value_per_call() {
        let mut rng = ScriptedRng::new([0.1, 0.9]);
        assert_eq!(pick_with(&uniform(2), &mut rng, 2), [0, 1]);
        assert_eq!(rng.consumed(), 2);
    }

    #[test]
    fn weighted_pick_empty() {
        let mut rng = ScriptedRng::new([0.5]);
        assert_eq!(weighted_pick::<i32>(&[], &mut rng), Err(Error::NoItems));
        assert_eq!(rng.consumed(), 0);
    }

    fn timeline<T>(items: impl IntoIterator<Item = (f64, T)>) -> Timeline<T> {
        Timeline::from_items(items.into_iter().map(|(t, v)| TimelineItem::new(t, v)))
    }

    fn events() -> Timeline<f64> {
        timeline([(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)])
    }

    const PROBES: [f64; 4] = [0.6, 1.6, 2.6, 3.6];

    fn values<T: Clone>(tl: &Timeline<T>) -> Vec<T> {
        tl.list().iter().map(|e| e.value.clone()).collect()
    }

    #[test]
    fn timeline_get() {
        let mut tl = events();
        let got = |tl: &Timeline<f64>| PROBES.map(|t| tl.get(t).unwrap().value);
        assert_eq!(got(&tl), [0.0, 1.0, 2.0, 3.0]);
        tl.add([TimelineItem::new(1.5, 1.5)]);
        assert_eq!(got(&tl), [0.0, 1.5, 2.0, 3.0]);
    }

    #[test]
    fn timeline_get_index() {
        let mut tl = events();
        assert_eq!(PROBES.map(|t| tl.get_index(t)), [0, 1, 2, 3].map(Some));
        tl.add([TimelineItem::new(1.5, 1.5)]);
        assert_eq!(PROBES.map(|t| tl.get_index(t)), [0, 2, 3, 4].map(Some));
    }

    #[test]
    fn timeline_exact_times() {
        let tl = events();
        let ts = [0.0, 1.0, 2.0, 3.0];
        assert_eq!(ts.map(|t| tl.get_index(t)), [0, 1, 2, 3].map(Some));
        assert_eq!(ts.map(|t| tl.get(t).unwrap().value), [0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn timeline_from_items_sorts_by_time() {
        let tl = timeline([(2.0, 'c'), (0.0, 'a'), (1.0, 'b')]);
        assert_eq!(values(&tl), ['a', 'b', 'c']);
    }

    #[test]
    fn timeline_sort_is_stable() {
        let tl = timeline([(1.0, 'a'), (0.0, 'b'), (1.0, 'c'), (0.0, 'd')]);
        assert_eq!(values(&tl), ['b', 'd', 'a', 'c']);
    }

    #[test]
    fn timeline_merge_shifts_by_the_offset() {
        let mut tl = timeline([(0.0, 'a'), (2.0, 'b')]);
        tl.merge(4.0, &timeline([(0.0, 'c'), (1.0, 'd')]));
        assert_eq!(
            tl,
            timeline([(0.0, 'a'), (2.0, 'b'), (4.0, 'c'), (5.0, 'd')])
        );
    }

    #[test]
    fn timeline_before_the_first_item() {
        let tl = timeline([(1.0, 'a'), (2.0, 'b')]);
        assert_eq!(tl.get_index(0.0), None);
        assert_eq!(tl.get(0.0), None);
    }
}
