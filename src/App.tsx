import { Component, createSignal } from 'solid-js';
import './App.scss';

import {ClassUI} from './solid_helper';

import {Tune} from './tune';
import {TuneGenerator} from './TuneGenerator';
import {PianoRoll} from './PianoRoll';
import {TunePlayer} from './TunePlayer';

const App: Component = () => {
  const [counter, setCounter] = createSignal(0);
  setInterval(setCounter, 1000, (c: number) => c + 1);

  const tuneSignal = createSignal(new Tune());
  const [tune, _setTune] = tuneSignal;
  const generator = new TuneGenerator(tuneSignal);
  generator.generate();

  return (
    <>
      <div>
        <h1 class="header">{counter()}</h1>
      </div>
      <ClassUI instance={generator} />
      <PianoRoll tune={tune()} note_bottom={57} note_top={81} />
      <TunePlayer tune={tune()} />
    </>
  );
};

export default App;
