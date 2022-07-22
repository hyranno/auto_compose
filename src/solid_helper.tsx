import type { Component } from 'solid-js';
import {createSignal} from 'solid-js';

import * as rand from 'reliable-random';
import * as util from './util';

export type ClassWithUI = {ui: Component};
export type ClassUIProps<T extends ClassWithUI> = {instance: T};
export function ClassUI<T extends ClassWithUI> (props: ClassUIProps<T>) {
  return props.instance.ui(props);
}

export class InputBoundNumber {
  get: ()=>number;
  set: (v: number)=>number;
  constructor(initialValue: number){
    [this.get, this.set] = createSignal(initialValue);
  }
  ui: Component = ()=>{
    return <input type="number" value={this.get()} onInput={(e)=>{this.set( parseFloat((e.target as HTMLInputElement).value) );}} />;
  };
}

export class RandomSeed {
  state: InputBoundNumber;
  sequence: InputBoundNumber;
  constructor(initState:number, initSequence:number) {
    this.state = new InputBoundNumber(initState);
    this.sequence = new InputBoundNumber(initSequence);
  }
  ui: Component = ()=>{
    return <>
      <ClassUI instance={this.state} />
      <ClassUI instance={this.sequence} />
    </>;
  }
  createRandom(): rand.Random {
    return new rand.Random(this.state.get(), this.sequence.get());
  }
  get(): util.RandomSeed {
    return {state: this.state.get(), sequence: this.sequence.get()};
  }
  set(seeds: util.RandomSeed): util.RandomSeed {
    return {state: this.state.set(seeds.state), sequence: this.sequence.set(seeds.sequence)};
  }
}

export class Smoothstep {
  edge0: InputBoundNumber;
  edge1: InputBoundNumber;
  constructor(edge0: number, edge1: number) {
    this.edge0 = new InputBoundNumber(edge0);
    this.edge1 = new InputBoundNumber(edge1);
  }
  ui: Component = ()=>{
    return <>
      <ClassUI instance={this.edge0} />
      <ClassUI instance={this.edge1} />
    </>;
  }
  calc(x: number): number {
    return util.smoothstep(this.edge0.get(), this.edge1.get(), x);
  }
  get(): util.SmoothstepParameters {
    return {edge0: this.edge0.get(), edge1: this.edge1.get()};
  }
  set(params: util.SmoothstepParameters): util.SmoothstepParameters {
    return {edge0: this.edge0.set(params.edge0), edge1: this.edge1.set(params.edge1)};
  }
}
