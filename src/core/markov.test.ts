import { describe, test, expect } from 'vitest';
import * as vectorious from 'vectorious';
import {Random} from 'reliable-random';

import * as util from './util';
import * as markov from './markov';


let seed0 = [754, 489];
let seed1 = [222, 9643];


describe('MarkovChain_TimeHomo_FiniteState', () => {
  type MarkovState = 0|1|2;
  let pMap = new Map<MarkovState, util.WeightedItem<MarkovState>[]>([
    [0, [{value: 0, weight: 0}, {value: 1, weight: 0.5}, {value: 2, weight: 0.5}] ],
    [1, [{value: 0, weight: 0.001}, {value: 1, weight: 0.999}, {value: 2, weight: 0.0}] ],
    [2, [{value: 0, weight: 0.001}, {value: 1, weight: 0.0}, {value: 2, weight: 0.999}] ],
  ]);
  test('transition', () => {
    let chain = new markov.MarkovChain_TimeHomo_FiniteState(
      new Random(seed0[0], seed0[1]),
      0,
      pMap
    );
    let got = [chain.next(), chain.next(), chain.next(), chain.next()];
    let expected = [2,2,2,2];
    expect(got).toEqual(expected);
  });
  test('conditional transition', () => {
    let chain = new markov.MarkovChain_TimeHomo_FiniteState(
      new Random(seed0[0], seed0[1]),
      0,
      pMap
    );
    let got = [
      chain.next_conditional({offset: 4, state:1}),
      chain.next_conditional({offset: 3, state:1}),
      chain.next_conditional({offset: 2, state:1}),
      chain.next_conditional({offset: 1, state:1})
    ];
    let expected = [1,1,1,1];
    expect(got).toEqual(expected);
  });
  test('conditional transition 2', () => {
    let chain = new markov.MarkovChain_TimeHomo_FiniteState(
      new Random(seed0[0], seed0[1]),
      0,
      pMap
    );
    let got = [
      chain.next(),
      chain.next_conditional({offset: 2, state:1}),
      chain.next_conditional({offset: 1, state:1}),
      chain.next_conditional({offset: 2, state:2}),
    ];
    let expected = [2,0,1,0];
    expect(got).toEqual(expected);
  });
});

describe('MarkovChain_TimeHomo_FiniteState matrices', () => {
  // The cadence transition probabilities of resources/TuneGenParams_default.json.
  const T = 'Tonic', D = 'Dominant', S = 'Subdominant';
  const cadence = new Map<string, util.WeightedItem<string>[]>([
    [T, [{value: T, weight: 0.1}, {value: D, weight: 0.3}, {value: S, weight: 0.6}]],
    [D, [{value: T, weight: 0.9}, {value: D, weight: 0.1}, {value: S, weight: 0.0}]],
    [S, [{value: T, weight: 0.4}, {value: D, weight: 0.5}, {value: S, weight: 0.1}]],
  ]);
  const chain = new markov.MarkovChain_TimeHomo_FiniteState(new Random(seed0[0], seed0[1]), T, cadence);
  const row = (m: vectorious.NDArray, i: number) => [0, 1, 2].map(j => m.get(i, j));
  const sum = (v: number[]) => v.reduce((a, b) => a + b);
  const expectClose = (got: number[], expected: number[]) => {
    expect(got.length).toBe(expected.length);
    got.forEach((v, i) => expect(v).toBeCloseTo(expected[i], 12));
  };

  test('states keep the order of the map', () => {
    expect(chain.states).toStrictEqual([T, D, S]);
  });
  test('stationary distribution', () => {
    const pi = row(chain.stationary, 0);
    expectClose(pi, [27 / 64, 19 / 64, 18 / 64]);
    expect(sum(pi)).toBeCloseTo(1, 12);
    // pi P = pi
    expectClose(row(chain.calc_distribution(chain.stationary), 0), pi);
  });
  test('backward matrix', () => {
    // B[i][j] = P[j][i] pi[j] / pi[i]
    expectClose(row(chain.backward_matrix, 0), [0.1, 57 / 90, 24 / 90]);
    expectClose(row(chain.backward_matrix, 1), [81 / 190, 0.1, 90 / 190]);
    expectClose(row(chain.backward_matrix, 2), [0.9, 0, 0.1]);
    for (const i of [0, 1, 2]) expect(sum(row(chain.backward_matrix, i))).toBeCloseTo(1, 12);
  });
  test('calc_distribution_far applies the matrix offset times', () => {
    const v = chain.stateToVector(D);
    expectClose(row(chain.calc_distribution_far(0, v), 0), [0, 1, 0]);
    expectClose(row(chain.calc_distribution_far(1, v), 0), [0.9, 0.1, 0]);
    expectClose(row(chain.calc_distribution_far(2, v), 0), [0.18, 0.28, 0.54]);
    expectClose(row(chain.calc_distribution_far(1, v, true), 0), [81 / 190, 0.1, 90 / 190]);
  });
  test('conditional distribution sums to 1', () => {
    // [past state, past offset, forward state, forward offset]
    const conds: [string, number, string, number][] = [
      [T, 1, D, 3], [T, 1, D, 1], [S, 1, T, 2], [D, 1, D, 4], [S, 2, S, 2],
    ];
    for (const [ps, po, fs, fo] of conds) {
      const d = chain.calc_distribution_conditional({offset: po, state: ps}, {offset: fo, state: fs});
      expect(sum(row(d, 0))).toBeCloseTo(1, 12);
    }
  });
  test('conditional distribution values', () => {
    // P(S(1) = j | S(0) = T, S(4) = D)
    const d = chain.calc_distribution_conditional({offset: 1, state: T}, {offset: 3, state: D});
    expectClose(row(d, 0), [0.08015267175572519, 0.33587786259541985, 0.5839694656488548]);
    // One step before the target: P(S(1) = j | S(0) = T, S(2) = D) is proportional to P[T][j] P[j][D].
    const d1 = chain.calc_distribution_conditional({offset: 1, state: T}, {offset: 1, state: D});
    const w = [0.1 * 0.3, 0.3 * 0.1, 0.6 * 0.5];
    expectClose(row(d1, 0), w.map(x => x / sum(w)));
  });
});
