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


class ScaleUiAdapter extends helper.UiAdapter<Scale> {
  private key = new helper.InputBoundNumber(69);
  private tones: ()=>number[];
  private setTones: (v:number[])=>number[];
  constructor() {
    super();
    [this.tones, this.setTones] = createSignal([0,2,4,5,7,9,11]);
  }
  get(): Scale { return new Scale(this.key.get(), this.tones()); }
  set(v:Scale): Scale {
    this.key.set(v.root);
    this.setTones(v.tones);
    return this.get();
  }
  ui: Component = () => {
    return <>
      <label>key
        <helper.ClassUI instance={this.key} />
      </label>
      <label>tones
        [{this.tones().join(",")}]
        <input type="button" value="Major" onClick={() => this.set(Scale.major(this.key.get()))} />
        <input type="button" value="minor" onClick={() => this.set(Scale.minor(this.key.get()))} />
      </label>
    </>;
  }
}

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
  private scale = new ScaleUiAdapter();
  private time_measure = [new helper.InputBoundNumber(4), new helper.InputBoundNumber(4)];
  private max_beat_division_depth = new helper.InputBoundNumber(3);
  private resolution = new helper.Enum<Resolution>(Resolution, Resolution.Deceptive);
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
    const scale = this.scale.get();
    return {
      scale: {key: scale.root, tones: scale.tones},
      time_measure: [this.time_measure[0].get(), this.time_measure[1].get()],
      max_beat_division_depth: this.max_beat_division_depth.get(),
      resolution: this.resolution.get(),
      cadence: this.cadence.get(),
      chord: this.chord.get(),
      note: this.note.get(),
    };
  }
  set(params: TuneGeneratorParameters): TuneGeneratorParameters {
    this.scale.set( new Scale(params.scale.key, params.scale.tones) );
    this.time_measure.forEach((t,i) => t.set(params.time_measure[i]));
    this.max_beat_division_depth.set(params.max_beat_division_depth);
    this.resolution.set(params.resolution);
    this.cadence.set(params.cadence);
    this.chord.set(params.chord);
    this.note.set(params.note);
    return params;
  }
  ui: Component = () => {
    let anchor = document.createElement("a");
    anchor.href = URL.createObjectURL(new Blob([]));
    anchor.download = "TuneGenParams.json";
    anchor.style.display = "none";
    return <>
      <details>
        <summary>Settings</summary>
        <input type="button" value="save as file" onClick={() => {
          URL.revokeObjectURL(anchor.href);
          anchor.href = URL.createObjectURL(new Blob([JSON.stringify(this.get())]));
          anchor.click();
        }} />
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
        <details>
          <summary>scale</summary>
          <helper.ClassUI instance={this.scale} />
        </details>
        <label>resolution
          <helper.ClassUI instance={this.resolution} />
        </label>
        <label>time measure
          <helper.ClassUI instance={this.time_measure[0]} />
          <helper.ClassUI instance={this.time_measure[1]} />
        </label>
        <label>max beat division depth
          <helper.ClassUI instance={this.max_beat_division_depth} />
        </label>
        <ClassUI instance={this.cadence} />
        <ClassUI instance={this.chord} />
        <ClassUI instance={this.note} />
      </details>
    </>
  };
}

export class TuneGenerator {
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
    tune.cadence = (new CadenceGenerator()).generate(tune, params.cadence);
    tune.chord = (new ChordGenerator()).generate(tune, params.chord);
    tune.notes = (new NoteGenerator()).generate(tune, params.note);
    return this.set(tune);
  }
}
