import * as util from './util';

import {Tune, Scale, Chord, Note} from './tune';
import type {notenum} from './tune';


export type Parameters = {
  absPitchFactor: util.SmoothstepParameters;
  relPitchFactor: util.SmoothstepParameters;
  factorInScale: number;
  factorInChord: number;
  rhythmExponentFactor: number;
};

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
