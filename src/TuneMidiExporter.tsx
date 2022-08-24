import type { Component } from 'solid-js';
import { Midi } from '@tonejs/midi'

import * as helper from './solid_helper';
import * as tunes from './tune';


function addChordTrack(midi: Midi, tune: tunes.Tune, timerate: number) {
  const duration = tune.chord.list()[1].t * timerate;
  const track = midi.addTrack();
  let midiNotes = tune.chord.list().map(e => {
    return e.value.getNotenums().map(pitch => {return {
      time: e.t * timerate,
      midi: pitch - 12,
      duration: duration,
    };});
  }).flat();
  midiNotes.forEach(e => {
    track.addNote(e);
  });
}

function addNoteTrack(midi: Midi, tune: tunes.Tune, timerate: number) {
  const track = midi.addTrack();
  let midiNotes = tune.notes.list().filter(e => e.value.isNoteOn).map(e => {return {
    time: e.t * timerate,
    midi: e.value.pitch,
    duration: e.value.duration * timerate,
  };});
  midiNotes.forEach(e => {
    track.addNote(e);
  });
}

export const TuneMidiExporter: Component<{tune: tunes.Tune}> = (props) => {
  const bpm = new helper.InputBoundNumber(100);
  const timerate = ()=> 60/bpm.get();
  let anchor = document.createElement("a");
  anchor.href = URL.createObjectURL(new Blob([]));
  anchor.download = "auto.midi";
  anchor.style.display = "none";
  return <div>
    BPM<helper.ClassUI instance={bpm} />
    {anchor}
    <input type="button" value="download Midi" onClick={()=>{
      URL.revokeObjectURL(anchor.href);
      let midi = new Midi();
      addNoteTrack(midi, props.tune, timerate());
      addChordTrack(midi, props.tune, timerate());
      anchor.href = URL.createObjectURL(new Blob([midi.toArray()]));
      anchor.click();
    }} />
  </div>;
}
