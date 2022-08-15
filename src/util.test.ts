import * as vectorious from 'vectorious';

import * as util from './util';
import * as markov from './markov';
import {Random} from 'reliable-random';


test('smoothstep', ()=>{
  expect(util.smoothstep(0,1,-1)).toBeCloseTo(0);
  expect(util.smoothstep(0,1,0)).toBeCloseTo(0);
  expect(util.smoothstep(0,1,0.5)).toBeCloseTo(0.5);
  expect(util.smoothstep(0,1,1)).toBeCloseTo(1);
  expect(util.smoothstep(0,1,2)).toBeCloseTo(1);
});


let seed0 = [754, 489];
let seed1 = [222, 9643];

describe('WeightedRandom', () => {
  describe('Random', () => {
    test('seed0', ()=>{
      let r = new Random(seed0[0], seed0[1]);
      expect(r.random()).toBeCloseTo(0.7883008599746972);
      expect(r.random()).toBeCloseTo(0.30768447695299983);
      expect(r.random()).toBeCloseTo(0.00584357138723135);
      expect(r.random()).toBeCloseTo(0.6822985948529094);
    });
    test('seed1', ()=>{
      let r = new Random(seed1[0], seed1[1]);
      expect(r.random()).toBeCloseTo(0.3066857950761914);
      expect(r.random()).toBeCloseTo(0.43649681424722075);
      expect(r.random()).toBeCloseTo(0.9942313127685338);
      expect(r.random()).toBeCloseTo(0.10204491158947349);
    });
  });
  let weighted = [
    {weight: 1, value: 0},
    {weight: 1, value: 1},
    {weight: 1, value: 2},
    {weight: 1, value: 3},
    {weight: 1, value: 4},
  ];
  test('seed0', () => {
    let r = new util.WeightedRandom(new Random(seed0[0], seed0[1]), weighted);
    expect([r.get(), r.get(), r.get(), r.get()]).toEqual([3,1,0,3]);
  });
  test('seed1', () => {
    let r = new util.WeightedRandom(new Random(seed1[0], seed1[1]), weighted);
    expect([r.get(), r.get(), r.get(), r.get()]).toEqual([1,2,4,0]);
  });
});

describe('Timeline', () => {
  let events = [
    {t: 0, value: 0},
    {t: 1, value: 1},
    {t: 2, value: 2},
    {t: 3, value: 3},
  ];
  test('get', () => {
    let timeline = util.Timeline.fromItems(events.slice());
    expect([timeline.get(0.6),timeline.get(1.6),timeline.get(2.6),timeline.get(3.6)].map(v=>v.value)).toEqual([0,1,2,3]);
    timeline.add({t:1.5, value:1.5});
    expect([timeline.get(0.6),timeline.get(1.6),timeline.get(2.6),timeline.get(3.6)].map(v=>v.value)).toEqual([0,1.5,2,3]);
  });
  test('getIndex', () => {
    let timeline = util.Timeline.fromItems(events.slice());
    expect(
      [timeline.getIndex(0.6), timeline.getIndex(1.6), timeline.getIndex(2.6), timeline.getIndex(3.6)]
    ).toEqual(
      [0,1,2,3]
    );
    timeline.add({t:1.5, value:1.5});
    expect(
      [timeline.getIndex(0.6), timeline.getIndex(1.6), timeline.getIndex(2.6), timeline.getIndex(3.6)]
    ).toEqual(
      [0,2,3,4]
    );
  });
});


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

describe("vectorious", ()=>{
  test("get() after transpose()", ()=>{
    let mat0 = (new vectorious.NDArray([[1,1], [0,0]])).transpose().copy(); /* need to copy to avoid bug? */
    let got = [[mat0.get(0,0), mat0.get(0,1)], [mat0.get(1,0), mat0.get(1,1)]];
    let expected = [[1,0], [1,0]];
    expect(got).toEqual(expected);
  });
});
