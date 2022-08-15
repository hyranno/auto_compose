import type { Component } from 'solid-js';
import { Random } from 'reliable-random';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Cadence, Chord} from './tune';


type RootGeneratorParameters = {
  seed: util.RandomSeed;
  probabilities: [Cadence, util.WeightedItem<number>[]][];
};
class RootGeneratorParametersUiAdapter extends helper.UiAdapter<RootGeneratorParameters> {
  private seed = new helper.RandomSeed(4649,459);
  private probabilities: [Cadence, util.WeightedItem<number>[]][] = [
    [Cadence.T, [{value: 0, weight: 1}] ],
    [Cadence.D, [{value: 4, weight: 1}] ],
    [Cadence.S, [{value: 3, weight: 1}] ],
  ];
  get(): RootGeneratorParameters {
    return {
      seed: this.seed.get(),
      probabilities: this.probabilities
    };
  }
  set(params: RootGeneratorParameters): RootGeneratorParameters {
    this.seed.set(params.seed);
    this.probabilities = params.probabilities;
    return params;
  }
  ui: Component = () => {
    return <details><summary>ChordRootGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
    </details>;
  };
}
class RootGenerator {
  generate(tune: Tune, params: RootGeneratorParameters): util.Timeline<number> {
    let rand = new Random(params.seed.state, params.seed.sequence);
    let probabilities = new Map(params.probabilities);
    let events = tune.cadence.list().map( c => {
      let items = probabilities.get(c.value);
      util.assertIsDefined(items);
      return {
        t: c.t,
        value: (new util.WeightedRandom(rand, items)).get(),
      }
    } );
    return util.Timeline.fromItems(events);
  }
}

type ToneGeneratorParameters = {
  rand_tones: util.WeightedRandomParameters<number[]>;
};
class ToneGeneratorParametersUiAdapter extends helper.UiAdapter<ToneGeneratorParameters> {
  private seed = new helper.RandomSeed(4649,459);
  private tones: util.WeightedItem<number[]>[] = [
    {value:[0,2,4], weight: 1},
  ];
  get(): ToneGeneratorParameters {
    return {
      rand_tones: {
        seed: this.seed.get(),
        items: this.tones
      }
    };
  }
  set(params: ToneGeneratorParameters): ToneGeneratorParameters {
    this.seed.set(params.rand_tones.seed);
    this.tones = params.rand_tones.items;
    return params;
  }
  ui: Component = () => {
    return <details><summary>ChordToneGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
    </details>;
  };
}
class ToneGenerator {
  generate(tune:Tune, roots: util.Timeline<number>, params: ToneGeneratorParameters): util.Timeline<Chord> {
    let rand = new Random(params.rand_tones.seed.state, params.rand_tones.seed.sequence);
    let rand_tones = new util.WeightedRandom(rand, params.rand_tones.items);
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
}

type ModifierParameters = {}
class ModifierParametersUiAdapter {
  get(): ModifierParameters {
    return {}
  }
  set(params: ModifierParameters) {
  }
}
class Modifier {
  setParameters(_params: ModifierParameters){}
}


export type ChordGeneratorParameters = {
  root: RootGeneratorParameters;
  tone: ToneGeneratorParameters;
  modifier: ModifierParameters;
};
export class ChordGeneratorParametersUiAdapter extends helper.UiAdapter<ChordGeneratorParameters> {
  private root = new RootGeneratorParametersUiAdapter();
  private tone = new ToneGeneratorParametersUiAdapter();
  private modifier = new ModifierParametersUiAdapter();
  get(): ChordGeneratorParameters {
    return {
      root: this.root.get(),
      tone: this.tone.get(),
      modifier: this.modifier.get()
    };
  }
  set(params: ChordGeneratorParameters): ChordGeneratorParameters {
    this.root.set(params.root);
    this.tone.set(params.tone);
    this.modifier.set(params.modifier);
    return params;
  }
  ui: Component = () => {
    return <details><summary>ChordGenerator</summary>
      <helper.ClassUI instance={this.root} />
      <helper.ClassUI instance={this.tone} />
    </details>;
  };
}
export class ChordGenerator {
  rootGen = new RootGenerator();
  toneGen = new ToneGenerator();
  modifier = new Modifier();
  generate(tune: Tune, params: ChordGeneratorParameters): util.Timeline<Chord> {
    return this.toneGen.generate(tune, this.rootGen.generate(tune, params.root), params.tone);
  }
}
