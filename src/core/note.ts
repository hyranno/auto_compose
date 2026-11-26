import {defaultContext, GenerateContext} from './random';
import type {Rng} from './random';

import * as util from './util';

import {Tune, Note, NoteTreeNode} from './tune';
import type {notenum} from './tune';
import * as PitchWeight from './pitch_weight';
import * as RhythmGenerator from './rhythm';


export type Parameters = {
  rhythm: RhythmGenerator.Parameters;
  pitchWeight: PitchWeight.Parameters;
  seed: util.RandomSeed;
};

export function generatePitch(tune: Tune, rand: Rng, prev: notenum, note: NoteTreeNode, params: PitchWeight.Parameters){
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
export function generateTree(tune: Tune, params: Parameters, ctx: GenerateContext = defaultContext): NoteTreeNode {
  const notes = RhythmGenerator.generate(tune, params.rhythm, ctx);
  let rand = ctx.rngFactory(params.seed, 'note.pitch');
  let prev = tune.scale.root; //dummy
  notes.leaves().forEach(note => {
    generatePitch(tune, rand, prev, note, params.pitchWeight);
    const pitch = note.pitch;   util.assertIsDefined(pitch);
    prev = pitch;
  });
  return notes;
}
export function generate(tune: Tune, params: Parameters, ctx: GenerateContext = defaultContext): util.Timeline<Note> {
  const noteTree = generateTree(tune, params, ctx);
  const events: util.TimelineItem<Note>[] = noteTree.leaves().map(note => {
    const pitch = note.pitch;   util.assertIsDefined(pitch);
    return { t: note.t, value: {duration: note.duration, pitch: pitch, isNoteOn: note.isNoteOn} };
  });
  return util.Timeline.fromItems(events);
}
