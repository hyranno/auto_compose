import type { TuneSnapshot } from 'auto-composer-wasm';

/** The frequency of a MIDI note number on 12 equal temperament (A4 = 69 = 440 Hz). */
export function notenumToHertz(notenum: number): number {
  return 440 * Math.pow(2, (notenum - 69) / 12);
}

/** The pitch class (0 to 11) of a MIDI note number. */
export function pitchClass(notenum: number): number {
  return ((notenum % 12) + 12) % 12;
}

/** Whether `notenum` is in the same pitch class as one of `notenums`. */
export function includesPitchClass(notenums: number[], notenum: number): boolean {
  const pc = pitchClass(notenum);
  return notenums.some((n) => pitchClass(n) === pc);
}

/** The notenums of the scale of the tune, from its key. */
export function scaleNotenums(tune: TuneSnapshot): number[] {
  return tune.scale.tones.map((tone) => tune.scale.key + tone);
}

/**
 * The item in effect at `t`: the last one that starts at or before `t`,
 * or `undefined` if `t` is before the first one. `items` must be sorted by `t`.
 */
export function itemAt<T extends { t: number }>(items: T[], t: number): T | undefined {
  let lo = 0;
  let hi = items.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (items[mid].t <= t) lo = mid + 1;
    else hi = mid;
  }
  return lo > 0 ? items[lo - 1] : undefined;
}

/** A sound of a track: `time` and `duration` are in beats. */
export type TrackNote = { time: number; notenum: number; duration: number };

/** The melody: the notes that sound (not rests). */
export function melodyTrack(tune: TuneSnapshot): TrackNote[] {
  return tune.notes
    .filter((note) => note.isNoteOn)
    .map((note) => ({ time: note.t, notenum: note.pitch, duration: note.duration }));
}

/**
 * The chords, an octave below their notenums as in the original page.
 * Each chord lasts until the next one, and the last one until the end of the tune.
 */
export function chordTrack(tune: TuneSnapshot): TrackNote[] {
  return tune.chord.flatMap((chord, i) => {
    const duration = (tune.chord[i + 1]?.t ?? tune.length) - chord.t;
    return chord.notenums.map((notenum) => ({ time: chord.t, notenum: notenum - 12, duration }));
  });
}
