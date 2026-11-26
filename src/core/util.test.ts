import { describe, test, expect } from 'vitest';
import * as vectorious from 'vectorious';
import {Random} from 'reliable-random';

import * as util from './util';
import {ScriptedRng} from './random';


test('smoothstep', ()=>{
  expect(util.smoothstep(0,1,-1)).toBeCloseTo(0);
  expect(util.smoothstep(0,1,0)).toBeCloseTo(0);
  expect(util.smoothstep(0,1,0.5)).toBeCloseTo(0.5);
  expect(util.smoothstep(0,1,1)).toBeCloseTo(1);
  expect(util.smoothstep(0,1,2)).toBeCloseTo(1);
});

describe('smoothstep (table)', () => {
  // [edge0, edge1, x, expected]
  const cases: [number, number, number, number][] = [
    [0.25, 3, 0.25, 0], [0.25, 3, 3, 1], [0.25, 3, 1.625, 0.5], [0.25, 3, 1, 0.18256949661908337],
    [-2, 1.4, 0.125, 0.68359375],
    // Reversed edges: the curve goes from 1 down to 0.
    [1, 0, 0.25, 0.84375],
  ];
  test.each(cases)('smoothstep(%d, %d, %d)', (e0, e1, x, expected) => {
    expect(util.smoothstep(e0, e1, x)).toBe(expected);
  });
  test('edge0 == edge1 is a step, except NaN exactly at the edge', () => {
    // (x - e) / 0 is -Infinity, +Infinity or NaN, and Math.min/max propagate NaN.
    // Note: Rust's f64::max(NaN, 0.0) returns 0.0, so the port must handle NaN explicitly.
    expect(util.smoothstep(1, 1, 0)).toBe(0);
    expect(util.smoothstep(1, 1, 2)).toBe(1);
    expect(util.smoothstep(1, 1, 1)).toBeNaN();
  });
});

test('modulo', () => {
  // [x, y, expected]
  const cases: [number, number, number][] = [
    [0, 12, 0], [5, 12, 5], [12, 12, 0], [13, 12, 1], [-1, 12, 11], [-12, 12, 0], [-13, 12, 11], [-1, 3, 2],
  ];
  expect(cases.map(([x, y]) => util.modulo(x, y))).toStrictEqual(cases.map(c => c[2]));
});

describe('TreeNode', () => {
  class Node extends util.TreeNode {
    constructor(public name: string) { super(); }
  }
  const node = (name: string, ...children: Node[]) => {
    const n = new Node(name);
    n.push(...children);
    return n;
  };
  //      r
  //    /   \
  //   a     b
  //  / \
  // c   d
  const tree = node('r', node('a', node('c'), node('d')), node('b'));
  test('flat is pre-order', () => {
    expect(tree.flat().map(n => n.name)).toStrictEqual(['r', 'a', 'c', 'd', 'b']);
  });
  test('leaves', () => {
    expect(tree.leaves().map(n => n.name)).toStrictEqual(['c', 'd', 'b']);
    expect(new Node('x').leaves().map(n => n.name)).toStrictEqual(['x']);
  });
  test('isLeaf', () => {
    expect([tree.isLeaf(), tree.children[1].isLeaf()]).toStrictEqual([false, true]);
  });
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

describe('WeightedRandom boundaries', () => {
  const pick = <T>(items: util.WeightedItem<T>[], values: number[]) => {
    const r = new util.WeightedRandom(new ScriptedRng(values), items);
    return values.map(() => r.get());
  };
  test('a draw equal to a cumulative ratio picks the lower item', () => {
    // cumulative ratios: 0.25, 0.75, 1
    const items = [{weight: 1, value: 'a'}, {weight: 2, value: 'b'}, {weight: 1, value: 'c'}];
    const draws = [0, 0.25, 0.2500001, 0.75, 0.7500001, 1 - 2 ** -32];
    expect(pick(items, draws)).toStrictEqual(['a', 'a', 'b', 'b', 'c', 'c']);
  });
  test('a zero-weight first item is picked by a draw of exactly 0', () => {
    const items = [{weight: 0, value: 'a'}, {weight: 1, value: 'b'}, {weight: 0, value: 'c'}];
    expect(pick(items, [0, 2 ** -32, 0.5, 1 - 2 ** -32])).toStrictEqual(['a', 'b', 'b', 'b']);
  });
  test('all weights zero falls back to the first item', () => {
    // total is 0, every ratio is NaN, findIndex fails and the index is clamped to 0.
    const items = [{weight: 0, value: 'a'}, {weight: 0, value: 'b'}];
    expect(pick(items, [0, 0.5, 0.99])).toStrictEqual(['a', 'a', 'a']);
  });
  test('cumulative ratios are left-to-right sums divided by the total', () => {
    // 0.1 + 0.2 = 0.30000000000000004 > 0.3, so a draw of 0.3 still picks the second item.
    const items = [{weight: 0.1, value: 'a'}, {weight: 0.2, value: 'b'}, {weight: 0.7, value: 'c'}];
    expect(pick(items, [0.3, 0.30000000000000004, 0.3000000000000001])).toStrictEqual(['b', 'b', 'c']);
  });
  test('consumes one value per get', () => {
    const rng = new ScriptedRng([0.1, 0.9]);
    const r = new util.WeightedRandom(rng, [{weight: 1, value: 0}, {weight: 1, value: 1}]);
    expect([r.get(), r.get()]).toStrictEqual([0, 1]);
    expect(rng.consumed).toBe(2);
  });
  test('mergeItems sums the weights of equal values, keeping first-seen order', () => {
    const merged = util.WeightedRandom.mergeItems([
      [{weight: 1, value: 'a'}, {weight: 2, value: 'b'}],
      [{weight: 3, value: 'c'}, {weight: 4, value: 'a'}],
    ]);
    expect(merged).toStrictEqual([{weight: 5, value: 'a'}, {weight: 2, value: 'b'}, {weight: 3, value: 'c'}]);
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
  test('exact times', () => {
    let timeline = util.Timeline.fromItems(events.slice());
    expect([0, 1, 2, 3].map(t => timeline.getIndex(t))).toStrictEqual([0, 1, 2, 3]);
    expect([0, 1, 2, 3].map(t => timeline.get(t).value)).toStrictEqual([0, 1, 2, 3]);
  });
  test('fromItems sorts by time', () => {
    let timeline = util.Timeline.fromItems([{t: 2, value: 'c'}, {t: 0, value: 'a'}, {t: 1, value: 'b'}]);
    expect(timeline.list().map(e => e.value)).toStrictEqual(['a', 'b', 'c']);
  });
  test('merge shifts the source by the offset', () => {
    let timeline = util.Timeline.fromItems([{t: 0, value: 'a'}, {t: 2, value: 'b'}]);
    timeline.merge(4, util.Timeline.fromItems([{t: 0, value: 'c'}, {t: 1, value: 'd'}]));
    expect(timeline.list()).toStrictEqual(
      [{t: 0, value: 'a'}, {t: 2, value: 'b'}, {t: 4, value: 'c'}, {t: 5, value: 'd'}]
    );
  });
  test('remove', () => {
    let timeline = util.Timeline.fromItems(events.slice());
    timeline.remove(1.5);
    expect(timeline.list().map(e => e.value)).toStrictEqual([0, 2, 3]);
  });
  // Known bug: before the first item, getIndex returns length and get throws.
  test('before the first item (known bug)', () => {
    let timeline = util.Timeline.fromItems([{t: 1, value: 'a'}, {t: 2, value: 'b'}]);
    expect(timeline.getIndex(0)).toBe(2);
    expect(() => timeline.get(0)).toThrow();
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
