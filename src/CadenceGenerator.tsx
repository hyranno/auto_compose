import type { Component } from 'solid-js';

import * as util from './core/util';
import * as helper from './solid_helper';

import {Cadence} from './core/tune';
import type {Parameters} from './core/cadence';


export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
  private seed = new helper.RandomSeed(4649,459);
  private probabilities: [Cadence, util.WeightedItem<Cadence>[]][] = ([
    [Cadence.T, [{value: Cadence.T, weight: 0.1}, {value: Cadence.D, weight: 0.3}, {value: Cadence.S, weight: 0.6}] ],
    [Cadence.D, [{value: Cadence.T, weight: 0.9}, {value: Cadence.D, weight: 0.1}, {value: Cadence.S, weight: 0.0}] ],
    [Cadence.S, [{value: Cadence.T, weight: 0.4}, {value: Cadence.D, weight: 0.5}, {value: Cadence.S, weight: 0.1}] ],
  ]);
  private duration = new helper.InputBoundNumber(2);
  get(): Parameters {
    return {
      seed: this.seed.get(),
      probabilities: this.probabilities,
      duration: this.duration.get()
    };
  }
  set(params: Parameters): Parameters {
    this.duration.set(params.duration);
    this.seed.set(params.seed);
    this.probabilities = params.probabilities;
    return params;
  }
  ui: Component = () => {
    return <details>
      <summary>Cadence Generator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
      <label>duration
        <helper.ClassUI instance={this.duration} />
      </label>
    </details>;
  };
}
