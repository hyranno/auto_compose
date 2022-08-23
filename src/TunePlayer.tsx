import type { Component } from 'solid-js';
import * as Tone from "tone";

import * as helper from './solid_helper';
import * as tunes from './tune';

//import './TunePlayer.scss';


function playChord(tune: tunes.Tune, timerate: number) {
  let now = Tone.now();
  let synth = new Tone.PolySynth().toDestination();
  let duration = tune.chord.list()[1].t;
  let chords_hertz = tune.chord.list().map(e => {return {
    t: e.t,
    value: e.value.getNotenums().map(p => tunes.notenumToHerzs(p-12))
  }});
  chords_hertz.forEach(e => {
    e.value.forEach(f => {synth.triggerAttackRelease(f, duration*timerate, now + e.t*timerate);});
  });
}

function playNotes(tune: tunes.Tune, timerate: number) {
  let now = Tone.now();
  let synth = new Tone.PolySynth().toDestination();
  let notes_hertz = tune.notes.list().filter(e => e.value.isNoteOn).map(e => {return {
    t: e.t,
    freq: tunes.notenumToHerzs( e.value.pitch ),
    duration: e.value.duration,
  }});
  notes_hertz.forEach(e => {
    synth.triggerAttackRelease(e.freq, e.duration*timerate, now + e.t*timerate);
  });
}

function playAll(tune: tunes.Tune, timerate: number) {
  playNotes(tune, timerate);
  playChord(tune, timerate);
}

export const TunePlayer: Component<{tune: tunes.Tune}> = (props) => {
  const bpm = new helper.InputBoundNumber(100);
  const timerate = ()=> 60/bpm.get();
  return <>
    BPM<helper.ClassUI instance={bpm} />
    <button onClick={() => playChord(props.tune, timerate())}>play chord</button>
    <button onClick={() => playNotes(props.tune, timerate())}>play notes</button>
    <button onClick={() => playAll(props.tune, timerate())}>play notes & chord</button>
  </>;
}
