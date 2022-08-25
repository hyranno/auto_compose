import type { Component } from 'solid-js';
import {Random} from 'reliable-random';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Note, NoteTreeNode} from './tune';
import type {notenum} from './tune';
import * as PitchWeight from './PitchWeightCalculator';
import * as RhythmGenerator from './RhytmGenerator';


export type Parameters = {
  rhythm: RhythmGenerator.Parameters;
  pitchWeight: PitchWeight.Parameters;
  seed: util.RandomSeed;
};
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

export function generatePitch(tune: Tune, rand: Random, prev: notenum, note: NoteTreeNode, params: PitchWeight.Parameters){
  const pitches = [...util.rangeIterator(prev-12, prev+12)];
  let rand_pitch: util.WeightedRandom<notenum> = new util.WeightedRandom(
    rand,
    pitches.map(pitch => {
      const candidate = {pitch: pitch, isNoteOn: note.isNoteOn, duration: note.duration};
      return {
      value: pitch,
      weight: PitchWeight.calc(tune, prev, candidate, note.t, params)}
    })
  );
  note.pitch = rand_pitch.get();
}
export function generateTree(tune: Tune, params: Parameters): NoteTreeNode {
  const notes = RhythmGenerator.generate(tune, params.rhythm);
  let rand = new Random(params.seed.state, params.seed.sequence);
  let prev = tune.scale.root; //dummy
  notes.leaves().forEach(note => {
    generatePitch(tune, rand, prev, note, params.pitchWeight);
    const pitch = note.pitch;   util.assertIsDefined(pitch);
    prev = pitch;
  });
  return notes;
}
export function generate(tune: Tune, params: Parameters): util.Timeline<Note> {
  const noteTree = generateTree(tune, params);
  const events: util.TimelineItem<Note>[] = noteTree.leaves().map(note => {
    const pitch = note.pitch;   util.assertIsDefined(pitch);
    return { t: note.t, value: {duration: note.duration, pitch: pitch, isNoteOn: note.isNoteOn} };
  });
  return util.Timeline.fromItems(events);
}
