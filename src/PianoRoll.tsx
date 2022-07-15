import * as tunes from './tune';
import {rangeIterator} from './util';

import './PianoRoll.scss';




function Cadence(tune: tunes.Tune, t: number) {
  return <td>
    {tune.cadence.get(t).value.charAt(0)}
  </td>;
}
function CadenceCol(tune: tunes.Tune, times: number[]) {
  return <tr>
    <td></td>
    {times.map((t)=>Cadence(tune, t))}
  </tr>;
}

function Notenum(pitch: number) {
  return <td>{pitch}</td>;
}
function Note(tune: tunes.Tune, pitch: number, t: number) {
  return <td classList={{
    "chord": tune.chord.get(t).value.includes(pitch),
    "chordroot": tune.chord.get(t).value.isRoot(pitch),
    "note": tune.notes.get(t).value.pitch == pitch,
  }}></td>;
}
function NoteRow(tune: tunes.Tune, pitch: number, times: number[]) {
  return <>
    <tr classList={{
      "key": pitch==tune.scale.root,
      "scale": tune.scale.includes(pitch),
      "scaleroot": tune.scale.isRoot(pitch),
    }}>
      {Notenum(pitch)}
      {times.map((t)=>Note(tune, pitch, t))}
    </tr>
  </>;
}

export function PianoRoll(
    tune: tunes.Tune,
    note_bottom: number, note_top: number,
  ) {
  let time_length = tune.time_measure[0] * tune.time_measure[1];
  let timetick = 1 / (1 << tune.max_beat_division_depth);
  let times = [...rangeIterator(0, time_length, timetick)];
  let notes = [...rangeIterator(note_bottom, note_top, 1)].reverse();
  return <table class="pianoroll">
    {CadenceCol(tune, times)}
    {notes.map((n)=>NoteRow(tune, n,times))}
  </table>;
}
