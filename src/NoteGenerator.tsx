import type { Component } from 'solid-js';
import {Random} from 'reliable-random';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Scale, Chord, Note} from './tune';
import type {notenum} from './tune';


export type Rhythm = {
  isNoteOn: boolean,
  t: number,
  duration: number,
}
export type RhythmGeneratorParameters = {
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

export class NoteTreeNode extends util.TreeNode {
  rhythm: Rhythm;
  pitch: notenum | undefined;
  get t(): number {return this.rhythm.t;}
  get duration(): number {return this.rhythm.duration};
  get isNoteOn(): boolean {return this.rhythm.isNoteOn};
  constructor(rhythm: Rhythm) {
    super();
    this.rhythm = rhythm;
  }
  getRhythms(): Rhythm[] {
    if (this.isLeaf()) {
      return [this.rhythm];
    }
    return this.children.map(n => n.getRhythms()).flat();
  }
  toTimelineItem(): util.TimelineItem<Note> {
    const pitch = this.pitch;
    util.assertIsDefined(pitch);
    return {t: this.t, value: {pitch: pitch, duration: this.duration, isNoteOn: this.isNoteOn}};
  }
}

export class RhythmGenerator {
  generateSubTree(
    node: NoteTreeNode, rand: Random, depthcount: number, params: RhythmGeneratorParameters
  ) {
    node.children = [];
    let pBranch = util.smoothstep(params.pBranch.edge0, params.pBranch.edge1, node.duration);
    let isBranch =  (0 < depthcount) && (rand.random() < pBranch);
    if (isBranch) {
      const numChildren = 2;
      const subDuration = node.duration / numChildren;
      node.children.push(
        ...[...util.rangeIterator(0,numChildren)].map(i => new NoteTreeNode(
          {t: node.t+i*subDuration, duration: subDuration, isNoteOn: true}
        ))
      );
      node.children.forEach(n => this.generateSubTree(n, rand, depthcount-1, params));
    } else {
      const pNoteOn = util.smoothstep(params.pNoteOn.edge0, params.pNoteOn.edge1, node.duration);
      node.rhythm.isNoteOn = rand.random() < pNoteOn;
    }
  }
  mergeTopBranches(root: NoteTreeNode, rand: Random, params: RhythmGeneratorParameters) {
    for (let i=0; i < root.children.length-1; i++) {
      const duration = root.children[i].duration + root.children[i+1].duration;
      const pMerge = 1 - util.smoothstep(params.pBranch.edge0, params.pBranch.edge1, duration);
      if (rand.random() < pMerge) {
        root.children[i].rhythm.duration = duration;
        root.children[i].children = [];
        root.children[i+1].rhythm.duration = 0;
        root.children[i+1].children = [];
        i++;
      }
    }
    root.children = root.children.filter(n => n.duration > 0);
  }
  generate(tune: Tune, params: RhythmGeneratorParameters): NoteTreeNode {
    const numBeats = tune.time_measure[0];
    const numBars = tune.time_measure[1];
    let rand = new Random(params.seed.state, params.seed.sequence);
    let phrase = new NoteTreeNode({t:0, duration: numBeats*numBars, isNoteOn: true});
    let bars = [...util.rangeIterator(0, numBars)].map(i => new NoteTreeNode({
      t: i*numBeats, duration: numBeats, isNoteOn: true,
    }));
    bars.forEach(bar => {
      let beats = [...util.rangeIterator(0, numBeats)].map(i => new NoteTreeNode({
        t: bar.t+i*1, duration: 1, isNoteOn: true,
      }));
      beats.forEach(beat => this.generateSubTree(beat, rand, tune.max_beat_division_depth, params));
      bar.push(...beats);
      this.mergeTopBranches(bar, rand, params);
    });
    phrase.push(...bars);
    return phrase;
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
  generatePitch(tune: Tune, rand: Random, prev: notenum, note: NoteTreeNode, params: PitchWeightParameters){
    const pitches = [...util.rangeIterator(prev-12, prev+12)];
    let rand_pitch: util.WeightedRandom<notenum> = new util.WeightedRandom(
      rand,
      pitches.map(pitch => {
        const candidate = {pitch: pitch, isNoteOn: note.isNoteOn, duration: note.duration};
        return {
        value: pitch,
        weight: this.pitchWeightCalc.calc(tune, prev, candidate, note.t, params)}
      })
    );
    note.pitch = rand_pitch.get();
  }
  generateTree(tune: Tune, params: NoteGeneratorParameters): NoteTreeNode {
    const notes = this.rhythmGen.generate(tune, params.rhythm);
    let rand = new Random(params.seed.state, params.seed.sequence);
    let prev = tune.scale.root; //dummy
    notes.leaves().forEach(note => {
      this.generatePitch(tune, rand, prev, note, params.pitchWeight);
      const pitch = note.pitch;   util.assertIsDefined(pitch);
      prev = pitch;
    });
    return notes;
  }
  generate(tune: Tune, params: NoteGeneratorParameters): util.Timeline<Note> {
    const noteTree = this.generateTree(tune, params);
    const events: util.TimelineItem<Note>[] = noteTree.leaves().map(note => {
      const pitch = note.pitch;   util.assertIsDefined(pitch);
      return { t: note.t, value: {duration: note.duration, pitch: pitch, isNoteOn: note.isNoteOn} };
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
  private absPitchFactorEdges = new helper.Smoothstep(6,12);
  private relPitchFactorEdges = new helper.Smoothstep(4,8);
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
  calc(tune: Tune, prev: notenum, candidate: Note, t: number, params: PitchWeightParameters): number {
    let timehomoWeight = this.calcTimeHomoWeight(tune, prev, candidate.pitch, params);
    return this.calcWithTimeHomoWeight(tune, timehomoWeight, candidate, t, params);
    /* return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale * factorInChord,
      params.regularity + rhythmExponent
    ); */
  }
}
