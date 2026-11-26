import type { Component } from 'solid-js';

import * as helper from './solid_helper';

import type {Parameters} from './core/rhythm';


export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
  private seed = new helper.RandomSeed(4649,459);
  private max_beat_division_depth = new helper.InputBoundNumber(3);
  private pBranch = new helper.Smoothstep(1/2, 3);
  private pNoteOn = new helper.Smoothstep(0, 1.2);
  get(): Parameters {
    return {
      seed: this.seed.get(),
      max_beat_division_depth: this.max_beat_division_depth.get(),
      pBranch: this.pBranch.get(),
      pNoteOn: this.pNoteOn.get()
    };
  }
  set(params: Parameters): Parameters {
    this.seed.set(params.seed);
    this.max_beat_division_depth.set(params.max_beat_division_depth);
    this.pBranch.set(params.pBranch);
    this.pNoteOn.set(params.pNoteOn);
    return params;
  }
  ui: Component = () => {
    return <details><summary>RhythmGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
      <label>max beat division depth
        <helper.ClassUI instance={this.max_beat_division_depth} />
      </label>
      <label>pBranchEdges
        <helper.ClassUI instance={this.pBranch} />
      </label>
      <label>pNoteOnEdges
        <helper.ClassUI instance={this.pNoteOn} />
      </label>
    </details>;
  };
}
