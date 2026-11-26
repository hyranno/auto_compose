import { Midi } from '@tonejs/midi';
import * as Tone from 'tone';
import type { TuneSnapshot } from 'auto-composer-wasm';

import { download } from './file';
import { chordTrack, melodyTrack, notenumToHertz, type TrackNote } from './tune';

async function play(tracks: TrackNote[][], secondsPerBeat: number) {
  // Browsers keep the audio context suspended until a user gesture resumes it.
  await Tone.start();
  const now = Tone.now();
  for (const track of tracks) {
    const synth = new Tone.PolySynth().toDestination();
    for (const note of track) {
      synth.triggerAttackRelease(
        notenumToHertz(note.notenum),
        note.duration * secondsPerBeat,
        now + note.time * secondsPerBeat,
      );
    }
  }
}

function toMidi(tracks: TrackNote[][], secondsPerBeat: number): Uint8Array {
  const midi = new Midi();
  for (const track of tracks) {
    const midiTrack = midi.addTrack();
    for (const note of track) {
      midiTrack.addNote({
        midi: note.notenum,
        time: note.time * secondsPerBeat,
        duration: note.duration * secondsPerBeat,
      });
    }
  }
  return midi.toArray();
}

/** Plays the tune and exports it as MIDI, at `bpm` beats per minute. */
export function TuneOutput(props: { tune: TuneSnapshot; bpm: number }) {
  const secondsPerBeat = () => 60 / props.bpm;
  const melody = () => melodyTrack(props.tune);
  const chords = () => chordTrack(props.tune);
  return (
    <div>
      <button type="button" onClick={() => play([chords()], secondsPerBeat())}>
        play chord
      </button>
      <button type="button" onClick={() => play([melody()], secondsPerBeat())}>
        play melody
      </button>
      <button type="button" onClick={() => play([melody(), chords()], secondsPerBeat())}>
        play chord &amp; melody
      </button>
      <button
        type="button"
        onClick={() => download(toMidi([melody(), chords()], secondsPerBeat()) as BlobPart, 'auto.midi', 'audio/midi')}
      >
        download MIDI
      </button>
    </div>
  );
}
