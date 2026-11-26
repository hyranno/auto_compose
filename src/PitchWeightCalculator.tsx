import type { Component } from 'solid-js';

import * as helper from './solid_helper';

import type {Parameters} from './core/pitch_weight';


export class ParametersUiAdapter {
  private absPitchFactorEdges = new helper.Smoothstep(6,12);
  private relPitchFactorEdges = new helper.Smoothstep(4,8);
  private factorInScale = new helper.InputBoundNumber(8);
  private factorInChord = new helper.InputBoundNumber(4);
  private rhythmExponentFactor = new helper.InputBoundNumber(1/2);
  get(): Parameters {
    return {
      absPitchFactor: this.absPitchFactorEdges.get(),
      relPitchFactor: this.relPitchFactorEdges.get(),
      factorInScale: this.factorInScale.get(),
      factorInChord: this.factorInChord.get(),
      rhythmExponentFactor: this.rhythmExponentFactor.get(),
    };
  }
  set(params: Parameters) {
    this.absPitchFactorEdges.set(params.absPitchFactor);
    this.relPitchFactorEdges.set(params.relPitchFactor);
    this.factorInScale.set(params.factorInScale);
    this.factorInChord.set(params.factorInChord);
    this.rhythmExponentFactor.set(params.rhythmExponentFactor);
  }
  ui: Component = () => {
    return <details><summary>PitchWeight</summary>
      <label>absPitchFactorEdges
        <helper.ClassUI instance={this.absPitchFactorEdges} />
      </label>
      <label>relPitchFactorEdges
        <helper.ClassUI instance={this.relPitchFactorEdges} />
      </label>
      <label>factorInScale
        <helper.ClassUI instance={this.factorInScale} />
      </label>
      <label>factorInChord
        <helper.ClassUI instance={this.factorInChord} />
      </label>
      <label>rhythmExponentFactor
        <helper.ClassUI instance={this.rhythmExponentFactor} />
      </label>
    </details>;
  };
}
