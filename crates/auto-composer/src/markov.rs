//! Time-homogeneous Markov chain on finite states.
//!
//! The chain holds only the probabilities. The state to move from and the [`Rng`] to draw
//! with are given to each draw.

use crate::error::{Error, Result};
use crate::linalg::Matrix;
use crate::random::Rng;
use crate::util::{WeightedItem, weighted_pick};

/// "The chain is in `state` after `offset` steps."
#[derive(Debug, Clone, PartialEq)]
pub struct Condition<T> {
    pub offset: usize,
    pub state: T,
}

impl<T> Condition<T> {
    pub fn new(offset: usize, state: T) -> Self {
        Self { offset, state }
    }
}

#[derive(Debug, Clone)]
pub struct MarkovChain<T> {
    states: Vec<T>,
    transition_matrix: Matrix,
}

impl<T: Clone + PartialEq> MarkovChain<T> {
    /// `transition` lists, for each state, the weights of the next states.
    /// The weights are ratios: each row is divided by its sum to make the transition
    /// probabilities. A row with a negative or non-finite weight, or whose sum is 0, is an error.
    ///
    /// It is read like `new Map(transition)` in TypeScript: the states are ordered by their
    /// first appearance, and a later entry for the same state replaces the earlier one.
    /// Every state needs a weight for every state.
    pub fn new(transition: &[(T, Vec<WeightedItem<T>>)]) -> Result<Self> {
        let mut entries: Vec<(&T, &[WeightedItem<T>])> = Vec::new();
        for (state, items) in transition {
            match entries.iter_mut().find(|(s, _)| *s == state) {
                Some(entry) => entry.1 = items,
                None => entries.push((state, items)),
            }
        }
        let states: Vec<T> = entries.iter().map(|(s, _)| (*s).clone()).collect();
        let n = states.len();
        let mut data = Vec::with_capacity(n * n);
        for (from, (_, items)) in entries.iter().enumerate() {
            let row = states
                .iter()
                .enumerate()
                .map(|(to, state)| {
                    items
                        .iter()
                        .find(|wi| wi.value == *state)
                        .map(|item| item.weight)
                        .ok_or(Error::MissingTransition { from, to })
                })
                .collect::<Result<Vec<f64>>>()?;
            let sum: f64 = row.iter().sum();
            let valid = row.iter().all(|w| (0.0..f64::INFINITY).contains(w));
            if !valid || sum == 0.0 || !sum.is_finite() {
                return Err(Error::InvalidTransition { from });
            }
            data.extend(row.iter().map(|w| w / sum));
        }
        let transition_matrix = Matrix::from_fn(n, n, |i, j| data[i * n + j]);
        Ok(Self {
            states,
            transition_matrix,
        })
    }

    pub fn states(&self) -> &[T] {
        &self.states
    }

    pub fn transition_matrix(&self) -> &Matrix {
        &self.transition_matrix
    }

    pub fn state_to_index(&self, state: &T) -> Option<usize> {
        self.states.iter().position(|s| s == state)
    }

    /// The indicator vector of `state` (all zeros if it is not a state).
    pub fn state_to_vector(&self, state: &T) -> Vec<f64> {
        self.states
            .iter()
            .map(|s| if s == state { 1.0 } else { 0.0 })
            .collect()
    }

    pub fn vector_to_weighted_items(&self, v: &[f64]) -> Vec<WeightedItem<T>> {
        self.states
            .iter()
            .zip(v)
            .map(|(s, &w)| WeightedItem::new(w, s.clone()))
            .collect()
    }

    /// The distribution one step after `v`.
    pub fn calc_distribution(&self, v: &[f64]) -> Vec<f64> {
        self.transition_matrix.left_mul(v)
    }

    /// The distribution `offset` steps after `v`.
    pub fn calc_distribution_far(&self, offset: usize, v: &[f64]) -> Vec<f64> {
        (0..offset).fold(v.to_vec(), |res, _| self.calc_distribution(&res))
    }

    /// The probability of being in `state` `offset` steps after each state.
    pub fn calc_reach_probabilities(&self, offset: usize, state: &T) -> Vec<f64> {
        let v = self.state_to_vector(state);
        (0..offset).fold(v, |res, _| self.transition_matrix.mul_vec(&res))
    }

    /// The distribution of the state `cond_past.offset` steps after `cond_past.state`,
    /// given that the chain is in `cond_forward.state` `cond_forward.offset` steps after that.
    ///
    /// The values are not finite if the condition cannot be met.
    pub fn calc_distribution_conditional(
        &self,
        cond_past: &Condition<T>,
        cond_forward: &Condition<T>,
    ) -> Result<Vec<f64>> {
        if self.state_to_index(&cond_forward.state).is_none() {
            return Err(Error::UnknownState);
        }
        let v_past = self.state_to_vector(&cond_past.state);
        let dist_past = self.calc_distribution_far(cond_past.offset, &v_past);
        let reach = self.calc_reach_probabilities(cond_forward.offset, &cond_forward.state);
        let joint: Vec<f64> = dist_past.iter().zip(&reach).map(|(p, r)| p * r).collect();
        let sum: f64 = joint.iter().sum();
        Ok(joint.iter().map(|w| w / sum).collect())
    }

    /// The distribution of the state one step after `from`, given that the chain is in
    /// `goal.state` `goal.offset` steps after `from`. When the goal cannot be reached, the
    /// condition is dropped: the distribution one step after `from`.
    pub fn next_distribution(&self, from: &T, goal: &Condition<T>) -> Result<Vec<f64>> {
        let here = Condition::new(1, from.clone());
        let there = Condition::new(goal.offset.saturating_sub(1), goal.state.clone());
        let dist = self.calc_distribution_conditional(&here, &there)?;
        if is_distribution(&dist) {
            Ok(dist)
        } else {
            Ok(self.calc_distribution(&self.state_to_vector(from)))
        }
    }

    /// Draws the state of [`MarkovChain::next_distribution`], drawing one random number.
    pub fn sample_next(&self, from: &T, goal: &Condition<T>, rng: &mut impl Rng) -> Result<T> {
        let dist = self.next_distribution(from, goal)?;
        self.pick(&dist, rng)
    }

    /// Draws a state with the weights `v`.
    fn pick(&self, v: &[f64], rng: &mut impl Rng) -> Result<T> {
        let items = self.vector_to_weighted_items(v);
        Ok(weighted_pick(&items, rng)?.clone())
    }
}

/// Whether the weights can be drawn from: finite, non-negative and not all 0.
fn is_distribution(v: &[f64]) -> bool {
    v.iter().all(|w| (0.0..f64::INFINITY).contains(w)) && v.iter().any(|&w| w > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_util::seeded;
    use crate::tune::Cadence::{self, D, S, T};

    fn chain3() -> MarkovChain<u8> {
        let w = WeightedItem::new;
        let p = [
            (0, vec![w(0.0, 0), w(0.5, 1), w(0.5, 2)]),
            (1, vec![w(0.001, 0), w(0.999, 1), w(0.0, 2)]),
            (2, vec![w(0.001, 0), w(0.0, 1), w(0.999, 2)]),
        ];
        MarkovChain::new(&p).unwrap()
    }

    /// Draws the states one at a time, each from the previous one (the first from 0).
    fn walk(
        chain: &MarkovChain<u8>,
        rng: &mut impl Rng,
        steps: impl IntoIterator<Item = Step>,
    ) -> Vec<u8> {
        let mut state = 0;
        steps
            .into_iter()
            .map(|step| {
                state = match step {
                    Step::Free => {
                        let dist = chain.calc_distribution(&chain.state_to_vector(&state));
                        chain.pick(&dist, rng).unwrap()
                    }
                    Step::To(goal) => chain.sample_next(&state, &goal, rng).unwrap(),
                };
                state
            })
            .collect()
    }

    #[derive(Clone)]
    enum Step {
        Free,
        To(Condition<u8>),
    }

    /// Walks with each seed and checks that every step has a positive probability and that
    /// every goal is met.
    fn check_walks(steps: &[Step]) {
        let chain = chain3();
        for s in 0..50 {
            let mut rng = seeded(s, 1);
            let walk = walk(&chain, &mut rng, steps.iter().cloned());
            let mut prev = 0;
            for (i, (step, &state)) in steps.iter().zip(&walk).enumerate() {
                let p = chain
                    .transition_matrix()
                    .get(usize::from(prev), usize::from(state));
                assert!(p > 0.0, "seed {s}: {walk:?}");
                if let Step::To(goal) = step {
                    // The goal is met at the step `i + goal.offset - 1`, if the walk gets there.
                    if let Some(&at) = walk.get(i + goal.offset - 1) {
                        assert_eq!(at, goal.state, "seed {s}: {walk:?}");
                    }
                }
                prev = state;
            }
        }
    }

    #[test]
    fn transition() {
        check_walks(&[Step::Free, Step::Free, Step::Free, Step::Free]);
    }

    #[test]
    fn conditional_transition() {
        check_walks(&[4, 3, 2, 1].map(|offset| Step::To(Condition::new(offset, 1))));
    }

    #[test]
    fn conditional_transition_2() {
        check_walks(&[
            Step::Free,
            Step::To(Condition::new(2, 1)),
            Step::To(Condition::new(1, 1)),
            Step::To(Condition::new(2, 2)),
        ]);
    }

    /// The cadence transition probabilities of resources/TuneGenParams_default.json.
    fn cadence_chain() -> MarkovChain<Cadence> {
        let w = WeightedItem::new;
        let p = [
            (T, vec![w(0.1, T), w(0.3, D), w(0.6, S)]),
            (D, vec![w(0.9, T), w(0.1, D), w(0.0, S)]),
            (S, vec![w(0.4, T), w(0.5, D), w(0.1, S)]),
        ];
        MarkovChain::new(&p).unwrap()
    }

    fn assert_close(got: &[f64], expected: &[f64]) {
        assert_eq!(got.len(), expected.len());
        for (g, e) in got.iter().zip(expected) {
            assert!((g - e).abs() < 1e-12, "{got:?} != {expected:?}");
        }
    }

    fn sum(v: &[f64]) -> f64 {
        v.iter().sum()
    }

    #[test]
    fn states_keep_the_order_of_the_map() {
        assert_eq!(cadence_chain().states(), [T, D, S]);
    }

    #[test]
    fn calc_distribution_far_applies_the_matrix_offset_times() {
        let chain = cadence_chain();
        let v = chain.state_to_vector(&D);
        assert_close(&chain.calc_distribution_far(0, &v), &[0.0, 1.0, 0.0]);
        assert_close(&chain.calc_distribution_far(1, &v), &[0.9, 0.1, 0.0]);
        assert_close(&chain.calc_distribution_far(2, &v), &[0.18, 0.28, 0.54]);
    }

    #[test]
    fn reach_probabilities_apply_the_matrix_from_the_right() {
        let chain = cadence_chain();
        assert_close(&chain.calc_reach_probabilities(0, &D), &[0.0, 1.0, 0.0]);
        assert_close(&chain.calc_reach_probabilities(1, &D), &[0.3, 0.1, 0.5]);
        // P[i][T] * 0.3 + P[i][D] * 0.1 + P[i][S] * 0.5
        assert_close(
            &chain.calc_reach_probabilities(2, &D),
            &[0.03 + 0.03 + 0.3, 0.27 + 0.01, 0.12 + 0.05 + 0.05],
        );
    }

    #[test]
    fn conditional_distribution_sums_to_1() {
        let chain = cadence_chain();
        // (past state, past offset, forward state, forward offset)
        let conds = [
            (T, 1, D, 3),
            (T, 1, D, 1),
            (S, 1, T, 2),
            (D, 1, D, 4),
            (S, 2, S, 2),
        ];
        for (ps, po, fs, fo) in conds {
            let d = chain
                .calc_distribution_conditional(&Condition::new(po, ps), &Condition::new(fo, fs))
                .unwrap();
            assert!((sum(&d) - 1.0).abs() < 1e-12, "{d:?}");
        }
    }

    #[test]
    fn conditional_distribution_values() {
        let chain = cadence_chain();
        // P(S(1) = j | S(0) = T, S(4) = D)
        let d = chain
            .calc_distribution_conditional(&Condition::new(1, T), &Condition::new(3, D))
            .unwrap();
        assert_close(
            &d,
            &[0.08015267175572519, 0.33587786259541985, 0.5839694656488548],
        );
        // One step before the target: proportional to P[T][j] P[j][D].
        let d1 = chain
            .calc_distribution_conditional(&Condition::new(1, T), &Condition::new(1, D))
            .unwrap();
        let w = [0.1 * 0.3, 0.3 * 0.1, 0.6 * 0.5];
        assert_close(&d1, &w.map(|x| x / sum(&w)));
    }

    // The values printed by the TypeScript implementation, up to rounding.
    #[test]
    fn known_values() {
        let chain = cadence_chain();
        // (past state, past offset, forward state, forward offset, expected)
        let cases = [
            (
                T,
                1,
                D,
                3,
                [0.08015267175572519, 0.33587786259541985, 0.5839694656488548],
            ),
            (
                S,
                1,
                T,
                2,
                [0.5925925925925927, 0.25641025641025644, 0.15099715099715105],
            ),
            (D, 1, D, 4, [0.9152542372881354, 0.0847457627118644, 0.0]),
            (
                S,
                2,
                S,
                2,
                [0.2596978358513679, 0.4850959575336872, 0.2552062066149448],
            ),
        ];
        for (ps, po, fs, fo, expected) in cases {
            let d = chain
                .calc_distribution_conditional(&Condition::new(po, ps), &Condition::new(fo, fs))
                .unwrap();
            assert_close(&d, &expected);
        }
    }

    #[test]
    fn next_distribution_drops_an_unreachable_goal() {
        let w = WeightedItem::new;
        // 0 is absorbing: 1 cannot be reached from 0.
        let p = [
            (0u8, vec![w(1.0, 0), w(0.0, 1)]),
            (1, vec![w(0.5, 0), w(0.5, 1)]),
        ];
        let chain = MarkovChain::new(&p).unwrap();
        let dist = chain.next_distribution(&0, &Condition::new(2, 1)).unwrap();
        assert_eq!(dist, [1.0, 0.0]);
        // Reachable: from 1 to 1 in 2 steps, the next state must be 1.
        let dist = chain.next_distribution(&1, &Condition::new(2, 1)).unwrap();
        assert_close(&dist, &[0.0, 1.0]);
    }

    #[test]
    fn weights_are_ratios() {
        let w = WeightedItem::new;
        let scaled = [
            (T, vec![w(1.0, T), w(3.0, D), w(6.0, S)]),
            (D, vec![w(90.0, T), w(10.0, D), w(0.0, S)]),
            (S, vec![w(0.8, T), w(1.0, D), w(0.2, S)]),
        ];
        let chain = MarkovChain::new(&scaled).unwrap();
        let expected = cadence_chain();
        for i in 0..3 {
            assert_close(
                chain.transition_matrix().row(i),
                expected.transition_matrix().row(i),
            );
        }
    }

    #[test]
    fn weights_that_are_not_ratios_are_errors() {
        let w = WeightedItem::new;
        for bad in [
            [0.0, 0.0],
            [-1.0, 2.0],
            [f64::NAN, 1.0],
            [f64::INFINITY, 1.0],
        ] {
            let p = [
                (T, vec![w(1.0, T), w(1.0, D)]),
                (D, vec![w(bad[0], T), w(bad[1], D)]),
            ];
            assert_eq!(
                MarkovChain::new(&p).unwrap_err(),
                Error::InvalidTransition { from: 1 },
                "{bad:?}"
            );
        }
    }

    #[test]
    fn errors() {
        let w = WeightedItem::new;
        let missing = [(T, vec![w(1.0, T)]), (D, vec![w(1.0, T), w(0.0, D)])];
        assert_eq!(
            MarkovChain::new(&missing).unwrap_err(),
            Error::MissingTransition { from: 0, to: 1 }
        );
        let chain = chain3();
        assert_eq!(
            chain
                .calc_distribution_conditional(&Condition::new(1, 0), &Condition::new(1, 9))
                .unwrap_err(),
            Error::UnknownState
        );
    }
}
