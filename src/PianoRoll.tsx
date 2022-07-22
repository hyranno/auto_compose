import type { Component } from 'solid-js';
import * as tunes from './tune';
import {rangeIterator} from './util';

import './PianoRoll.scss';


export const PianoRoll: Component<{tune: tunes.Tune, note_bottom: number, note_top: number,}> = (props) => {
  const tune = () => props.tune;

  let time_length = props.tune.time_measure[0] * props.tune.time_measure[1];
  let timetick = 1 / (1 << props.tune.max_beat_division_depth);
  let times = [...rangeIterator(0, time_length, timetick)];
  let notes = [...rangeIterator(props.note_bottom, props.note_top, 1)].reverse();

  const Cadence: Component<{t:number}> = (props) => {
    return <td>
      {tune().cadence.get(props.t).value.charAt(0)}
    </td>;
  }
  const CadenceCol: Component = () => {
    return <tr>
      <td></td>
      {times.map((t) => <Cadence t={t} />)}
    </tr>;
  }

  const Notenum: Component<{pitch: number}> = (props) => {
    return <td>{props.pitch}</td>;
  }
  const Note: Component<{pitch: number, t: number}> = (props) => {
    return <td classList={{
      "chord": tune().chord.get(props.t).value.includes(props.pitch),
      "chordroot": tune().chord.get(props.t).value.isRoot(props.pitch),
      "note": tune().notes.get(props.t).value.pitch == props.pitch,
    }}></td>;
  }
  const NoteRow: Component<{pitch: number}> = (props) => {
    return <>
      <tr classList={{
        "key": props.pitch==tune().scale.root,
        "scale": tune().scale.includes(props.pitch),
        "scaleroot": tune().scale.isRoot(props.pitch),
      }}>
        <Notenum pitch={props.pitch} />
        {times.map((t) => <Note pitch={props.pitch} t={t} />)}
      </tr>
    </>;
  }

  return <table class="pianoroll">
    <CadenceCol />
    {notes.map((n) => <NoteRow pitch={n} />)}
  </table>;
}
