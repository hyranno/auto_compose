import type { Component } from 'solid-js';
import { Random } from 'reliable-random';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Cadence, Chord} from './tune';


namespace RootGenerator {
  export type Parameters = {
    seed: util.RandomSeed;
    probabilities: [Cadence, util.WeightedItem<number>[]][];
  };
  export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
    private seed = new helper.RandomSeed(4649,459);
    private probabilities: [Cadence, util.WeightedItem<number>[]][] = [
      [Cadence.T, [{value: 0, weight: 1}] ],
      [Cadence.D, [{value: 4, weight: 1}] ],
      [Cadence.S, [{value: 3, weight: 1}] ],
    ];
    get(): Parameters {
      return {
        seed: this.seed.get(),
        probabilities: this.probabilities
      };
    }
    set(params: Parameters): Parameters {
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
  export function generate(tune: Tune, params: Parameters): util.Timeline<number> {
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

namespace ToneGenerator {
  export type Parameters = {
    rand_tones: util.WeightedRandomParameters<number[]>;
  };
  export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
    private seed = new helper.RandomSeed(4649,459);
    private tones: util.WeightedItem<number[]>[] = [
      {value:[0,2,4], weight: 1},
    ];
    get(): Parameters {
      return {
        rand_tones: {
          seed: this.seed.get(),
          items: this.tones
        }
      };
    }
    set(params: Parameters): Parameters {
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
  export function generate(tune:Tune, roots: util.Timeline<number>, params: Parameters): util.Timeline<Chord> {
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

namespace ToneModifier {
  export type Parameters = {}
  export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
    get(): Parameters {
      return {}
    }
    set(params: Parameters): Parameters { return params; }
    ui: Component = () => { return <></>; }
  }
}


export type Parameters = {
  root: RootGenerator.Parameters;
  tone: ToneGenerator.Parameters;
  modifier: ToneModifier.Parameters;
};
export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
  private root = new RootGenerator.ParametersUiAdapter();
  private tone = new ToneGenerator.ParametersUiAdapter();
  private modifier = new ToneModifier.ParametersUiAdapter();
  get(): Parameters {
    return {
      root: this.root.get(),
      tone: this.tone.get(),
      modifier: this.modifier.get()
    };
  }
  set(params: Parameters): Parameters {
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
export function generate(tune: Tune, params: Parameters): util.Timeline<Chord> {
    return ToneGenerator.generate(tune, RootGenerator.generate(tune, params.root), params.tone);
}
