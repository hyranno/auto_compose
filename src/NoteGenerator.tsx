import type { Component } from 'solid-js';
import {Random} from 'reliable-random';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Scale, Chord, Note} from './tune';
import type {notenum} from './tune';


type Rhythm = {
  isNoteOn: boolean,
  t: number,
  duration: number,
}
type RhythmGeneratorParameters = {
  seed: util.RandomSeed;
  pBranch: util.SmoothstepParameters;
  pNoteOn: util.SmoothstepParameters;
};
class RhythmGenerator {
  private seed = new helper.RandomSeed(4649,459);
  private pBranch = new helper.Smoothstep(1/2, 3);
  private pNoteOn = new helper.Smoothstep(0, 1.2);
  generateNode(rand: Random, depthcount: number, dest: Rhythm[], candidate: Rhythm) {
    let pLeaf = 1 - this.pBranch.calc(candidate.duration);
    let isLeaf = (rand.random() < pLeaf) || (depthcount <= 0);
    if (isLeaf) {
      let pNoteOn = this.pNoteOn.calc(candidate.duration);
      let isNoteOn = rand.random() < pNoteOn;
      candidate.isNoteOn = isNoteOn;
      dest.push(candidate);
    } else {
      let c0 = {isNoteOn: true, t: candidate.t, duration: candidate.duration/2};
      let c1 = {isNoteOn: true, t: c0.t + c0.duration, duration: c0.duration};
      this.generateNode(rand, depthcount-1, dest, c0);
      this.generateNode(rand, depthcount-1, dest, c1);
    }
  }
  generate(tune: Tune): Rhythm[] {
    let rand = this.seed.createRandom();
    let res = [];
    [...util.rangeIterator(0,tune.time_measure[1],1)].forEach( i =>
      this.generateNode(rand, 3, res,
        {isNoteOn:true, t:i*tune.time_measure[0], duration: tune.time_measure[0]}
      )
    );
    return res;
  }
  ui: Component = () => {
    return <details><summary>RhythmGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
      <label>pBranchEdges
        <helper.ClassUI instance={this.pBranch} />
      </label>
      <label>pNoteOnEdges
        <helper.ClassUI instance={this.pNoteOn} />
      </label>
    </details>;
  };
  setParameters(params: RhythmGeneratorParameters) {
    this.seed.set(params.seed);
    this.pBranch.set(params.pBranch);
    this.pNoteOn.set(params.pNoteOn);
  }
}

export type NoteGeneratorParameters = {
  rhythm: RhythmGeneratorParameters;
  seed: util.RandomSeed;
  absPitchFactor: util.SmoothstepParameters;
  relPitchFactor: util.SmoothstepParameters;
  factorInScale: number;
  factorInChord: number;
  rhythmExponentFactor: number;
  regularity: number;
};
export class NoteGenerator {
  private rhythmGen = new RhythmGenerator();
  private seed = new helper.RandomSeed(4649,459);
  private absPitchFactorEdges = new helper.Smoothstep(4,12);
  private relPitchFactorEdges = new helper.Smoothstep(6,12);
  private factorInScale = new helper.InputBoundNumber(8);
  private factorInChord = new helper.InputBoundNumber(4);
  private rhythmExponentFactor = new helper.InputBoundNumber(1/2);
  private regularity = new helper.InputBoundNumber(1);
  calcAbsolutePitchFactor(centor: notenum, candidate: Note): number {
    let distance = Math.abs(candidate.pitch - centor);
    return 1 - this.absPitchFactorEdges.calc(distance);
  }
  calcRelativePitchFactor(prev: Note, candidate: Note): number {
    let distance = Math.abs(candidate.pitch - prev.pitch);
    return 1 - this.relPitchFactorEdges.calc(distance);
  }
  calcFactorInScale(scale: Scale, candidate: Note): number {
    return scale.includes(candidate.pitch)? this.factorInScale.get() : 1;
  }
  calcFactorInChord(chord: Chord, candidate: Note): number {
    return chord.includes(candidate.pitch)? this.factorInChord.get() : 1;
  }
  calcRhythmExponent(duration: number): number {
    return duration * this.rhythmExponentFactor.get();
  }
  calcWeight(tune: Tune, t: number, prev: Note, candidate: Note): number {
    let absPitchFactor = this.calcAbsolutePitchFactor(tune.scale.root, candidate);
    let relPitchFactor = this.calcRelativePitchFactor(prev, candidate);
    let factorInScale = this.calcFactorInScale(tune.scale, candidate);
    let factorInChord = this.calcFactorInChord(tune.chord.get(t).value, candidate);
    let rhythmExponent = this.calcRhythmExponent(candidate.duration);
    return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale * factorInChord,
      rhythmExponent + this.regularity.get()
    );
  }
  generateNote(rand: Random, tune: Tune, prev: Note, rhythm: Rhythm): Note{
    let candidates: Note[] = [...util.rangeIterator(prev.pitch-12, prev.pitch+12, 1)].map(p => {return {
      pitch: p,
      duration: rhythm.duration
    }});
    let rand_notes = new util.WeightedRandom(
      rand,
      candidates.map((c) => {return {weight: this.calcWeight(tune, rhythm.t, prev, c), value: c}})
    );
    return rand_notes.get();
  }
  generate(tune: Tune): util.Timeline<Note> {
    let rhythms = this.rhythmGen.generate(tune);
    let rand = this.seed.createRandom();
    let prev = {pitch: tune.scale.root, duration: 0}; //dummy
    let events = rhythms.map(r => {
      prev = this.generateNote(rand, tune, prev, r);
      return {t:r.t, value:prev};
    });
    return util.Timeline.fromItems(events);
  }
  ui: Component = () => {
    return <details><summary>NoteGenerator</summary>
      <helper.ClassUI instance={this.rhythmGen} />
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
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
      <label>regularity
        <helper.ClassUI instance={this.regularity} />
      </label>
    </details>;
  };
  setParameters(params: NoteGeneratorParameters) {
    this.rhythmGen.setParameters(params.rhythm);
    this.seed.set(params.seed);
    this.absPitchFactorEdges.set(params.absPitchFactor);
    this.relPitchFactorEdges.set(params.relPitchFactor);
    this.factorInScale.set(params.factorInScale);
    this.factorInChord.set(params.factorInChord);
    this.rhythmExponentFactor.set(params.rhythmExponentFactor);
    this.regularity.set(params.regularity);
  }
}




type PitchState = {
  index: number,
  pitch: notenum
}
export class PitchChain {
  private tune: Tune;
  private rhythms: Rhythm[];
  private random: Random;
  private state: PitchState;
  private seed = new helper.RandomSeed(4649,459);
  private absPitchFactorEdges = new helper.Smoothstep(4,12);
  private relPitchFactorEdges = new helper.Smoothstep(6,12);
  private factorInScale = new helper.InputBoundNumber(8);
  private factorInChord = new helper.InputBoundNumber(4);
  private rhythmExponentFactor = new helper.InputBoundNumber(1/2);
  private regularity = new helper.InputBoundNumber(1);
  private calcAbsolutePitchFactor(centor: notenum, candidate: Note): number {
    let distance = Math.abs(candidate.pitch - centor);
    return 1 - this.absPitchFactorEdges.calc(distance);
  }
  private calcRelativePitchFactor(prev: Note, candidate: Note): number {
    let distance = Math.abs(candidate.pitch - prev.pitch);
    return 1 - this.relPitchFactorEdges.calc(distance);
  }
  private calcFactorInScale(scale: Scale, candidate: Note): number {
    return scale.includes(candidate.pitch)? this.factorInScale.get() : 1;
  }
  private calcFactorInChord(chord: Chord, candidate: Note): number {
    return chord.includes(candidate.pitch)? this.factorInChord.get() : 1;
  }
  private calcRhythmExponent(duration: number): number {
    return duration * this.rhythmExponentFactor.get();
  }
  private calcWeight(tune: Tune, index: number, prev: Note, candidate: Note): number {
    let absPitchFactor = this.calcAbsolutePitchFactor(tune.scale.root, candidate);
    let relPitchFactor = this.calcRelativePitchFactor(prev, candidate);
    let factorInScale = this.calcFactorInScale(tune.scale, candidate);
    let factorInChord = this.calcFactorInChord(tune.chord.get(this.rhythms[index].t).value, candidate);
    let rhythmExponent = this.calcRhythmExponent(candidate.duration);
    return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale * factorInChord,
      rhythmExponent + this.regularity.get()
    );
  }
  calc_distribution_sub(state: PitchState, isBackward: boolean = false): util.WeightedItem<PitchState>[] {
    let index = state.index + (isBackward? -1: 1);
    let prev: Note = {pitch: state.pitch, duration: this.rhythms[state.index].duration};
    let candidates: Note[] = [...util.rangeIterator(state.pitch-12, state.pitch+12, 1)].map(p => {return {
      pitch: p,
      duration: this.rhythms[index].duration
    }});
    return candidates.map((c) => {return {
      weight: this.calcWeight(this.tune, index, prev, c),
      value: {index: index, pitch: c.pitch}
    }});
  }
}
