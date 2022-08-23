import Ajv from "ajv"
import { Component, createSignal } from 'solid-js';
import {Random} from 'reliable-random';

import schema from '../schemas/VariantGeneratorParameters.json';

import {Tune, Scale, Resolution, Cadence, Chord, Note} from './tune';
import {TuneGenerator, TuneGeneratorParameters} from './TuneGenerator';
import {CadenceGenerator, CadenceGeneratorParameters} from './CadenceGenerator';
import {ChordGenerator, ChordGeneratorParameters} from './ChordGenerator';
import {NoteGenerator, NoteGeneratorParameters, NoteTreeNode} from './NoteGenerator';
import {RhythmGenerator, RhythmGeneratorParameters} from './NoteGenerator';
import {PitchWeightCalculator, PitchWeightParameters} from './NoteGenerator';
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
  resolution: Resolution;
  timespan: util.WeightedItem<[number,number]>[];
  layer: util.WeightedItem<VariantGeneratorLayer>[];
};
export class VariantGeneratorParametersUiAdapter extends helper.UiAdapter<VariantGeneratorParameters> {
  private seed = new helper.RandomSeed(4649,459);
  private resolution: Resolution = Resolution.Perfect;
  private timespan: util.WeightedItem<[number,number]>[];
  private layer: util.WeightedItem<VariantGeneratorLayer>[];
  get(): VariantGeneratorParameters {
    return {
      seed: this.seed.get(),
      resolution: this.resolution,
      timespan: this.timespan,
      layer: this.layer,
    };
  }
  set(v: VariantGeneratorParameters): VariantGeneratorParameters {
    this.seed.set(v.seed);
    this.resolution = v.resolution;
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

function overrideTuneGeneratorParameters(params: TuneGeneratorParameters, override: Partial<TuneGeneratorParameters>): TuneGeneratorParameters {
  return {
    scale: params.scale,
    time_measure: params.time_measure,
    max_beat_division_depth: params.max_beat_division_depth,
    resolution: override.resolution ?? params.resolution,
    cadence: override.cadence ?? params.cadence,
    chord: override.chord ?? params.chord,
    note: override.note ?? params.note,
  };
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
    const timespan = (new util.WeightedRandom(rand, params.timespan)).get();
    const layer = (new util.WeightedRandom(rand, params.layer)).get();
    const tuneParams = overrideTuneGeneratorParameters(baseParams, {resolution: params.resolution});
    let tune = new Tune();
    tune.length = tuneParams.time_measure[0] * tuneParams.time_measure[1];
    tune.scale = new Scale(tuneParams.scale.key, tuneParams.scale.tones);
    tune.time_measure = tuneParams.time_measure;
    tune.max_beat_division_depth = tuneParams.max_beat_division_depth;

    tune.resolution = util.Timeline.fromItems([{t:0, value:tuneParams.resolution}]);
    tune.cadence = (new CadenceGenerator()).generate(tune, tuneParams.cadence);
    if (layer == VariantGeneratorLayer.Cadence) {
      tune.cadence = (new CadenceVariantGenerator()).generate(tune, timespan, rand, tuneParams.cadence);
    }
    tune.chord = (new ChordGenerator()).generate(tune, tuneParams.chord);
    if (layer == VariantGeneratorLayer.Chord) {
      tune.chord = (new ChordVariantGenerator()).generate(tune, timespan, rand, tuneParams.chord);
    }
    tune.notes = (new NoteGenerator()).generate(tune, tuneParams.note);
    if (layer == VariantGeneratorLayer.Pitch) {
      tune.notes = (new PitchVariantGenerator()).generate(tune, timespan, rand, tuneParams.note);
    }
    if (layer == VariantGeneratorLayer.Rhythm) {
      tune.notes = (new RhythmVariantGenerator()).generate(tune, timespan, rand, tuneParams.note);
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
  interpolate(tune: Tune, start: number, end: number, rand: Random, dest: NoteTreeNode[], params: PitchWeightParameters) {
    const weightCalculator = new PitchWeightCalculator();
    const maxLengthToKey = Math.ceil(params.absPitchFactor.edge1);
    const candidates = [...util.rangeIterator(tune.scale.root-maxLengthToKey+1, tune.scale.root+maxLengthToKey)];
    let transition = new Map<number, util.WeightedItem<number>[]>();
    candidates.forEach(p => {
      transition.set(p, candidates.map(c => {return {value: c, weight: weightCalculator.calcTimeHomoWeight(tune, p, c, params)}}));
    });
    let chain = new markov.MarkovChain_TimeHomo_FiniteState(rand, 0, transition);
    let prev = start;
    dest.forEach((note,i) => {
      const items_timehomo = chain.vectorToWeightedItems(
        chain.calc_distribution_conditional(
          {offset: 1, state: prev},
          {offset: dest.length+1-i, state: end}
        )
      );
      let items = items_timehomo.map(item => {return {
        value: item.value,
        weight: weightCalculator.calcWithTimeHomoWeight(
          tune, item.weight, {
            pitch: item.value,
            isNoteOn: note.isNoteOn,
            duration: note.duration
          }, note.t, params
        ),
      }});
      let rand_pitch = new util.WeightedRandom(rand, items);
      note.pitch = rand_pitch.get();
    });
  }
  generate(tune: Tune, timespan: [number, number], rand: Random, params: NoteGeneratorParameters): util.Timeline<Note> {
    const [startIndex, endIndex] = timespan.map((t) => tune.notes.getIndex(t));
    let notes = (new NoteGenerator()).generateTree(tune, params);
    let leaves = notes.leaves();
    const [startPitch, endPitch] = [startIndex, endIndex].map(i => leaves[i].pitch);
    util.assertIsDefined(startPitch); util.assertIsDefined(endPitch);
    let dest = leaves.slice(startIndex+1, endIndex);
    this.interpolate(tune, startPitch, endPitch, rand, dest, params.pitchWeight);
    return util.Timeline.fromItems( leaves.map(l => l.toTimelineItem()) );
  }
}

class RhythmVariantGenerator {
  splitRhythm(tune: Tune, root: NoteTreeNode, timespan: [number, number], rand: Random, params: NoteGeneratorParameters) {
    const numChildren = 2;
    let leaves = root.leaves();
    leaves.forEach((leaf,i) => {
      if (!(timespan[0] < leaf.t && leaf.t < timespan[1])) {
        return;
      }
      const pSplit = util.smoothstep(params.rhythm.pBranch.edge0, params.rhythm.pBranch.edge1, leaf.duration);
      if (rand.random() < pSplit) {
        const subDuration = leaf.duration / numChildren;
        leaf.children.push(
          ...[...util.rangeIterator(0,numChildren)].map(i => new NoteTreeNode(
            {t: leaf.t+i*subDuration, duration: subDuration, isNoteOn: true}
          ))
        );
        const [start, end] = [leaves[i-1].pitch, leaves[i+1].pitch];
        util.assertIsDefined(start); util.assertIsDefined(end);
        (new PitchVariantGenerator()).interpolate(tune, start, end, rand, leaf.children, params.pitchWeight);
      }
    });
  }
  mergeRhythm(tune: Tune, root: NoteTreeNode, timespan: [number, number], rand: Random, params: NoteGeneratorParameters) {
    const numChildren = 2;
    const leaves = root.leaves();
    let targets = root.flat().filter(n => n.children.length == numChildren);
    targets.forEach(branch => {
      if (!(timespan[0] < branch.t && branch.t < timespan[1])) {
        return;
      }
      const pMerge = 1 - util.smoothstep(params.rhythm.pBranch.edge0, params.rhythm.pBranch.edge1, branch.duration);
      if (rand.random() < pMerge) {
        const [i0, i1] = [0,1].map(i => leaves.findIndex(l => l.t == branch.children[i].t));
        const [start, end] = [leaves[i0-1].pitch, leaves[i1+1].pitch];
        util.assertIsDefined(start); util.assertIsDefined(end);
        branch.children = [];
        (new PitchVariantGenerator()).interpolate(tune, start, end, rand, [branch], params.pitchWeight);
      }
    });
  }
  toggleNoteOn(root: NoteTreeNode, timespan: [number, number], rand: Random, params: RhythmGeneratorParameters) {
    let leaves = root.leaves();
    leaves.forEach(leaf => {
      if (!(timespan[0] < leaf.t && leaf.t < timespan[1])) {
        return;
      }
      const pNoteOn = util.smoothstep(params.pNoteOn.edge0, params.pNoteOn.edge1, leaf.duration);
      leaf.rhythm.isNoteOn = rand.random() < pNoteOn;
    });
  }
  generate(tune: Tune, timespan: [number, number], rand: Random, params: NoteGeneratorParameters): util.Timeline<Note> {
    let notes = (new NoteGenerator()).generateTree(tune, params);
    this.splitRhythm(tune, notes, timespan, rand, params);
    this.mergeRhythm(tune, notes, timespan, rand, params);
    this.toggleNoteOn(notes, timespan, rand, params.rhythm);
    return util.Timeline.fromItems(notes.leaves().map(l => l.toTimelineItem()));
  }
}
