import { createMemo, For } from 'solid-js';
import type { TuneSnapshot } from 'auto-composer-wasm';

import { includesPitchClass, itemAt, pitchClass, scaleNotenums } from './tune';
import './PianoRoll.css';

/** What is in effect at one column (one tick) of the roll. */
type Column = {
  t: number;
  cadence?: string;
  chordNotenums: number[];
  chordRoot?: number;
  note?: { pitch: number; isNoteOn: boolean };
  beat: boolean;
  bar: boolean;
  phrase: boolean;
};

function columns(tune: TuneSnapshot): Column[] {
  const tick = 1 / (1 << tune.max_beat_division_depth);
  const bar = tune.time_measure[0];
  const phrase = bar * tune.time_measure[1];
  const result: Column[] = [];
  // Counting ticks, rather than adding up `tick`, keeps `t` exact.
  for (let i = 0; i * tick < tune.length; i++) {
    const t = i * tick;
    const chord = itemAt(tune.chord, t);
    result.push({
      t,
      cadence: itemAt(tune.cadence, t)?.value,
      chordNotenums: chord?.notenums ?? [],
      chordRoot: chord?.notenums[0],
      note: itemAt(tune.notes, t),
      beat: t % 1 === 0,
      bar: t % bar === 0,
      phrase: t % phrase === 0,
    });
  }
  return result;
}

/** The tune on a grid of pitches (rows, from `noteTop` down to `noteBottom`) and ticks (columns). */
export function PianoRoll(props: { tune: TuneSnapshot; noteBottom: number; noteTop: number }) {
  const cols = createMemo(() => columns(props.tune));
  const pitches = createMemo(() => {
    const result: number[] = [];
    for (let p = props.noteTop; p >= props.noteBottom; p--) result.push(p);
    return result;
  });
  const scale = createMemo(() => scaleNotenums(props.tune));

  return (
    <table class="pianoroll">
      <tbody>
        <tr>
          <td />
          <For each={cols()}>{(col) => <td class={col.cadence} />}</For>
        </tr>
        <For each={pitches()}>
          {(pitch) => (
            <tr
              classList={{
                key: pitch === props.tune.scale.key,
                scale: includesPitchClass(scale(), pitch),
                scaleroot: pitchClass(pitch) === pitchClass(props.tune.scale.key),
              }}
            >
              <td>{pitch}</td>
              <For each={cols()}>
                {(col) => (
                  <td
                    classList={{
                      chord: includesPitchClass(col.chordNotenums, pitch),
                      chordroot: col.chordRoot !== undefined && pitchClass(col.chordRoot) === pitchClass(pitch),
                      note: col.note?.isNoteOn === true && col.note.pitch === pitch,
                      beat: col.beat,
                      bar: col.bar,
                      phrase: col.phrase,
                    }}
                  />
                )}
              </For>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  );
}
