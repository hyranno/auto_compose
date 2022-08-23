import type { Component } from 'solid-js';
import {createSignal, For} from 'solid-js';

import * as rand from 'reliable-random';
import * as util from './util';

export type ClassWithUI = {ui: Component};
export type ClassUIProps<T extends ClassWithUI> = {instance: T};
export function ClassUI<T extends ClassWithUI> (props: ClassUIProps<T>) {
  return props.instance.ui(props);
}

export abstract class UiAdapter<T> {
  abstract get(): T;
  abstract set(v:T): T;
  abstract ui: Component;
}

export class InputBoundNumber extends UiAdapter<number> {
  get: ()=>number;
  set: (v: number)=>number;
  constructor(initialValue: number){
    super();
    [this.get, this.set] = createSignal(initialValue);
  }
  ui: Component = ()=>{
    return <input type="number" value={this.get()} onInput={(e)=>{this.set( parseFloat((e.target as HTMLInputElement).value) );}} />;
  };
}

export class Enum_bak<T extends {[key: string]: E}, E extends string> extends UiAdapter<E> {
  private enumObject: T;
  get: ()=>E;
  set: (v:E)=>E;
  constructor(enumObject: T, initialValue: E){
    super();
    this.enumObject = enumObject;
    [this.get, this.set] = createSignal(initialValue);
  }
  assertEnum(val: string): asserts val is E {
    if (Object.values(typeof this.enumObject).includes(val)) {
      throw new Error(
        `Expected 'val' to be defined, but received ${val}`
      );
    }
  }
  ui: Component = ()=>{
    return <select value={this.get()} onInput={e => {
      const val = e.currentTarget.value;
      this.assertEnum(val);
      this.set(val);
    }}>
    <For each={Object.values(this.enumObject)}>{
      e => <option value={e}>{e}</option>
    }</For>
    </select>;
  }
}

export class Enum<E extends string> extends UiAdapter<E> {
  private enumObject: {[key: number | string | symbol]: E};
  get: ()=>E;
  set: (v:E)=>E;
  constructor(enumObject: {[key: number | string | symbol]: E}, initialValue: E){
    super();
    this.enumObject = enumObject;
    [this.get, this.set] = createSignal(initialValue);
  }
  ui: Component = ()=>{
    return <select value={this.get()} onInput={e => {
      const val = e.currentTarget.value;
      util.assertEnum(this.enumObject, val);
      this.set(val);
    }}>
    <For each={Object.values(this.enumObject)}>{
      e => <option value={e}>{e}</option>
    }</For>
    </select>;
  }
}

export class RandomSeed extends UiAdapter<util.RandomSeed> {
  state: InputBoundNumber;
  sequence: InputBoundNumber;
  constructor(initState:number, initSequence:number) {
    super();
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

export class Smoothstep extends UiAdapter<util.SmoothstepParameters> {
  edge0: InputBoundNumber;
  edge1: InputBoundNumber;
  constructor(edge0: number, edge1: number) {
    super();
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
