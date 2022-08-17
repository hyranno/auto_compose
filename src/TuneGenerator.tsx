import Ajv from "ajv"
import type { Component } from 'solid-js';
import {createSignal} from 'solid-js';
import './TuneGenerator.scss';

import {ClassUI} from './solid_helper';
import * as util from './util';
import * as helper from './solid_helper';

import schema from '../schemas/TuneGeneratorParameters.json';
import {CadenceGenerator, CadenceGeneratorParameters, CadenceGeneratorParametersUiAdapter} from './CadenceGenerator';
import {ChordGenerator, ChordGeneratorParameters, ChordGeneratorParametersUiAdapter} from './ChordGenerator';
import {NoteGenerator, NoteGeneratorParameters, NoteGeneratorParametersUiAdapter} from './NoteGenerator';

import {Tune, Scale, Resolution} from './tune';


export type TuneGeneratorParameters = {
  scale: {key: number, tones: number[]};
  time_measure: [number, number];
  max_beat_division_depth: number;
  resolution: Resolution;
  cadence: CadenceGeneratorParameters;
  chord: ChordGeneratorParameters;
  note: NoteGeneratorParameters;
};
export class TuneGeneratorParametersUiAdapter extends helper.UiAdapter<TuneGeneratorParameters> {
  private scale = Scale.major(69);
  private time_measure: [number, number] = [4,4];
  private max_beat_division_depth = 2;
  private resolution: Resolution = Resolution.Perfect;
  private cadence = new CadenceGeneratorParametersUiAdapter();
  private chord = new ChordGeneratorParametersUiAdapter();
  private note = new NoteGeneratorParametersUiAdapter();
  static fromJSON(data: any): TuneGeneratorParameters {
    //const data = JSON.parse(jsonstr);
    const ajv = new Ajv();
    const validate = ajv.compile<TuneGeneratorParameters>(schema);
    if (!validate(data)) {
      alert("Parameters JSON is not valid");
      console.log(validate.errors);
    }
    return data;
  }
  get(): TuneGeneratorParameters {
    return {
      scale: {key: this.scale.root, tones: this.scale.tones},
      time_measure: this.time_measure,
      max_beat_division_depth: this.max_beat_division_depth,
      resolution: this.resolution,
      cadence: this.cadence.get(),
      chord: this.chord.get(),
      note: this.note.get(),
    };
  }
  set(params: TuneGeneratorParameters): TuneGeneratorParameters {
    this.scale = new Scale(params.scale.key, params.scale.tones);
    this.time_measure = params.time_measure;
    this.max_beat_division_depth = params.max_beat_division_depth;
    this.resolution = params.resolution;
    this.cadence.set(params.cadence);
    this.chord.set(params.chord);
    this.note.set(params.note);
    return params;
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
            fileElem[0].text().then( (str) => {
              this.set( TuneGeneratorParametersUiAdapter.fromJSON( JSON.parse(str)) )
            } );
          }} />
        </label>
        <ClassUI instance={this.cadence} />
        <ClassUI instance={this.chord} />
        <ClassUI instance={this.note} />
      </details>
    </>
  };
}

export class TuneGenerator {
  private cadenceGen = new CadenceGenerator();
  private chordGen = new ChordGenerator();
  private noteGen = new NoteGenerator();
  get: ()=>Tune;
  private set: (v: Tune)=>Tune;
  constructor(signal: [()=>Tune, (v:Tune)=>Tune]) {
    [this.get, this.set] = signal;
  }
  generate(params: TuneGeneratorParameters): Tune {
    util.assertIsDefined(params);
    let tune = new Tune();
    tune.length = params.time_measure[0] * params.time_measure[1];
    tune.scale = new Scale(params.scale.key, params.scale.tones);
    tune.time_measure = params.time_measure;
    tune.max_beat_division_depth = params.max_beat_division_depth;
    tune.resolution = util.Timeline.fromItems([{t:0, value:params.resolution}]);
    tune.cadence = this.cadenceGen.generate(tune, params.cadence);
    tune.chord = this.chordGen.generate(tune, params.chord);
    tune.notes = this.noteGen.generate(tune, params.note);
    return this.set(tune);
  }
}
