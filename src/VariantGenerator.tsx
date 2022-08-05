import {Random} from 'reliable-random';

import {Tune, Scale, Cadence, Chord, Note} from './tune';
import {TuneGeneratorParameters} from './TuneGenerator';
import {CadenceGenerator, CadenceGeneratorParameters} from './CadenceGenerator';
import {ChordGenerator, ChordGeneratorParameters} from './ChordGenerator';
import {NoteGenerator, NoteGeneratorParameters} from './NoteGenerator';
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
export class VariantGenerator {
  private params: VariantGeneratorParameters;
  private tuneParameters: TuneGeneratorParameters;
  private cadenceGen = new CadenceGenerator();
  private chordGen = new ChordGenerator();
  private noteGen = new NoteGenerator();
  get: ()=>Tune;
  private set: (v: Tune)=>Tune;
  generate(): Tune {
    let rand = new Random(this.params.seed.state, this.params.seed.sequence);
    let timespan = (new util.WeightedRandom(rand, this.params.timespan)).get();
    let layer = (new util.WeightedRandom(rand, this.params.layer)).get();
    let tune = new Tune();
    tune.scale = new Scale(this.tuneParameters.scale.key, this.tuneParameters.scale.tones);
    tune.time_measure = this.tuneParameters.time_measure;
    tune.max_beat_division_depth = this.tuneParameters.max_beat_division_depth;
    tune.cadence = this.cadenceGen.generate(tune);
    if (layer == VariantGeneratorLayer.Cadence) {
      //modify cadence
    }
    tune.chord = this.chordGen.generate(tune);
    if (layer == VariantGeneratorLayer.Chord) {
      //modify cadence
    }
    tune.notes = this.noteGen.generate(tune);
    if (layer == VariantGeneratorLayer.Pitch) {
      //modify cadence
    }
    if (layer == VariantGeneratorLayer.Rhythm) {
      //modify rhythm
    }
    return this.set(tune);
  }
}


class CadenceVariantGenerator {
  private orginalParams: CadenceGeneratorParameters;
  generate(tune: Tune, timespan: [number, number], rand: Random): util.Timeline<Cadence> {
    let [startIndex, endIndex] = timespan.map((t) => tune.cadence.getIndex(t))
    let li = tune.cadence.list();
    let cadenceMarkov = new markov.MarkovChain_TimeHomo_FiniteState(
      rand, li[startIndex].value,
      new Map<Cadence, util.WeightedItem<Cadence>[]>(this.orginalParams.probabilities)
    );
    for (let i=startIndex+1; i<endIndex; i++) {
      li[i].value = cadenceMarkov.next_conditional({offset:endIndex-i, state:li[endIndex].value});
    }
    return util.Timeline.fromItems(li);
  }
}

class ChordVariantGenerator {
  private originalParams: ChordGeneratorParameters;
  generate(tune: Tune, timespan: [number, number], rand: Random): util.Timeline<Chord> {
    let rand_tones = new util.WeightedRandom(rand, this.originalParams.tone.rand_tones.items);
    let [startIndex, endIndex] = timespan.map((t) => tune.cadence.getIndex(t))
    let li = tune.chord.list();
    for (let i=startIndex+1; i<endIndex; i++) {
      let cadence = tune.cadence.get(li[i].t).value;
      let probabilities_root = new Map<Cadence, util.WeightedItem<number>[]>(this.originalParams.root.probabilities);
      let items_root = probabilities_root.get(cadence);   util.assertIsDefined(items_root);
      let rand_root = new util.WeightedRandom(rand, items_root);
      li[i].value = new Chord(tune.scale, rand_root.get(), rand_tones.get());
    }
    return util.Timeline.fromItems(li);
  }
}

class PitchVariantGenerator {
  //generate(tune: Tune, part: [number, number]): util.Timeline<Note> {}
}

class RhythmVariantGenerator {}
