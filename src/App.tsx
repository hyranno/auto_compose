import { Component, createSignal } from 'solid-js';
import './App.scss';

import {ClassUI} from './solid_helper';

import * as DefaultTuneGenParams from '../resources/TuneGenParams_default.json';
import * as DefaultVariantGenParams from '../resources/VariantGenParams_default.json';

import {Tune} from './core/tune';
import * as TuneGenerator from './TuneGenerator';
import * as VariantGenerator from './VariantGenerator';
import * as TuneGeneratorCore from './core/tune_generator';
import * as VariantGeneratorCore from './core/variant';
import {PianoRoll} from './PianoRoll';
import {TunePlayer} from './TunePlayer';
import {TuneMidiExporter} from './TuneMidiExporter';

const App: Component = () => {
  const tuneParamsUi = new TuneGenerator.ParametersUiAdapter();
  const variantParamsUi = new VariantGenerator.ParametersUiAdapter();

  const [baseTune, setBaseTune] = createSignal(new Tune());
  const [variantTune, setVariantTune] = createSignal(new Tune());
  const generate = () => {
    setBaseTune( TuneGeneratorCore.generate( tuneParamsUi.get() ) );
    setVariantTune( VariantGeneratorCore.generate(variantParamsUi.get(), tuneParamsUi.get()) );
  };
  const tune = () => baseTune().merge( variantTune() );

  tuneParamsUi.set( TuneGenerator.ParametersUiAdapter.fromJSON( DefaultTuneGenParams ) );
  variantParamsUi.set( VariantGenerator.ParametersUiAdapter.getParametersFromJSON( DefaultVariantGenParams ) );
  generate();

  return (
    <>
      <ClassUI instance={tuneParamsUi} />
      <ClassUI instance={variantParamsUi} />
      <button onClick={() => generate()}>generate</button>
      <PianoRoll tune={tune()} note_bottom={tune().scale.root-14} note_top={tune().scale.root+14} />
      <TunePlayer tune={tune()} />
      <TuneMidiExporter tune={tune()} />
    </>
  );
};

export default App;
