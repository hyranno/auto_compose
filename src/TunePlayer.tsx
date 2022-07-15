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

export function TunePlayer(
    tune: tunes.Tune,
  ) {
  return <>
    <button onClick={() => playChord(tune)}>play chord</button>
    <button onClick={() => playNotes(tune)}>play notes</button>
    <button onClick={() => {playNotes(tune); playChord(tune);}}>play notes & chord</button>
  </>;
}
