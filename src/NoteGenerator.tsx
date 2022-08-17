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
class RhythmGeneratorParametersUiAdapter extends helper.UiAdapter<RhythmGeneratorParameters> {
  private seed = new helper.RandomSeed(4649,459);
  private pBranch = new helper.Smoothstep(1/2, 3);
  private pNoteOn = new helper.Smoothstep(0, 1.2);
  get(): RhythmGeneratorParameters {
    return {
      seed: this.seed.get(),
      pBranch: this.pBranch.get(),
      pNoteOn: this.pNoteOn.get()
    };
  }
  set(params: RhythmGeneratorParameters): RhythmGeneratorParameters {
    this.seed.set(params.seed);
    this.pBranch.set(params.pBranch);
    this.pNoteOn.set(params.pNoteOn);
    return params;
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
}
class RhythmGenerator {
  generateNode(rand: Random, depthcount: number, dest: Rhythm[], candidate: Rhythm, params: RhythmGeneratorParameters) {
    let pLeaf = 1 - util.smoothstep(params.pBranch.edge0, params.pBranch.edge1, candidate.duration);
    let isLeaf = (rand.random() < pLeaf) || (depthcount <= 0);
    if (isLeaf) {
      let pNoteOn = util.smoothstep(params.pNoteOn.edge0, params.pNoteOn.edge1, candidate.duration);
      candidate.isNoteOn = rand.random() < pNoteOn;
      dest.push(candidate);
    } else {
      let c0 = {isNoteOn: true, t: candidate.t, duration: candidate.duration/2};
      let c1 = {isNoteOn: true, t: c0.t + c0.duration, duration: c0.duration};
      this.generateNode(rand, depthcount-1, dest, c0, params);
      this.generateNode(rand, depthcount-1, dest, c1, params);
    }
  }
  generate(tune: Tune, params: RhythmGeneratorParameters): Rhythm[] {
    let rand = new Random(params.seed.state, params.seed.sequence);
    let res = [];
    [...util.rangeIterator(0,tune.time_measure[1],1)].forEach( i =>
      this.generateNode(rand, 3, res,
        {isNoteOn:true, t:i*tune.time_measure[0], duration: tune.time_measure[0]},
        params
      )
    );
    return res;
  }
}

export type NoteGeneratorParameters = {
  rhythm: RhythmGeneratorParameters;
  pitchWeight: PitchWeightParameters;
  seed: util.RandomSeed;
};
export class NoteGeneratorParametersUiAdapter extends helper.UiAdapter<NoteGeneratorParameters> {
  private rhythm = new RhythmGeneratorParametersUiAdapter();
  private pitchWeight = new PitchWeightParametersUiAdapter();
  private seed = new helper.RandomSeed(4649,459);
  get(): NoteGeneratorParameters {
    return {
      rhythm: this.rhythm.get(),
      pitchWeight: this.pitchWeight.get(),
      seed: this.seed.get()
    };
  }
  set(params: NoteGeneratorParameters): NoteGeneratorParameters {
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
export class NoteGenerator {
  private rhythmGen = new RhythmGenerator();
  private pitchWeightCalc = new PitchWeightCalculator();
  generateNote(tune: Tune, rand: Random, prev: Note, rhythm: Rhythm, params: NoteGeneratorParameters): Note{
    let candidates: Note[] = [...util.rangeIterator(prev.pitch-12, prev.pitch+12, 1)].map(p => {return {
      pitch: p,
      isNoteOn: rhythm.isNoteOn,
      duration: rhythm.duration
    }});
    let rand_notes = new util.WeightedRandom(
      rand,
      candidates.map((c) => {return {weight: this.pitchWeightCalc.calc(tune, prev, c, rhythm.t, params.pitchWeight), value: c}})
    );
    return rand_notes.get();
  }
  generate(tune: Tune, params: NoteGeneratorParameters): util.Timeline<Note> {
    const rhythms = this.rhythmGen.generate(tune, params.rhythm);
    let rand = new Random(params.seed.state, params.seed.sequence);
    let prev = {pitch: tune.scale.root, isNoteOn: false, duration: 0}; //dummy
    const events = rhythms.map(r => {
      const note = this.generateNote(tune, rand, prev, r, params);
      if (note.isNoteOn) {
        prev = note;
      }
      return {t:r.t, value:note};
    });
    return util.Timeline.fromItems(events);
  }
}


export type PitchWeightParameters = {
  absPitchFactor: util.SmoothstepParameters;
  relPitchFactor: util.SmoothstepParameters;
  factorInScale: number;
  factorInChord: number;
  rhythmExponentFactor: number;
  regularity: number;
};
export class PitchWeightParametersUiAdapter {
  private absPitchFactorEdges = new helper.Smoothstep(4,12);
  private relPitchFactorEdges = new helper.Smoothstep(6,12);
  private factorInScale = new helper.InputBoundNumber(8);
  private factorInChord = new helper.InputBoundNumber(4);
  private rhythmExponentFactor = new helper.InputBoundNumber(1/2);
  private regularity = new helper.InputBoundNumber(1);
  get(): PitchWeightParameters {
    return {
      absPitchFactor: this.absPitchFactorEdges.get(),
      relPitchFactor: this.relPitchFactorEdges.get(),
      factorInScale: this.factorInScale.get(),
      factorInChord: this.factorInChord.get(),
      rhythmExponentFactor: this.rhythmExponentFactor.get(),
      regularity: this.regularity.get()
    };
  }
  set(params: PitchWeightParameters) {
    this.absPitchFactorEdges.set(params.absPitchFactor);
    this.relPitchFactorEdges.set(params.relPitchFactor);
    this.factorInScale.set(params.factorInScale);
    this.factorInChord.set(params.factorInChord);
    this.rhythmExponentFactor.set(params.rhythmExponentFactor);
    this.regularity.set(params.regularity);
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
      <label>regularity
        <helper.ClassUI instance={this.regularity} />
      </label>
    </details>;
  };
}
export class PitchWeightCalculator {
  private getAbsolutePitchFactor(centor: notenum, candidate: notenum, params: PitchWeightParameters): number {
    let distance = Math.abs(candidate - centor);
    return 1 - util.smoothstep(params.absPitchFactor.edge0, params.absPitchFactor.edge1, distance);
  }
  private getRelativePitchFactor(prev: notenum, candidate: notenum, params: PitchWeightParameters): number {
    let distance = Math.abs(candidate - prev);
    return 1 - util.smoothstep(params.relPitchFactor.edge0, params.relPitchFactor.edge1, distance);
  }
  private getFactorInScale(scale: Scale, candidate: notenum, params: PitchWeightParameters): number {
    return scale.includes(candidate)? params.factorInScale : 1;
  }
  private getFactorInChord(chord: Chord, candidate: notenum, params: PitchWeightParameters): number {
    return chord.includes(candidate)? params.factorInChord : 1;
  }
  private getRhythmExponent(duration: number, params: PitchWeightParameters): number {
    return duration * params.rhythmExponentFactor;
  }
  calcTimeHomoWeight(tune: Tune, prev: notenum, candidate: notenum, params: PitchWeightParameters): number {
    let absPitchFactor = this.getAbsolutePitchFactor(tune.scale.root, candidate, params);
    let relPitchFactor = this.getRelativePitchFactor(prev, candidate, params);
    let factorInScale = this.getFactorInScale(tune.scale, candidate, params);
    return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale,
      params.regularity
    );
  }
  calcWithTimeHomoWeight(tune: Tune, timehomoWeight: number, candidate: Note, t: number, params: PitchWeightParameters): number {
    let factorInChord = this.getFactorInChord(tune.chord.get(t).value, candidate.pitch, params);
    let rhythmExponent = this.getRhythmExponent(candidate.duration, params);
    return Math.pow(
      Math.pow(timehomoWeight, 1/params.regularity) * factorInChord,
      params.regularity + rhythmExponent
    );
  }
  calc(tune: Tune, prev: Note, candidate: Note, t: number, params: PitchWeightParameters): number {
    let timehomoWeight = this.calcTimeHomoWeight(tune, prev.pitch, candidate.pitch, params);
    return this.calcWithTimeHomoWeight(tune, timehomoWeight, candidate, t, params);
    /* return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale * factorInChord,
      params.regularity + rhythmExponent
    ); */
  }
}


export class PitchWeightCalculator_bak {
  private absPitchFactorEdges = new helper.Smoothstep(4,12);
  private relPitchFactorEdges = new helper.Smoothstep(6,12);
  private factorInScale = new helper.InputBoundNumber(8);
  private factorInChord = new helper.InputBoundNumber(4);
  private rhythmExponentFactor = new helper.InputBoundNumber(1/2);
  private regularity = new helper.InputBoundNumber(1);
  private getAbsolutePitchFactor(centor: notenum, candidate: notenum): number {
    let distance = Math.abs(candidate - centor);
    return 1 - this.absPitchFactorEdges.calc(distance);
  }
  private getRelativePitchFactor(prev: notenum, candidate: notenum): number {
    let distance = Math.abs(candidate - prev);
    return 1 - this.relPitchFactorEdges.calc(distance);
  }
  private getFactorInScale(scale: Scale, candidate: notenum): number {
    return scale.includes(candidate)? this.factorInScale.get() : 1;
  }
  private getFactorInChord(chord: Chord, candidate: notenum): number {
    return chord.includes(candidate)? this.factorInChord.get() : 1;
  }
  private getRhythmExponent(duration: number): number {
    return duration * this.rhythmExponentFactor.get();
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
      <label>regularity
        <helper.ClassUI instance={this.regularity} />
      </label>
    </details>;
  };
  getParameters(): PitchWeightParameters {
    return {
      absPitchFactor: this.absPitchFactorEdges.get(),
      relPitchFactor: this.relPitchFactorEdges.get(),
      factorInScale: this.factorInScale.get(),
      factorInChord: this.factorInChord.get(),
      rhythmExponentFactor: this.rhythmExponentFactor.get(),
      regularity: this.regularity.get()
    };
  }
  setParameters(params: PitchWeightParameters) {
    this.absPitchFactorEdges.set(params.absPitchFactor);
    this.relPitchFactorEdges.set(params.relPitchFactor);
    this.factorInScale.set(params.factorInScale);
    this.factorInChord.set(params.factorInChord);
    this.rhythmExponentFactor.set(params.rhythmExponentFactor);
    this.regularity.set(params.regularity);
  }

  calcTimeHomoWeight(tune: Tune, prev: notenum, candidate: notenum): number {
    let absPitchFactor = this.getAbsolutePitchFactor(tune.scale.root, candidate);
    let relPitchFactor = this.getRelativePitchFactor(prev, candidate);
    let factorInScale = this.getFactorInScale(tune.scale, candidate);
    return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale,
      this.regularity.get()
    );
  }
  calcWeightFromTimeHomoWeight(tune: Tune, timehomoWeight: number, candidate: Note, t: number): number {
    let factorInChord = this.getFactorInChord(tune.chord.get(t).value, candidate.pitch);
    let rhythmExponent = this.getRhythmExponent(candidate.duration);
    return Math.pow(
      Math.pow(timehomoWeight, 1/this.regularity.get()) * factorInChord,
      this.regularity.get() + rhythmExponent
    );
  }
  calcWeight(tune: Tune, prev: Note, candidate: Note, t: number): number {
    let timehomoWeight = this.calcTimeHomoWeight(tune, prev.pitch, candidate.pitch);
    return this.calcWeightFromTimeHomoWeight(tune, timehomoWeight, candidate, t);
    /* return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale * factorInChord,
      this.regularity.get() + rhythmExponent
    ); */
  }
}
