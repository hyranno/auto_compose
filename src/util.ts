import {Random} from 'reliable-random';

export function assertIsDefined<T>(val: T): asserts val is NonNullable<T> {
  if (val === undefined || val === null) {
    throw new Error(
      `Expected 'val' to be defined, but received ${val}`
    );
  }
}

export function* rangeIterator(start: number, end: number, tick: number = 1){
  for (let t=start; t<end; t+=tick) {
    yield t;
  }
}

export function modulo(x:number, y:number): number {
  return ((x % y) + y) % y;
}

export function step(edge:number, x:number): number {
  return (x<edge)? 0 : 1;
}
export function smoothstep(edge0: number, edge1: number, x: number): number {
  let xn = (x - edge0) / (edge1 - edge0);
  xn = Math.min(Math.max(xn, 0), 1);
  return xn*xn*(3-2*xn);
}
export type SmoothstepParameters = {
  edge0: number;
  edge1: number;
};

export class TreeNode {
  children: this[] = [];
  flat(): this[] {
    let res: this[] = [];
    res.push(this);
    res.push(...(this.children.map(n => n.flat())).flat());
    return res;
  }
  isLeaf(): boolean {
    return this.children.length == 0;
  }
  leaves(): this[] {
    return this.flat().filter(n => n.isLeaf());
  }
  push(...children: this[]): number {
    return this.children.push(...children);
  }
}

export type RandomSeed = {
  state: number;
  sequence: number;
};
export type WeightedItem<T> = {
  weight: number;
  value: T;
}
export type WeightedRandomParameters<T> = {
  items: WeightedItem<T>[];
  seed : RandomSeed;
}
export class WeightedRandom<T> {
  randGenerator: Random;
  items: WeightedItem<T>[];
  constructor(randGenerator: Random, items: WeightedItem<T>[]) {
    this.randGenerator = randGenerator;
    this.items = items;
  }
  get(): T {
    let randval = this.randGenerator.random();
    let sum_reducer = (prev: number, curr: number) => prev+curr;
    let weights = this.items.map((item)=>item.weight);
    let total = weights.reduce(sum_reducer);
    let accum = weights.map((_v, i) => weights.slice(0,i+1).reduce(sum_reducer) / total);
    let i_rev = accum.slice().findIndex((v) => randval <= v);
    return this.items[Math.max(i_rev, 0)].value;
  }
  static mergeItems<T>(src: WeightedItem<T>[][]): WeightedItem<T>[] {
    let res: WeightedItem<T>[] = [];
    src.flat().forEach((item) => {
      let foundIndex = res.findIndex((wi) => wi.value == item.value);
      if (foundIndex >= 0) {
        res[foundIndex].weight += item.weight;
      } else {
        res.push(item);
      }
    });
    return res;
  }
}

export type TimelineItem<T> = {
  t: number;
  value: T;
}
export class Timeline<T> {
  private items: TimelineItem<T>[] = [];
  static fromItems<T>(items: TimelineItem<T>[]): Timeline<T> {
    let res = new Timeline<T>();
    res.items = items;
    res.items.sort((a, b) => a.t - b.t);
    return res;
  }
  add(...e: TimelineItem<T>[]) {
    this.items.push(...e);
    this.items.sort((a, b) => a.t - b.t);
  }
  merge(offset: number, src: Timeline<T>) {
    this.add( ...src.list().map((e) => {return {t:e.t+offset, value:e.value}}) );
  }
  get(t: number): TimelineItem<T> {
    let res = this.items.slice().reverse().find((item) => item.t <= t); //.slice().reverse().find() == .findLast()
    assertIsDefined(res);
    return res;
  }
  getIndex(t: number): number {
    return this.items.length-1 - this.items.slice().reverse().findIndex((item) => item.t <= t);
  }
  remove(t: number) {
    this.items.splice(this.getIndex(t), 1);
  }
  list(): TimelineItem<T>[] {
    return this.items.slice();
  }
}
