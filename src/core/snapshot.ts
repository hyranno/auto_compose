// Plain-JSON form of a Tune, used by the golden fixtures shared with the Rust port.
import {Tune} from './tune';

export type TuneSnapshot = {
  length: number;
  time_measure: [number, number];
  max_beat_division_depth: number;
  scale: {key: number, tones: number[]};
  resolution: {t: number, value: string}[];
  cadence: {t: number, value: string}[];
  // notenums is redundant with root/tones; it is kept to cross-check Chord.getNotenums.
  chord: {t: number, root: number, tones: number[], notenums: number[]}[];
  notes: {t: number, pitch: number, duration: number, isNoteOn: boolean}[];
};

export function snapshot(tune: Tune): TuneSnapshot {
  return {
    length: tune.length,
    time_measure: [tune.time_measure[0], tune.time_measure[1]],
    max_beat_division_depth: tune.max_beat_division_depth,
    scale: {key: tune.scale.root, tones: tune.scale.tones.slice()},
    resolution: tune.resolution.list().map(e => ({t: e.t, value: e.value})),
    cadence: tune.cadence.list().map(e => ({t: e.t, value: e.value})),
    chord: tune.chord.list().map(e => ({
      t: e.t, root: e.value.root, tones: e.value.tones.slice(), notenums: e.value.getNotenums(),
    })),
    notes: tune.notes.list().map(e => ({
      t: e.t, pitch: e.value.pitch, duration: e.value.duration, isNoteOn: e.value.isNoteOn,
    })),
  };
}
