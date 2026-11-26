//! Time-homogeneous Markov chain on finite states (`src/core/markov.ts`).
//!
//! Unlike the TypeScript class, the chain does not own its random number source:
//! each transition takes the [`Rng`] to draw from.

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
    backward_matrix: Matrix,
    stationary: Vec<f64>,
    pub state: T,
}

impl<T: Clone + PartialEq> MarkovChain<T> {
    /// `transition` lists, for each state, the weights of the next states.
    ///
    /// It is read like `new Map(transition)` in TypeScript: the states are ordered by their
    /// first appearance, and a later entry for the same state replaces the earlier one.
    /// Every state needs a weight for every state.
    pub fn new(initial_state: T, transition: &[(T, Vec<WeightedItem<T>>)]) -> Result<Self> {
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
            for (to, state) in states.iter().enumerate() {
                let item = items
                    .iter()
                    .find(|wi| wi.value == *state)
                    .ok_or(Error::MissingTransition { from, to })?;
                data.push(item.weight);
            }
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
            state: initial_state,
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

    /// The distribution one step forward (or backward) from `v`.
    pub fn calc_distribution(&self, v: &[f64], is_backward: bool) -> Vec<f64> {
        let mat = if is_backward {
            &self.backward_matrix
        } else {
            &self.transition_matrix
        };
        mat.left_mul(v)
    }

    /// The distribution `offset` steps forward (or backward) from `v`.
    pub fn calc_distribution_far(&self, offset: usize, v: &[f64], is_backward: bool) -> Vec<f64> {
        (0..offset).fold(v.to_vec(), |res, _| {
            self.calc_distribution(&res, is_backward)
        })
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
        let dist_past = self.calc_distribution_far(cond_past.offset, &v_past, false);
        let dist_forward = self.calc_distribution_far(cond_forward.offset, &v_forward, true);
        let dist_past_to_forward =
            self.calc_distribution_far(cond_past.offset + cond_forward.offset, &v_past, false);
        let scale = self.stationary[index_forward] / dist_past_to_forward[index_forward];
        Ok((0..self.states.len())
            .map(|i| dist_past[i] * dist_forward[i] * (1.0 / self.stationary[i]) * scale)
            .collect())
    }

    /// Moves one step forward (or backward), drawing one random number.
    pub fn next(&mut self, rng: &mut impl Rng, is_backward: bool) -> Result<T> {
        let vp = self.calc_distribution(&self.state_to_vector(&self.state), is_backward);
        self.pick(&vp, rng)
    }

    /// Moves one step forward (or backward) so that the chain reaches `cond.state`
    /// `cond.offset` steps from the current state, drawing one random number.
    pub fn next_conditional(
        &mut self,
        rng: &mut impl Rng,
        cond: &Condition<T>,
        is_backward: bool,
    ) -> Result<T> {
        let here = Condition::new(1, self.state.clone());
        let there = Condition::new(cond.offset.saturating_sub(1), cond.state.clone());
        let vp = if is_backward {
            self.calc_distribution_conditional(&there, &here)?
        } else {
            self.calc_distribution_conditional(&here, &there)?
        };
        self.pick(&vp, rng)
    }

    fn pick(&mut self, v: &[f64], rng: &mut impl Rng) -> Result<T> {
        let items = self.vector_to_weighted_items(v);
        let next = weighted_pick(&items, rng).ok_or(Error::NoItems)?.clone();
        self.state = next.clone();
        Ok(next)
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
        MarkovChain::new(0, &p).unwrap()
    }

    fn seed0() -> ReliableRandom {
        ReliableRandom::new(754, 489)
    }

    #[test]
    fn transition() {
        let mut chain = chain3();
        let mut r = seed0();
        let got: Vec<_> = (0..4).map(|_| chain.next(&mut r, false).unwrap()).collect();
        assert_eq!(got, [2, 2, 2, 2]);
    }

    #[test]
    fn conditional_transition() {
        let mut chain = chain3();
        let mut r = seed0();
        let got: Vec<_> = [4, 3, 2, 1]
            .map(|offset| {
                chain
                    .next_conditional(&mut r, &Condition::new(offset, 1), false)
                    .unwrap()
            })
            .to_vec();
        assert_eq!(got, [1, 1, 1, 1]);
    }

    #[test]
    fn conditional_transition_2() {
        let mut chain = chain3();
        let mut r = seed0();
        let mut got = vec![chain.next(&mut r, false).unwrap()];
        for (offset, state) in [(2, 1), (1, 1), (2, 2)] {
            let cond = Condition::new(offset, state);
            got.push(chain.next_conditional(&mut r, &cond, false).unwrap());
        }
        assert_eq!(got, [2, 0, 1, 0]);
    }

    /// The cadence transition probabilities of resources/TuneGenParams_default.json.
    fn cadence_chain() -> MarkovChain<Cadence> {
        let w = WeightedItem::new;
        let p = [
            (T, vec![w(0.1, T), w(0.3, D), w(0.6, S)]),
            (D, vec![w(0.9, T), w(0.1, D), w(0.0, S)]),
            (S, vec![w(0.4, T), w(0.5, D), w(0.1, S)]),
        ];
        MarkovChain::new(T, &p).unwrap()
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
        assert_close(&chain.calc_distribution(pi, false), pi);
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
        assert_close(&chain.calc_distribution_far(0, &v, false), &[0.0, 1.0, 0.0]);
        assert_close(&chain.calc_distribution_far(1, &v, false), &[0.9, 0.1, 0.0]);
        assert_close(
            &chain.calc_distribution_far(2, &v, false),
            &[0.18, 0.28, 0.54],
        );
        assert_close(
            &chain.calc_distribution_far(1, &v, true),
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
    fn errors() {
        let w = WeightedItem::new;
        let missing = [(T, vec![w(1.0, T)]), (D, vec![w(1.0, T), w(0.0, D)])];
        assert_eq!(
            MarkovChain::new(T, &missing).unwrap_err(),
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
