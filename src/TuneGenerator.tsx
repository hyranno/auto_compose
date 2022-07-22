import Ajv, {JSONSchemaType} from "ajv"
import type { Component } from 'solid-js';
import {createSignal} from 'solid-js';
import './TuneGenerator.scss';

import {ClassUI} from './solid_helper';
import * as util from './util';

import schema from '../schemas/TuneGeneratorParameters.json';
import {CadenceGenerator, CadenceGeneratorParameters} from './CadenceGenerator';
import {ChordGenerator, ChordGeneratorParameters} from './ChordGenerator';
import {NoteGenerator, NoteGeneratorParameters} from './NoteGenerator';

import {Tune, Scale} from './tune';


export type TuneGeneratorParameters = {
  scale: {key: number, tones: number[]};
  time_measure: [number, number];
  max_beat_division_depth: number;
  cadence: CadenceGeneratorParameters;
  chord: ChordGeneratorParameters;
  note: NoteGeneratorParameters;
};
export class TuneGenerator {
  private scale = Scale.major(69);
  private time_measure: [number, number] = [4,4];
  private max_beat_division_depth = 2;
  private cadenceGen = new CadenceGenerator();
  private chordGen = new ChordGenerator();
  private noteGen = new NoteGenerator();
  get: ()=>Tune;
  private set: (v: Tune)=>Tune;
  constructor(signal: [()=>Tune, (v:Tune)=>Tune]) {
    [this.get, this.set] = signal;
  }
  ui: Component = () => {
    return <>
      <details>
        <summary>Settings</summary>
        <label>
          load from file
          <input type="file" onInput={(e) => {
            let fileElem = (e.target as HTMLInputElement).files;
            util.assertIsDefined(fileElem);
            fileElem[0].text().then( (str) => this.setParametersFromJSON(str) );
          }} />
        </label>
        <ClassUI instance={this.cadenceGen} />
        <ClassUI instance={this.chordGen} />
        <ClassUI instance={this.noteGen} />
      </details>
      <button onClick={() => this.generate()}>generate</button>
    </>
  };
  generate(): Tune {
    let tune = new Tune();
    tune.scale = this.scale;
    tune.time_measure = this.time_measure;
    tune.max_beat_division_depth = this.max_beat_division_depth;
    tune.cadence = this.cadenceGen.generate(tune);
    tune.chord = this.chordGen.generate(tune);
    tune.notes = this.noteGen.generate(tune);
    return this.set(tune);
  }
  setParameters(params: TuneGeneratorParameters) {
    this.scale = new Scale(params.scale.key, params.scale.tones);
    this.time_measure = params.time_measure;
    this.max_beat_division_depth = params.max_beat_division_depth;
    this.cadenceGen.setParameters(params.cadence);
    this.chordGen.setParameters(params.chord);
    this.noteGen.setParameters(params.note);
  }
  setParametersFromJSON(jsonstr: string) {
    const ajv = new Ajv();
    const data = JSON.parse(jsonstr);
    const validate = ajv.compile<TuneGeneratorParameters>(schema);
    if (validate(data)) {
      this.setParameters(data);
    } else {
      alert("Parameters JSON is not valid");
      console.log(validate.errors);
    };
  }
}
