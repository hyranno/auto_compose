import {Random} from 'reliable-random';
import * as vectorious from 'vectorious';

import {assertIsDefined, rangeIterator} from './util';
import {WeightedItem, WeightedRandom} from './util';


export abstract class MarkovChain<T> {
  protected random: Random;
  state: T;
  constructor(random: Random, initial_state: T) {
    this.random = random;
    this.state = initial_state;
  }
  abstract next(): T;
}


export type Condition<T> = {
  offset: number;
  state: T;
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
    super(random, initial_state);
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

  protected stateToIndex(state: T): number {
    return this.states.findIndex((s) => s == state);
  }
  protected stateToVector(state: T): vectorious.NDArray {
    return new vectorious.NDArray( this.states.map(s => (s==state)? 1 : 0), {shape: [1, this.states.length]});
  }
  protected vectorToWeightedItems(stateVec: vectorious.NDArray): WeightedItem<T>[] {
    return this.states.map( (s, si) => {
      let weight = stateVec.get(0, si);   assertIsDefined(weight);
      return {weight: weight, value: s};
    } );
  }

  protected calc_distribution(stateVec: vectorious.NDArray, isBackward: boolean =false): vectorious.NDArray {
    const mat = (isBackward? this.backward_matrix: this.transition_matrix);
    return stateVec.copy().multiply(mat);
  }
  protected calc_distribution_far(offset: number, stateVec: vectorious.NDArray, isBackward: boolean =false): vectorious.NDArray {
    let res = stateVec.copy();
    for (let i=0; i<offset; i++) {
      res = this.calc_distribution(res, isBackward);
    }
    return res;
  }
  protected calc_distribution_conditional(cond_past: Condition<T>, cond_forward: Condition<T>): vectorious.NDArray {
    const index_forward = this.stateToIndex(cond_forward.state);
    const v_past = this.stateToVector(cond_past.state);
    const v_forward = this.stateToVector(cond_forward.state);
    let dist_past = this.calc_distribution_far(cond_past.offset, v_past, false);
    let dist_forward = this.calc_distribution_far(cond_forward.offset, v_forward, true);
    let dist_pastToforward = this.calc_distribution_far(cond_past.offset + cond_forward.offset, v_past, false);
    const scale = this.stationary.get(0,index_forward) / dist_pastToforward.get(0,index_forward);
    const v = dist_past.copy().product(dist_forward).product(this.stationary.copy().map((v)=>1/v));
    const dist = v.copy().scale(scale);
    return dist;
  }

  protected transit(isBackward: boolean =false) {
    const vp = this.calc_distribution(this.stateToVector(this.state), isBackward);
    const generator = new WeightedRandom(this.random, this.vectorToWeightedItems(vp));
    return generator.get();
  }
  override next(isBackward: boolean =false): T {
    return this.state = this.transit(isBackward);
  }

  protected conditionalTransit(cond: {offset: number, state: T}, isBackward: boolean =false): T {
    const cond_past = (!isBackward)? {offset:1, state:this.state}: {offset:cond.offset-1, state:cond.state};
    const cond_forward = (!isBackward)? {offset:cond.offset-1, state:cond.state}: {offset:1, state:this.state};
    const vp = this.calc_distribution_conditional(cond_past, cond_forward);
    const generator = new WeightedRandom(this.random, this.vectorToWeightedItems(vp));
    return generator.get();
  }
  next_conditional(cond: {offset: number, state: T}, isBackward: boolean =false): T {
    return this.state = this.conditionalTransit(cond, isBackward);
  }
}
