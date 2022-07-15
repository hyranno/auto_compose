import { Component, createSignal } from 'solid-js';
import './App.scss';

import * as tunes from './tune';
import {PianoRoll} from './PianoRoll';
import {TunePlayer} from './TunePlayer';

const App: Component = () => {
  const [counter, setCounter] = createSignal(0);
  setInterval(setCounter, 1000, (c: number) => c + 1);

  let tune = new tunes.Tune();
  tune.scale = tunes.Scale.major(69);
  tune.time_measure = [4,4];
  tune.max_beat_division_depth = 2;

  let cadenceGen = new tunes.CadenceGenerator();
  tune.cadence = cadenceGen.generate(tune);

  let chordGen = new tunes.ChordGenerator();
  tune.chord = chordGen.generate(tune);

  let noteGen = new tunes.NoteGenerator();
  tune.notes = noteGen.generate(tune);

  return (
    <>
      <div>
        <h1 class="header">{counter()}</h1>
      </div>
      {PianoRoll(tune, 57,81)}
      {TunePlayer(tune)}
    </>
  );
};

export default App;
