import type { Component } from 'solid-js';
import * as Tone from "tone";

import * as tunes from './tune';

//import './TunePlayer.scss';


function playChord(tune: tunes.Tune) {
  let now = Tone.now();
  let synth = new Tone.PolySynth().toDestination();
  let chords_hertz = tune.chord.list().map(e => {return {
    t: e.t,
    value: e.value.getNotenums().map(p => tunes.notenumToHerzs(p-12))
  }});
  chords_hertz.forEach(e => {
    e.value.forEach(f => {synth.triggerAttackRelease(f, 2, now + e.t);});
  });
}

function playNotes(tune: tunes.Tune) {
  let now = Tone.now();
  let synth = new Tone.PolySynth().toDestination();
  let notes_hertz = tune.notes.list().map(e => {return {
    t: e.t,
    freq: tunes.notenumToHerzs( e.value.pitch ),
    duration: e.value.duration,
  }});
  notes_hertz.forEach(e => {
    synth.triggerAttackRelease(e.freq, e.duration, now + e.t);
  });
}

export const TunePlayer: Component<{tune: tunes.Tune}> = (props) => {
  return <>
    <button onClick={() => playChord(props.tune)}>play chord</button>
    <button onClick={() => playNotes(props.tune)}>play notes</button>
    <button onClick={() => {playNotes(props.tune); playChord(props.tune);}}>play notes & chord</button>
  </>;
}
