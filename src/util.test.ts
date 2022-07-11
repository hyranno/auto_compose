import * as util from './util';
import {Random} from 'reliable-random';


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
  test('', () => {
    let events = [
      {t: 0, value: 0},
      {t: 1, value: 1},
      {t: 2, value: 2},
      {t: 3, value: 3},
    ];
    let timeline = util.Timeline.fromItems(events);
    expect([timeline.get(0.6),timeline.get(1.6),timeline.get(2.6),timeline.get(3.6)].map(v=>v.value)).toEqual([0,1,2,3]);
    timeline.add({t:1.5, value:1.5});
    expect([timeline.get(0.6),timeline.get(1.6),timeline.get(2.6),timeline.get(3.6)].map(v=>v.value)).toEqual([0,1.5,2,3]);
  });
});

describe('MarkovChain', () => {
  test('', () => {
    type MarkovItem = {t:number, val:0|1|2};
    let pFunc = (s: MarkovItem) => [
      {weight: 1, value:{t:s.t+1, val:0}},
      {weight: 1, value:{t:s.t+1, val:1}},
      {weight: 1, value:{t:s.t+1, val:2}},
    ];
    let chain = new util.MarkovChain(
      new Random(seed0[0], seed0[1]),
      {t:0, val:0},
      pFunc
    );
    let got = [chain.next(), chain.next(), chain.next(), chain.next()];
    let expected = [{t:1, val:2}, {t:2, val:0}, {t:3, val:0}, {t:4, val:2}];
    expect(got).toEqual(expected);
  });
});
