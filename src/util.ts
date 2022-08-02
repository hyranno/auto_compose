import {Random} from 'reliable-random';
import * as vectorious from 'vectorious';

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
    return this.items.slice().reverse().findIndex((item) => item.t <= t);
  }
  remove(t: number) {
    this.items.splice(this.getIndex(t), 1);
  }
  list(): TimelineItem<T>[] {
    return this.items.slice();
  }
}

export class MarkovChain<T> {
  protected random: Random;
  state: T;
  protected probabilityFunc: (state: T) => WeightedItem<T>[];
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

export class MarkovChain_TimeHomo_FiniteState<T> extends MarkovChain<T> {
  private _states: T[];
  private _transition_matrix: vectorious.NDArray;
  private _backward_matrix: vectorious.NDArray;
  private _stationary: vectorious.NDArray;
  get states(): T[] {return this._states;}
  get transition_matrix(): vectorious.NDArray {return this._transition_matrix;}
  get backward_matrix(): vectorious.NDArray {return this._backward_matrix;}
  get stationary(): vectorious.NDArray {return this._stationary;}
  constructor(random: Random, initial_state: T, transition: Map<T, WeightedItem<T>[]>) {
    super(random, initial_state, s => {
      let res = transition.get(s);    assertIsDefined(res);
      return res;
    });
    this._states = [...transition.keys()];

    let matrix_data = [...rangeIterator(0, this.states.length)].map(
      (i) => [...rangeIterator(0, this.states.length)].map(
        (j) => {
          let items = transition.get( this.states[i] ); assertIsDefined(items);
          let item = items.find(wi => wi.value == this.states[j]); assertIsDefined(item);
          return item.weight;
        }
      )
    );
    this._transition_matrix = new vectorious.NDArray(matrix_data);
    this.init_stationary();
    this.init_backward_matrix();
  }
  private init_stationary() {
    const size = this.states.length;
    let righthand0 = vectorious.zeros(size);
    let lefthand0 = vectorious.eye(size).subtract(this.transition_matrix).transpose().copy();
    let righthand = righthand0.combine(vectorious.ones(1)).reshape(size+1, 1);
    let lefthand_data = [...rangeIterator(0,size+1)].map((i)=>
      [...rangeIterator(0,size+1)].map((j) =>
        (i<size && j<size)? lefthand0.get(i,j): 1
      )
    );
    let lefthand = new vectorious.NDArray(lefthand_data);
    this._stationary = lefthand.solve(righthand).slice(0,size,1).reshape(1, size);
  }
  private init_backward_matrix() {
    let matrix_data = [...rangeIterator(0, this.states.length)].map((i) =>
      [...rangeIterator(0, this.states.length)].map((j) => {
        ///P(S(t-1)=j | S(t)=i) = P(S(t)=i | S(t-1)=j) * P(S(t)=j) / P(S(t)=i)
        let pfji = this.transition_matrix.get(j,i);    assertIsDefined(pfji);
        let psi = this.stationary.get(0,i);   assertIsDefined(psi);
        let psj = this.stationary.get(0,j);   assertIsDefined(psj);
        let pbij = pfji * psj / psi;
        return pbij;
      })
    );
    this._backward_matrix = new vectorious.NDArray(matrix_data);
  }
  private transit(isBackward: boolean) {
    let matrix = isBackward? this.backward_matrix: this.transition_matrix;
    let v0 = this.stateToVector(this.state);
    let v1 = v0.copy().multiply(matrix);
    let probability: WeightedItem<T>[] = this.states.map( (s, si) => {
      let weight = v1.get(0, si);   assertIsDefined(weight);
      return {weight: weight, value: s};
    } );
    let generator = new WeightedRandom(this.random, probability);
    return generator.get();
  }
  override next(): T {
    return this.state = this.transit(false);
  }
  backward(): T {
    return this.state = this.transit(true);
  }
  private stateToVector(state: T): vectorious.NDArray {
    return new vectorious.NDArray( this.states.map(s => (s==state)? 1 : 0), {shape: [1, this.states.length]});
  }
  private conditionalTransit(isBackward: boolean, cond: {offset: number, state: T}): T {
    let matrix = isBackward? this.backward_matrix: this.transition_matrix;
    let matrix_back = isBackward? this.transition_matrix: this.backward_matrix;
    let v0 = this.stateToVector(this.state);
    let vc = this.stateToVector(cond.state);
    let vp0 = v0.copy().multiply(matrix);
    let vpc = vc.multiply( matrix_back.copy().pow(cond.offset) );
    let vpb = v0.multiply( matrix.copy().pow(cond.offset-1) );
    let vp = vp0.copy().product(vpc).product(vpb.copy().map(v => 1/v));
    let probability: WeightedItem<T>[] = this.states.map( (s, si) => {
      let weight = vp.get(0, si);   assertIsDefined(weight);
      weight = Number.isNaN(weight)? 0: weight;
      return {weight: weight, value: s};
    } );
    let generator = new WeightedRandom(this.random, probability);
    return generator.get();
  }
  next_conditional(cond: {offset: number, state: T}): T {
    return this.state = this.conditionalTransit(false, cond);
  }
  backward_conditional(cond: {offset: number, state: T}): T {
    return this.state = this.conditionalTransit(true, cond);
  }
}
