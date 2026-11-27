//! Time-homogeneous Markov chain on finite states (`src/core/markov.ts`).
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

/// The direction of a step of the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    /// By the time-reversed chain ([`MarkovChain::backward_matrix`]).
    Backward,
}

#[derive(Debug, Clone)]
pub struct MarkovChain<T> {
    states: Vec<T>,
    transition_matrix: Matrix,
    backward_matrix: Matrix,
    stationary: Vec<f64>,
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
        let stationary = stationary_distribution(&transition_matrix);
        // P(S(t-1)=j | S(t)=i) = P(S(t)=i | S(t-1)=j) * P(S(t)=j) / P(S(t)=i)
        let backward_matrix = Matrix::from_fn(n, n, |i, j| {
            transition_matrix.get(j, i) * stationary[j] / stationary[i]
        });
        Ok(Self {
            states,
            transition_matrix,
            backward_matrix,
            stationary,
        })
    }

    pub fn states(&self) -> &[T] {
        &self.states
    }

    pub fn transition_matrix(&self) -> &Matrix {
        &self.transition_matrix
    }

    pub fn backward_matrix(&self) -> &Matrix {
        &self.backward_matrix
    }

    pub fn stationary(&self) -> &[f64] {
        &self.stationary
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

    /// The distribution one step from `v`.
    pub fn calc_distribution(&self, v: &[f64], direction: Direction) -> Vec<f64> {
        let mat = match direction {
            Direction::Forward => &self.transition_matrix,
            Direction::Backward => &self.backward_matrix,
        };
        mat.left_mul(v)
    }

    /// The distribution `offset` steps from `v`.
    pub fn calc_distribution_far(
        &self,
        offset: usize,
        v: &[f64],
        direction: Direction,
    ) -> Vec<f64> {
        (0..offset).fold(v.to_vec(), |res, _| self.calc_distribution(&res, direction))
    }

    /// The distribution of the state `cond_past.offset` steps after `cond_past.state`,
    /// given that the chain is in `cond_forward.state` `cond_forward.offset` steps after that.
    pub fn calc_distribution_conditional(
        &self,
        cond_past: &Condition<T>,
        cond_forward: &Condition<T>,
    ) -> Result<Vec<f64>> {
        let index_forward = self
            .state_to_index(&cond_forward.state)
            .ok_or(Error::UnknownState)?;
        let v_past = self.state_to_vector(&cond_past.state);
        let v_forward = self.state_to_vector(&cond_forward.state);
        let dist_past = self.calc_distribution_far(cond_past.offset, &v_past, Direction::Forward);
        let dist_forward =
            self.calc_distribution_far(cond_forward.offset, &v_forward, Direction::Backward);
        let dist_past_to_forward = self.calc_distribution_far(
            cond_past.offset + cond_forward.offset,
            &v_past,
            Direction::Forward,
        );
        let scale = self.stationary[index_forward] / dist_past_to_forward[index_forward];
        Ok((0..self.states.len())
            .map(|i| dist_past[i] * dist_forward[i] * (1.0 / self.stationary[i]) * scale)
            .collect())
    }

    /// Draws the state one step after `from`, given that the chain is in `goal.state`
    /// `goal.offset` steps after `from`. Draws one random number.
    pub fn sample_next(&self, from: &T, goal: &Condition<T>, rng: &mut impl Rng) -> Result<T> {
        let here = Condition::new(1, from.clone());
        let there = Condition::new(goal.offset.saturating_sub(1), goal.state.clone());
        let dist = self.calc_distribution_conditional(&here, &there)?;
        self.pick(&dist, rng)
    }

    /// Draws a state with the weights `v`.
    fn pick(&self, v: &[f64], rng: &mut impl Rng) -> Result<T> {
        let items = self.vector_to_weighted_items(v);
        Ok(weighted_pick(&items, rng).ok_or(Error::NoItems)?.clone())
    }
}

/// Solves `(I - P)^T x = 0` with `sum(x) = 1`, as the TypeScript version does:
/// the system is extended to `(n+1) x (n+1)` with the last row and column all 1,
/// the right-hand side `[0, ..., 0, 1]`, and the last element of the solution is dropped.
fn stationary_distribution(p: &Matrix) -> Vec<f64> {
    let n = p.rows();
    let lefthand0 = Matrix::identity(n).sub(p).transpose();
    let lefthand = Matrix::from_fn(n + 1, n + 1, |i, j| {
        if i < n && j < n {
            lefthand0.get(i, j)
        } else {
            1.0
        }
    });
    let mut righthand = vec![0.0; n + 1];
    righthand[n] = 1.0;
    let mut x = lefthand.solve(&righthand);
    x.truncate(n);
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::ReliableRandom;
    use crate::tune::Cadence::{self, D, S, T};

    // Ported from src/core/markov.test.ts.

    fn chain3() -> MarkovChain<u8> {
        let w = WeightedItem::new;
        let p = [
            (0, vec![w(0.0, 0), w(0.5, 1), w(0.5, 2)]),
            (1, vec![w(0.001, 0), w(0.999, 1), w(0.0, 2)]),
            (2, vec![w(0.001, 0), w(0.0, 1), w(0.999, 2)]),
        ];
        MarkovChain::new(&p).unwrap()
    }

    fn seed0() -> ReliableRandom {
        ReliableRandom::new(754, 489)
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
                        let dist = chain
                            .calc_distribution(&chain.state_to_vector(&state), Direction::Forward);
                        chain.pick(&dist, rng).unwrap()
                    }
                    Step::To(goal) => chain.sample_next(&state, &goal, rng).unwrap(),
                };
                state
            })
            .collect()
    }

    enum Step {
        Free,
        To(Condition<u8>),
    }

    #[test]
    fn transition() {
        let got = walk(&chain3(), &mut seed0(), (0..4).map(|_| Step::Free));
        assert_eq!(got, [2, 2, 2, 2]);
    }

    #[test]
    fn conditional_transition() {
        let steps = [4, 3, 2, 1].map(|offset| Step::To(Condition::new(offset, 1)));
        assert_eq!(walk(&chain3(), &mut seed0(), steps), [1, 1, 1, 1]);
    }

    #[test]
    fn conditional_transition_2() {
        let steps = [
            Step::Free,
            Step::To(Condition::new(2, 1)),
            Step::To(Condition::new(1, 1)),
            Step::To(Condition::new(2, 2)),
        ];
        assert_eq!(walk(&chain3(), &mut seed0(), steps), [2, 0, 1, 0]);
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
    fn stationary_distribution_of_the_default_cadence() {
        let chain = cadence_chain();
        let pi = chain.stationary();
        assert_close(pi, &[27.0 / 64.0, 19.0 / 64.0, 18.0 / 64.0]);
        assert!((sum(pi) - 1.0).abs() < 1e-12);
        assert_close(&chain.calc_distribution(pi, Direction::Forward), pi);
    }

    #[test]
    fn backward_matrix() {
        let chain = cadence_chain();
        let b = chain.backward_matrix();
        assert_close(b.row(0), &[0.1, 57.0 / 90.0, 24.0 / 90.0]);
        assert_close(b.row(1), &[81.0 / 190.0, 0.1, 90.0 / 190.0]);
        assert_close(b.row(2), &[0.9, 0.0, 0.1]);
        for i in 0..3 {
            assert!((sum(b.row(i)) - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn calc_distribution_far_applies_the_matrix_offset_times() {
        let chain = cadence_chain();
        let v = chain.state_to_vector(&D);
        assert_close(
            &chain.calc_distribution_far(0, &v, Direction::Forward),
            &[0.0, 1.0, 0.0],
        );
        assert_close(
            &chain.calc_distribution_far(1, &v, Direction::Forward),
            &[0.9, 0.1, 0.0],
        );
        assert_close(
            &chain.calc_distribution_far(2, &v, Direction::Forward),
            &[0.18, 0.28, 0.54],
        );
        assert_close(
            &chain.calc_distribution_far(1, &v, Direction::Backward),
            &[81.0 / 190.0, 0.1, 90.0 / 190.0],
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

    // Printed with full precision from the TypeScript implementation (vectorious 6).
    #[test]
    fn matches_typescript_bit_for_bit() {
        let chain = cadence_chain();
        assert_eq!(chain.stationary(), [0.421875, 0.296875, 0.28125]);
        let b = chain.backward_matrix();
        assert_eq!(b.row(0), [0.1, 0.6333333333333334, 0.26666666666666666]);
        assert_eq!(b.row(1), [0.4263157894736842, 0.1, 0.47368421052631576]);
        assert_eq!(b.row(2), [0.8999999999999999, 0.0, 0.1]);
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
            assert_eq!(d, expected, "{ps:?} {po} {fs:?} {fo}");
        }
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
        assert_close(chain.stationary(), expected.stationary());
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
