import type { Component } from 'solid-js';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Scale, Chord, Note} from './tune';
import type {notenum} from './tune';


export type Parameters = {
  absPitchFactor: util.SmoothstepParameters;
  relPitchFactor: util.SmoothstepParameters;
  factorInScale: number;
  factorInChord: number;
  rhythmExponentFactor: number;
};
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

  function getAbsolutePitchFactor(centor: notenum, candidate: notenum, params: Parameters): number {
    let distance = Math.abs(candidate - centor);
    return 1 - util.smoothstep(params.absPitchFactor.edge0, params.absPitchFactor.edge1, distance);
  }
  function getRelativePitchFactor(prev: notenum, candidate: notenum, params: Parameters): number {
    let distance = Math.abs(candidate - prev);
    return 1 - util.smoothstep(params.relPitchFactor.edge0, params.relPitchFactor.edge1, distance);
  }
  function getFactorInScale(scale: Scale, candidate: notenum, params: Parameters): number {
    return scale.includes(candidate)? params.factorInScale : 1;
  }
  function getFactorInChord(chord: Chord, candidate: notenum, params: Parameters): number {
    return chord.includes(candidate)? params.factorInChord : 1;
  }
  function getRhythmExponent(duration: number, params: Parameters): number {
    return duration * params.rhythmExponentFactor;
  }
export function  calcTimeHomoWeight(tune: Tune, prev: notenum, candidate: notenum, params: Parameters): number {
    let absPitchFactor = getAbsolutePitchFactor(tune.scale.root, candidate, params);
    let relPitchFactor = getRelativePitchFactor(prev, candidate, params);
    let factorInScale = getFactorInScale(tune.scale, candidate, params);
    return absPitchFactor * relPitchFactor * factorInScale;
  }
export function  calcWithTimeHomoWeight(tune: Tune, timehomoWeight: number, candidate: Note, t: number, params: Parameters): number {
    let factorInChord = getFactorInChord(tune.chord.get(t).value, candidate.pitch, params);
    let rhythmExponent = getRhythmExponent(candidate.duration, params);
    return Math.pow(
      timehomoWeight * factorInChord,
      1 + rhythmExponent
    );
  }
export function  calc(tune: Tune, prev: notenum, candidate: Note, t: number, params: Parameters): number {
    let timehomoWeight = calcTimeHomoWeight(tune, prev, candidate.pitch, params);
    return calcWithTimeHomoWeight(tune, timehomoWeight, candidate, t, params);
    /* return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale * factorInChord,
      params.regularity + rhythmExponent
    ); */
  }
