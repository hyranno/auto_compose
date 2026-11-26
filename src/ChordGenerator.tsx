import type { Component } from 'solid-js';

import * as util from './core/util';
import * as helper from './solid_helper';

import {Cadence} from './core/tune';
import * as core from './core/chord';
import type {Parameters} from './core/chord';


namespace RootGenerator {
  type Parameters = core.RootGenerator.Parameters;
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

}

namespace ToneGenerator {
  type Parameters = core.ToneGenerator.Parameters;
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

}

namespace ToneModifier {
  type Parameters = core.ToneModifier.Parameters;
  export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
    get(): Parameters {
      return {}
    }
    set(params: Parameters): Parameters { return params; }
    ui: Component = () => { return <></>; }
  }
}


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
