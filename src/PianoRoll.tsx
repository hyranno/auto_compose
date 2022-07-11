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

function NoteHeight(height: number) {
  return <td>{height}</td>;
}
function Note(tune: tunes.Tune, t: number) {
  return <td class="chord note"></td>;
}
function NoteRow(tune: tunes.Tune, height: number, times: number[]) {
  return <>
    <tr classList={{
      "key": height==tune.key,
      "scale": tune.scale.includes(height),
      "scaleroot": tune.scale.isRoot(height),
    }}>
      {NoteHeight(height)}
      {times.map((t)=>Note(tune, t))}
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
