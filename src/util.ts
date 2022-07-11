import {Random} from 'reliable-random';


export function* rangeIterator(start: number, end: number, tick: number){
  for (let t=start; t<end; t+=tick) {
    yield t;
  }
}


export type WeightedItem<T> = {
  weight: number;
  value: T;
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
  add(e: TimelineItem<T>) {
    this.items.push(e);
    this.items.sort((a, b) => a.t - b.t);
  }
  get(t: number): TimelineItem<T> {
    return this.items.slice().reverse().find((item) => item.t <= t); //.slice().reverse().find() == .findLast()
  }
}

export class MarkovChain<T> {
  random: Random;
  state: T;
  probabilityFunc: (state: T) => WeightedItem<T>[];
  constructor(random: Random, initial_state: T, probabilityFunc: (state: T) => WeightedItem<T>[]) {
    this.random = random;
    this.state = initial_state;
    this.probabilityFunc = probabilityFunc;
  }
  next(): T {
    let probability = this.probabilityFunc(this.state);
    let generator = new WeightedRandom(this.random, probability);
    this.state = generator.get();
    return this.state;
  }
}
