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
