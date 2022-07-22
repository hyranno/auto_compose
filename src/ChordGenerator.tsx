import type { Component } from 'solid-js';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Cadence, Chord} from './tune';


type RootGeneratorParameters = {
  seed: util.RandomSeed;
  probabilities: [Cadence, util.WeightedItem<number>[]][];
};
class RootGenerator {
  private seed = new helper.RandomSeed(4649,459);
  private probabilities = new Map<Cadence, util.WeightedItem<number>[]>([
    [Cadence.T, [{value: 0, weight: 1}] ],
    [Cadence.D, [{value: 4, weight: 1}] ],
    [Cadence.S, [{value: 3, weight: 1}] ],
  ]);
  generate(tune: Tune): util.Timeline<number> {
    let rand = this.seed.createRandom();
    let events = tune.cadence.list().map( c => {
      let items = this.probabilities.get(c.value);
      util.assertIsDefined(items);
      return {
        t: c.t,
        value: (new util.WeightedRandom(rand, items)).get(),
      }
    } );
    return util.Timeline.fromItems(events);
  }
  ui: Component = () => {
    return <details><summary>ChordRootGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
    </details>;
  };
  setParameters(params: RootGeneratorParameters) {
    this.seed.set(params.seed);
    this.probabilities = new Map<Cadence, util.WeightedItem<number>[]>(params.probabilities);
  }
}

type ToneGeneratorParameters = {
  rand_tones: util.WeightedRandomParameters<number[]>;
};
class ToneGenerator {
  private seed = new helper.RandomSeed(4649,459);
  private tones: util.WeightedItem<number[]>[] = [
    {value:[0,2,4], weight: 1},
  ];
  generate(tune:Tune, roots: util.Timeline<number>): util.Timeline<Chord> {
    let rand = this.seed.createRandom();
    let rand_tones = new util.WeightedRandom(rand, this.tones);
    let events = roots.list().map( c => {return {
      t: c.t,
      value: new Chord(
        tune.scale,
        c.value,
        rand_tones.get()
      )
    }} );
    return util.Timeline.fromItems(events);
  }
  ui: Component = () => {
    return <details><summary>ChordToneGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
    </details>;
  };
  setParameters(params: ToneGeneratorParameters) {
    this.seed.set(params.rand_tones.seed);
    this.tones = params.rand_tones.items;
  }
}

type ModifierParameters = {}
export class Modifier {
  setParameters(_params: ModifierParameters){}
}


export type ChordGeneratorParameters = {
  root: RootGeneratorParameters;
  tone: ToneGeneratorParameters;
  modifier: ModifierParameters;
};
export class ChordGenerator {
  rootGen = new RootGenerator();
  toneGen = new ToneGenerator();
  modifier = new Modifier();
  generate(tune: Tune): util.Timeline<Chord> {
    return this.toneGen.generate(tune, this.rootGen.generate(tune));
  }
  ui: Component = () => {
    return <details><summary>ChordGenerator</summary>
      <helper.ClassUI instance={this.rootGen} />
      <helper.ClassUI instance={this.toneGen} />
    </details>;
  };
  setParameters(params: ChordGeneratorParameters) {
    this.rootGen.setParameters(params.root);
    this.toneGen.setParameters(params.tone);
    this.modifier.setParameters(params.modifier);
  }
}
