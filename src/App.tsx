import { Component, createSignal } from 'solid-js';
import './App.scss';

import {ClassUI} from './solid_helper';

import * as DefaultTuneGenParams from '../resources/TuneGenParams_default.json';
import * as DefaultVariantGenParams from '../resources/VariantGenParams_default.json';

import {Tune} from './tune';
import {TuneGenerator, TuneGeneratorParametersUiAdapter} from './TuneGenerator';
import {VariantGenerator, VariantGeneratorParametersUiAdapter} from './VariantGenerator';
import {PianoRoll} from './PianoRoll';
import {TunePlayer} from './TunePlayer';

const App: Component = () => {
  const [counter, setCounter] = createSignal(0);
  setInterval(setCounter, 1000, (c: number) => c + 1);

  const baseTuneSignal = createSignal(new Tune());
  const [baseTune, _setBaseTune] = baseTuneSignal;
  const generator = new TuneGenerator(baseTuneSignal);
  const tuneParamsUi = new TuneGeneratorParametersUiAdapter();
  const tuneParams = TuneGeneratorParametersUiAdapter.fromJSON( DefaultTuneGenParams );
  tuneParamsUi.set( tuneParams );

  const variantTuneSignal = createSignal(new Tune());
  const [variantTune, _setVariantTune] = variantTuneSignal;
  const variantParamsUi = new VariantGeneratorParametersUiAdapter();
  const variantParams = VariantGeneratorParametersUiAdapter.getParametersFromJSON( DefaultVariantGenParams );
  variantParamsUi.set( variantParams );
  const variantGenerator = new VariantGenerator(variantTuneSignal);

  const generate = () => {
    generator.generate( tuneParamsUi.get() );
    variantGenerator.generate(variantParamsUi.get(), tuneParamsUi.get());
  };
  const tune = () => baseTune().merge( variantTune() );

  generate();

  return (
    <>
      <div>
        <h1 class="header">{counter()}</h1>
      </div>
      <ClassUI instance={tuneParamsUi} />
      <ClassUI instance={variantParamsUi} />
      <button onClick={() => generate()}>generate</button>
      <PianoRoll tune={tune()} note_bottom={57} note_top={81} />
      <TunePlayer tune={tune()} />
    </>
  );
  //      <ClassUI instance={generator} />
};

export default App;
