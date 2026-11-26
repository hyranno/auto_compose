//! Small numeric helpers, weighted random choice and timelines (`src/core/util.ts`).

use serde::{Deserialize, Serialize};

use crate::random::Rng;

/// `((x % y) + y) % y`, the same expression as the TypeScript version.
///
/// It is not replaced with `rem_euclid`, whose result differs for some floating point inputs.
pub fn modulo(x: f64, y: f64) -> f64 {
    ((x % y) + y) % y
}

/// Hermite interpolation between `edge0` and `edge1`, clamped to `[0, 1]`.
///
/// As in TypeScript, `edge0 == edge1` gives a step function that is `NaN` exactly at the edge
/// (`f64::clamp` propagates `NaN` like `Math.min` / `Math.max`, unlike `f64::max`).
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let xn = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    xn * xn * (3.0 - 2.0 * xn)
}

/// The edges of a [`smoothstep`] (`SmoothstepParameters` in TypeScript).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct SmoothstepParams {
    pub edge0: f64,
    pub edge1: f64,
}

impl SmoothstepParams {
    pub fn eval(&self, x: f64) -> f64 {
        smoothstep(self.edge0, self.edge1, x)
    }
}

/// `start, start + tick, ...` while the value is less than `end`.
///
/// The value is accumulated by addition, like the `for` loop of `rangeIterator` in TypeScript.
/// It does not end if `tick` is not positive (and `start < end`).
pub fn range_step(start: f64, end: f64, tick: f64) -> impl Iterator<Item = f64> {
    std::iter::successors(Some(start), move |t| Some(t + tick)).take_while(move |&t| t < end)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
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
/// Reproduces `WeightedRandom.get` of the TypeScript version exactly:
/// the cumulative ratios are left-to-right sums divided by the total, the first ratio that is
/// not less than the draw is chosen, and the first item is chosen when there is none
/// (for example when every weight is 0 and the ratios are `NaN`).
///
/// Returns `None` if `items` is empty (the TypeScript version throws). A value is drawn anyway.
pub fn weighted_pick<'a, T>(items: &'a [WeightedItem<T>], rng: &mut impl Rng) -> Option<&'a T> {
    let randval = rng.random();
    let first = items.first()?;
    let total = items[1..]
        .iter()
        .fold(first.weight, |acc, w| acc + w.weight);
    let mut accum = first.weight;
    let index = items
        .iter()
        .enumerate()
        .position(|(i, item)| {
            if i > 0 {
                accum += item.weight;
            }
            randval <= accum / total
        })
        .unwrap_or(0);
    Some(&items[index].value)
}

/// Sums the weights of items with equal values, keeping the order in which values first appear.
pub fn merge_weighted_items<T: PartialEq + Clone>(
    src: &[Vec<WeightedItem<T>>],
) -> Vec<WeightedItem<T>> {
    let mut res: Vec<WeightedItem<T>> = Vec::new();
    for item in src.iter().flatten() {
        match res.iter_mut().find(|wi| wi.value == item.value) {
            Some(found) => found.weight += item.weight,
            None => res.push(item.clone()),
        }
    }
    res
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct TimelineItem<T> {
    pub t: f64,
    pub value: T,
}

impl<T> TimelineItem<T> {
    pub fn new(t: f64, value: T) -> Self {
        Self { t, value }
    }
}

/// Items sorted by time. Items with equal times keep their insertion order (a stable sort,
/// like JavaScript's `Array.prototype.sort`).
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

    pub fn add(&mut self, items: impl IntoIterator<Item = TimelineItem<T>>) {
        self.items.extend(items);
        self.items
            .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
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

    /// The last item whose time is not after `t`. `None` before the first item
    /// (the TypeScript version throws).
    pub fn get(&self, t: f64) -> Option<&TimelineItem<T>> {
        self.get_index(t).map(|i| &self.items[i])
    }

    /// The index of [`Timeline::get`]. `None` before the first item
    /// (the TypeScript version returned the length).
    pub fn get_index(&self, t: f64) -> Option<usize> {
        self.items.iter().rposition(|item| item.t <= t)
    }

    /// Removes the item at [`Timeline::get_index`] (nothing before the first item).
    pub fn remove(&mut self, t: f64) {
        if let Some(i) = self.get_index(t) {
            self.items.remove(i);
        }
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

    // Ported from src/core/util.test.ts.

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
    fn smoothstep_equal_edges_is_a_step_except_nan_at_the_edge() {
        assert_eq!(smoothstep(1.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(1.0, 1.0, 2.0), 1.0);
        assert!(smoothstep(1.0, 1.0, 1.0).is_nan());
    }

    #[test]
    fn modulo_table() {
        let cases = [
            (0.0, 12.0, 0.0),
            (5.0, 12.0, 5.0),
            (12.0, 12.0, 0.0),
            (13.0, 12.0, 1.0),
            (-1.0, 12.0, 11.0),
            (-12.0, 12.0, 0.0),
            (-13.0, 12.0, 11.0),
            (-1.0, 3.0, 2.0),
        ];
        for (x, y, expected) in cases {
            assert_eq!(modulo(x, y), expected, "modulo({x}, {y})");
        }
    }

    #[test]
    fn range_step_accumulates() {
        assert_eq!(
            range_step(0.0, 4.0, 1.0).collect::<Vec<_>>(),
            [0.0, 1.0, 2.0, 3.0]
        );
        assert_eq!(range_step(0.0, 3.0, 2.0).collect::<Vec<_>>(), [0.0, 2.0]);
        assert_eq!(range_step(0.0, 0.0, 1.0).count(), 0);
        // Accumulated, not multiplied: 0.1 + 0.1 + 0.1 = 0.30000000000000004.
        assert_eq!(
            range_step(0.0, 0.35, 0.1).collect::<Vec<_>>(),
            [0.0, 0.1, 0.2, 0.30000000000000004]
        );
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
    fn weighted_pick_draw_equal_to_a_ratio_picks_the_lower_item() {
        // cumulative ratios: 0.25, 0.75, 1
        let items = [
            WeightedItem::new(1.0, 'a'),
            WeightedItem::new(2.0, 'b'),
            WeightedItem::new(1.0, 'c'),
        ];
        let draws = [0.0, 0.25, 0.2500001, 0.75, 0.7500001, 1.0 - 2f64.powi(-32)];
        assert_eq!(
            pick_scripted(&items, &draws),
            ['a', 'a', 'b', 'b', 'c', 'c']
        );
    }

    #[test]
    fn weighted_pick_zero_weight_first_item_is_picked_by_zero() {
        let items = [
            WeightedItem::new(0.0, 'a'),
            WeightedItem::new(1.0, 'b'),
            WeightedItem::new(0.0, 'c'),
        ];
        let draws = [0.0, 2f64.powi(-32), 0.5, 1.0 - 2f64.powi(-32)];
        assert_eq!(pick_scripted(&items, &draws), ['a', 'b', 'b', 'b']);
    }

    #[test]
    fn weighted_pick_all_zero_falls_back_to_the_first_item() {
        let items = [WeightedItem::new(0.0, 'a'), WeightedItem::new(0.0, 'b')];
        assert_eq!(pick_scripted(&items, &[0.0, 0.5, 0.99]), ['a', 'a', 'a']);
    }

    #[test]
    fn weighted_pick_ratios_are_left_to_right_sums() {
        // 0.1 + 0.2 = 0.30000000000000004 > 0.3, so a draw of 0.3 still picks the second item.
        let items = [
            WeightedItem::new(0.1, 'a'),
            WeightedItem::new(0.2, 'b'),
            WeightedItem::new(0.7, 'c'),
        ];
        let draws = [0.3, 0.30000000000000004, 0.3000000000000001];
        assert_eq!(pick_scripted(&items, &draws), ['b', 'b', 'c']);
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
        assert_eq!(weighted_pick::<i32>(&[], &mut rng), None);
        assert_eq!(rng.consumed(), 1);
    }

    #[test]
    fn merge_weighted_items_sums_equal_values() {
        let merged = merge_weighted_items(&[
            vec![WeightedItem::new(1.0, 'a'), WeightedItem::new(2.0, 'b')],
            vec![WeightedItem::new(3.0, 'c'), WeightedItem::new(4.0, 'a')],
        ]);
        assert_eq!(
            merged,
            [
                WeightedItem::new(5.0, 'a'),
                WeightedItem::new(2.0, 'b'),
                WeightedItem::new(3.0, 'c'),
            ]
        );
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
    fn timeline_remove() {
        let mut tl = events();
        tl.remove(1.5);
        assert_eq!(values(&tl), [0.0, 2.0, 3.0]);
    }

    #[test]
    fn timeline_before_the_first_item() {
        let mut tl = timeline([(1.0, 'a'), (2.0, 'b')]);
        assert_eq!(tl.get_index(0.0), None);
        assert_eq!(tl.get(0.0), None);
        tl.remove(0.0);
        assert_eq!(tl.len(), 2);
    }
}
