import Ajv from "ajv"
import { Component, createSignal } from 'solid-js';
import {Random} from 'reliable-random';

import schema from '../schemas/VariantGeneratorParameters.json';

import {Tune, Scale, Cadence, Chord, Note} from './tune';
import {TuneGenerator, TuneGeneratorParameters} from './TuneGenerator';
import {CadenceGenerator, CadenceGeneratorParameters} from './CadenceGenerator';
import {ChordGenerator, ChordGeneratorParameters} from './ChordGenerator';
import {NoteGenerator, NoteGeneratorParameters} from './NoteGenerator';
import {PitchWeightCalculator_bak, PitchWeightCalculator, PitchWeightParameters} from './NoteGenerator';
import * as util from './util';
import * as helper from './solid_helper';
import * as markov from './markov';


export const VariantGeneratorLayer = {
  "Cadence": 0,
  "Chord": 1,
  "Pitch": 2,
  "Rhythm": 3,
} as const;
export type VariantGeneratorLayer = (typeof VariantGeneratorLayer)[keyof (typeof VariantGeneratorLayer)];


export type VariantGeneratorParameters = {
  seed: util.RandomSeed;
  timespan: util.WeightedItem<[number,number]>[];
  layer: util.WeightedItem<VariantGeneratorLayer>[];
};
export class VariantGeneratorParametersUiAdapter extends helper.UiAdapter<VariantGeneratorParameters> {
  private seed = new helper.RandomSeed(4649,459);
  private timespan: util.WeightedItem<[number,number]>[];
  private layer: util.WeightedItem<VariantGeneratorLayer>[];
  get(): VariantGeneratorParameters {
    return {
      seed: this.seed.get(),
      timespan: this.timespan,
      layer: this.layer,
    };
  }
  set(v: VariantGeneratorParameters): VariantGeneratorParameters {
    this.seed.set(v.seed);
    this.timespan = v.timespan;
    this.layer = v.layer;
    return v;
  }
  ui: Component = ()=>{
    return <details><summary>VariantGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
    </details>;
  }
  static getParametersFromJSON(data: any): VariantGeneratorParameters {
    //const data = JSON.parse(jsonstr);
    const ajv = new Ajv();
    const validate = ajv.compile<VariantGeneratorParameters>(schema);
    if (!validate(data)) {
      alert("Parameters JSON is not valid");
      console.log(validate.errors);
    }
    return data;
  }
}

export class VariantGenerator {
  get: ()=>Tune;
  private set: (v: Tune)=>Tune;
  constructor(signals: [()=>Tune, (v:Tune)=>Tune]) {
    this.get = signals[0];
    this.set = signals[1];
  }
  generate(params: VariantGeneratorParameters, baseParams: TuneGeneratorParameters): Tune {
    let rand = new Random(params.seed.state, params.seed.sequence);
    let timespan = (new util.WeightedRandom(rand, params.timespan)).get();
    let layer = (new util.WeightedRandom(rand, params.layer)).get();
    let tuneGen = new TuneGenerator(createSignal(new Tune()));
    let tune = tuneGen.generate( baseParams );
    if (layer <= VariantGeneratorLayer.Cadence) {
      let cadenceMod = new CadenceVariantGenerator();
      tune.cadence = cadenceMod.generate(tune, timespan, rand, baseParams.cadence);
    }
    if (layer <= VariantGeneratorLayer.Chord) {
      let chordMod = new ChordVariantGenerator();
      tune.chord = chordMod.generate(tune, timespan, rand, baseParams.chord);
    }
    if (layer <= VariantGeneratorLayer.Pitch) {
      let pitchMod = new PitchVariantGenerator();
      tune.notes = pitchMod.generate(tune, timespan, rand, baseParams.note.pitchWeight);
    }
    if (layer == VariantGeneratorLayer.Rhythm) {
      //modify rhythm
      //interpolate pitch
    }
    return this.set(tune);
  }
}


class CadenceVariantGenerator {
  generate(tune: Tune, timespan: [number, number], rand: Random, params: CadenceGeneratorParameters): util.Timeline<Cadence> {
    let [startIndex, endIndex] = timespan.map((t) => tune.cadence.getIndex(t))
    let li = tune.cadence.list();
    let cadenceMarkov = new markov.MarkovChain_TimeHomo_FiniteState(
      rand, li[startIndex].value,
      new Map<Cadence, util.WeightedItem<Cadence>[]>(params.probabilities)
    );
    let vli = li.map((v,i) => {
      if (!(startIndex<i && i<endIndex)) {
        return v;
      }
      return {
        value: cadenceMarkov.next_conditional({offset:endIndex-i, state:li[endIndex].value}),
        t: v.t
      };
    });
    return util.Timeline.fromItems(vli);
  }
}

class ChordVariantGenerator {
  generate(tune: Tune, timespan: [number, number], rand: Random, params: ChordGeneratorParameters): util.Timeline<Chord> {
    let [startIndex, endIndex] = timespan.map((t) => tune.cadence.getIndex(t))
    let li = tune.chord.list();
    let rand_tones = new util.WeightedRandom(rand, params.tone.rand_tones.items);
    let vli = li.map((v,i)=>{
      if (!(startIndex<i && i<endIndex)) {
        return v;
      }
      let cadence = tune.cadence.get(li[i].t).value;
      let probabilities_root = new Map<Cadence, util.WeightedItem<number>[]>(params.root.probabilities);
      let items_root = probabilities_root.get(cadence);   util.assertIsDefined(items_root);
      let rand_root = new util.WeightedRandom(rand, items_root);
      return {
        value: new Chord(tune.scale, rand_root.get(), rand_tones.get()),
        t: v.t
      }
    });
    return util.Timeline.fromItems(vli);
  }
}

class PitchVariantGenerator {
  generate(tune: Tune, timespan: [number, number], rand: Random, params: PitchWeightParameters): util.Timeline<Note> {
    const weightCalculator = new PitchWeightCalculator();
    let [startIndex, endIndex] = timespan.map((t) => tune.notes.getIndex(t));
    let li = tune.notes.list();
    let maxLengthToKey = Math.ceil(params.absPitchFactor.edge1);
    let candidates = [...util.rangeIterator(tune.scale.root-maxLengthToKey+1, tune.scale.root+maxLengthToKey)];
    let transition = new Map<number, util.WeightedItem<number>[]>();
    candidates.forEach(p => {
      transition.set(p, candidates.map(c => {return {value: c, weight: weightCalculator.calcTimeHomoWeight(tune, p, c, params)}}));
    });
    let chain = new markov.MarkovChain_TimeHomo_FiniteState(rand, 0, transition);
    let vli = li.map((v,i)=>{
      if (!(startIndex<i && i<endIndex)) {
        return v;
      }
      let items_timehomo = chain.vectorToWeightedItems(
        chain.calc_distribution_conditional(
          {offset: 1, state: li[i-1].value.pitch},
          {offset: endIndex-i, state: li[endIndex].value.pitch}
        )
      );
      let items = items_timehomo.map(item => {return {
        value: item.value,
        weight: weightCalculator.calcWithTimeHomoWeight(
          tune, item.weight, {pitch: item.value, duration:li[i].value.duration}, li[i].t, params
        ),
      }});
      let rand_pitch = new util.WeightedRandom(rand, items);
      return {
        t: v.t,
        value: {duration: v.value.duration, pitch: rand_pitch.get()}
        //value: {duration: v.value.duration, pitch: 64}
      }
    });
    return util.Timeline.fromItems(vli);
  }
}

class RhythmVariantGenerator {}
