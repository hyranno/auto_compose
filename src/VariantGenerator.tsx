import Ajv from "ajv"
import { Component } from 'solid-js';

import schema from '../schemas/VariantGeneratorParameters.json';

import {Resolution} from './core/tune';
import type {Layer, Parameters} from './core/variant';
import * as util from './core/util';
import * as helper from './solid_helper';


export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
  private seed = new helper.RandomSeed(4649,459);
  private resolution: Resolution = Resolution.Perfect;
  private timespan: util.WeightedItem<[number,number]>[];
  private layer: util.WeightedItem<Layer>[];
  get(): Parameters {
    return {
      seed: this.seed.get(),
      resolution: this.resolution,
      timespan: this.timespan,
      layer: this.layer,
    };
  }
  set(v: Parameters): Parameters {
    this.seed.set(v.seed);
    this.resolution = v.resolution;
    this.timespan = v.timespan;
    this.layer = v.layer;
    return v;
  }
  ui: Component = ()=>{
    return <details><summary>Settings(Variant)</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
    </details>;
  }
  static getParametersFromJSON(data: any): Parameters {
    //const data = JSON.parse(jsonstr);
    const ajv = new Ajv();
    const validate = ajv.compile<Parameters>(schema);
    if (!validate(data)) {
      alert("Parameters JSON is not valid");
      console.log(validate.errors);
    }
    return data;
  }
}
