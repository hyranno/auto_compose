import type { Component } from 'solid-js';

import * as helper from './solid_helper';

import type {Parameters} from './core/note';
import * as PitchWeight from './PitchWeightCalculator';
import * as RhythmGenerator from './RhytmGenerator';


export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
  private rhythm = new RhythmGenerator.ParametersUiAdapter();
  private pitchWeight = new PitchWeight.ParametersUiAdapter();
  private seed = new helper.RandomSeed(4649,459);
  get(): Parameters {
    return {
      rhythm: this.rhythm.get(),
      pitchWeight: this.pitchWeight.get(),
      seed: this.seed.get()
    };
  }
  set(params: Parameters): Parameters {
    this.rhythm.set(params.rhythm);
    this.pitchWeight.set(params.pitchWeight);
    this.seed.set(params.seed);
    return params;
  }
  ui: Component = () => {
    return <details><summary>NoteGenerator</summary>
      <helper.ClassUI instance={this.rhythm} />
      <helper.ClassUI instance={this.pitchWeight} />
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
    </details>;
  };
}
