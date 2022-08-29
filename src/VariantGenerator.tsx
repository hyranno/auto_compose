import Ajv from "ajv"
import { Component } from 'solid-js';
import {Random} from 'reliable-random';

import schema from '../schemas/VariantGeneratorParameters.json';

import {Tune, Scale, Resolution, Cadence, Chord, Note, NoteTreeNode} from './tune';
import * as TuneGenerator from './TuneGenerator';
import * as CadenceGenerator from './CadenceGenerator';
import * as ChordGenerator from './ChordGenerator';
import * as NoteGenerator from './NoteGenerator';
import * as RhythmGenerator from './RhytmGenerator';
import * as PitchWeight from './PitchWeightCalculator';
import * as util from './util';
import * as helper from './solid_helper';
import * as markov from './markov';


export const Layer = {
  "Cadence": 0,
  "Chord": 1,
  "Pitch": 2,
  "Rhythm": 3,
} as const;
export type Layer = (typeof Layer)[keyof (typeof Layer)];


export type Parameters = {
  seed: util.RandomSeed;
  resolution: Resolution;
  timespan: util.WeightedItem<[number,number]>[];
  layer: util.WeightedItem<Layer>[];
};
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

function overrideTuneGeneratorParameters(params: TuneGenerator.Parameters, override: Partial<TuneGenerator.Parameters>): TuneGenerator.Parameters {
  return {
    scale: params.scale,
    time_measure: params.time_measure,
    resolution: override.resolution ?? params.resolution,
    cadence: override.cadence ?? params.cadence,
    chord: override.chord ?? params.chord,
    note: override.note ?? params.note,
  };
}

export function generate(params: Parameters, baseParams: TuneGenerator.Parameters): Tune {
  let rand = new Random(params.seed.state, params.seed.sequence);
  const timespan = (new util.WeightedRandom(rand, params.timespan)).get();
  const layer = (new util.WeightedRandom(rand, params.layer)).get();
  const tuneParams = overrideTuneGeneratorParameters(baseParams, {resolution: params.resolution});
  let tune = new Tune();
  tune.length = tuneParams.time_measure[0] * tuneParams.time_measure[1];
  tune.scale = new Scale(tuneParams.scale.key, tuneParams.scale.tones);
  tune.time_measure = tuneParams.time_measure;
  tune.max_beat_division_depth = tuneParams.note.rhythm.max_beat_division_depth;
  tune.resolution = util.Timeline.fromItems([{t:0, value:tuneParams.resolution}]);
  tune.cadence = CadenceGenerator.generate(tune, tuneParams.cadence);
  if (layer == Layer.Cadence) {
    tune.cadence = CadenceVariantGenerator.generate(tune, timespan, rand, tuneParams.cadence);
  }
  tune.chord = ChordGenerator.generate(tune, tuneParams.chord);
  if (layer == Layer.Chord) {
    tune.chord = ChordVariantGenerator.generate(tune, timespan, rand, tuneParams.chord);
  }
  tune.notes = NoteGenerator.generate(tune, tuneParams.note);
  if (layer == Layer.Pitch) {
    tune.notes = PitchVariantGenerator.generate(tune, timespan, rand, tuneParams.note);
  }
  if (layer == Layer.Rhythm) {
    tune.notes = RhythmVariantGenerator.generate(tune, timespan, rand, tuneParams.note);
  }
  return tune;
  }


namespace CadenceVariantGenerator {
  export function generate(tune: Tune, timespan: [number, number], rand: Random, params: CadenceGenerator.Parameters): util.Timeline<Cadence> {
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

namespace ChordVariantGenerator {
  export function generate(tune: Tune, timespan: [number, number], rand: Random, params: ChordGenerator.Parameters): util.Timeline<Chord> {
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

namespace PitchVariantGenerator {
  export function interpolate(tune: Tune, start: number, end: number, rand: Random, dest: NoteTreeNode[], params: PitchWeight.Parameters) {
    const maxLengthToKey = Math.ceil(params.absPitchFactor.edge1);
    const candidates = [...util.rangeIterator(tune.scale.root-maxLengthToKey+1, tune.scale.root+maxLengthToKey)];
    let transition = new Map<number, util.WeightedItem<number>[]>();
    candidates.forEach(p => {
      transition.set(p, candidates.map(c => {return {value: c, weight: PitchWeight.calcTimeHomoWeight(tune, p, c, params)}}));
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
        weight: PitchWeight.calcWithTimeHomoWeight(
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
  export function generate(tune: Tune, timespan: [number, number], rand: Random, params: NoteGenerator.Parameters): util.Timeline<Note> {
    const [startIndex, endIndex] = timespan.map((t) => tune.notes.getIndex(t));
    let notes = NoteGenerator.generateTree(tune, params);
    let leaves = notes.leaves();
    const [startPitch, endPitch] = [startIndex, endIndex].map(i => leaves[i].pitch);
    util.assertIsDefined(startPitch); util.assertIsDefined(endPitch);
    let dest = leaves.slice(startIndex+1, endIndex);
    this.interpolate(tune, startPitch, endPitch, rand, dest, params.pitchWeight);
    return util.Timeline.fromItems( leaves.map(l => l.toTimelineItem()) );
  }
}

namespace RhythmVariantGenerator {
  export function splitRhythm(tune: Tune, root: NoteTreeNode, timespan: [number, number], rand: Random, params: NoteGenerator.Parameters) {
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
        PitchVariantGenerator.interpolate(tune, start, end, rand, leaf.children, params.pitchWeight);
      }
    });
  }
  export function mergeRhythm(tune: Tune, root: NoteTreeNode, timespan: [number, number], rand: Random, params: NoteGenerator.Parameters) {
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
        PitchVariantGenerator.interpolate(tune, start, end, rand, [branch], params.pitchWeight);
      }
    });
  }
  export function toggleNoteOn(root: NoteTreeNode, timespan: [number, number], rand: Random, params: RhythmGenerator.Parameters) {
    let leaves = root.leaves();
    leaves.forEach(leaf => {
      if (!(timespan[0] < leaf.t && leaf.t < timespan[1])) {
        return;
      }
      const pNoteOn = util.smoothstep(params.pNoteOn.edge0, params.pNoteOn.edge1, leaf.duration);
      leaf.rhythm.isNoteOn = rand.random() < pNoteOn;
    });
  }
  export function generate(tune: Tune, timespan: [number, number], rand: Random, params: NoteGenerator.Parameters): util.Timeline<Note> {
    let notes = NoteGenerator.generateTree(tune, params);
    this.splitRhythm(tune, notes, timespan, rand, params);
    this.mergeRhythm(tune, notes, timespan, rand, params);
    this.toggleNoteOn(notes, timespan, rand, params.rhythm);
    return util.Timeline.fromItems(notes.leaves().map(l => l.toTimelineItem()));
  }
}
