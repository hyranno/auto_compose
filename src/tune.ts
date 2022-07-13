import {Random} from 'reliable-random';
import * as util from './util';

export class Tune {
  time_measure: [number, number];
  max_beat_division_depth: number;
  scale: Scale;
  cadence: util.Timeline<Cadence>;
  chord: util.Timeline<Chord>;
  //Timeline<Rhythm>
  //Timeline<Note>
}

type notenum = number; //MIDI note number on 12 equal temperament
type degree = number; //index on scale or chord

export function notenumToHerzs(pitch: notenum): number {
  return 440 * Math.pow(2, (pitch-69)/12);
}

abstract class RootedTones {
  root: number;
  tones: number[];
  constructor(root: number, tones: number[]) {
    this.root = root;
    this.tones = tones;
  }
  getNotenums(): notenum[] {
    return this.tones.map((h)=> this.root+h);
  }
  includes(note: notenum): boolean {
    return this.getNotenums().map(v => v%12).includes(note%12);
  }
  isRoot(note: notenum): boolean {
    return this.getNotenums()[0]%12 == note%12;
  }
  get(pitch: degree): notenum {
    let notenums = this.getNotenums();
    return notenums[pitch % notenums.length] + 12*Math.floor(pitch / notenums.length);
  }
}

export class Scale extends RootedTones {
  constructor(key: notenum, tones: notenum[]) {
    super(key, tones);
  }
  static major(key: notenum): Scale {
    return new Scale(key, [0,2,4,5,7,9,11]);
  }
  static minor(key: notenum): Scale {
    return new Scale(key, [0,2,3,5,7,8,10]);
  }
}

export const Cadence = {
  T: "Tonic",
  D: "Dominant",
  S: "Subdominant",
} as const;
export type Cadence = (typeof Cadence)[keyof (typeof Cadence)];

export class CadenceGenerator {
  probabilities: Map<Cadence, util.WeightedItem<Cadence>[]>;
  constructor() {
    this.probabilities = new Map<Cadence, util.WeightedItem<Cadence>[]>([
      [Cadence.T, [{value: Cadence.T, weight: 0.1}, {value: Cadence.D, weight: 0.6}, {value: Cadence.S, weight: 0.3}] ],
      [Cadence.D, [{value: Cadence.T, weight: 0.6}, {value: Cadence.D, weight: 0.1}, {value: Cadence.S, weight: 0.3}] ],
      [Cadence.S, [{value: Cadence.T, weight: 0.9}, {value: Cadence.D, weight: 0.0}, {value: Cadence.S, weight: 0.1}] ],
    ]); // TODO: from setting
  }
  generate(tune: Tune): util.Timeline<Cadence> {
    let times = [...util.rangeIterator(0, tune.time_measure[0] * tune.time_measure[1], 2)]; //TODO: interval from setting
    let cadences: Cadence[] = [Cadence.T, Cadence.D, ]; // how it end  //TODO: can change
    let chain = new util.MarkovChain<Cadence>(
      new Random(4649,459), //TODO: seed setting
      cadences[cadences.length-1],
      s => this.probabilities.get(s)
    );
    while (cadences.length < times.length) {
      cadences.push(chain.next());
    }
    let events: util.TimelineItem<Cadence>[] = cadences.reverse().map((v,i)=>{return {t:times[i], value:v}});
    return util.Timeline.fromItems(events);
  }
}

export class Chord extends RootedTones {
  scale: Scale;
  constructor(scale: Scale, root: degree, tones: degree[]) {
    super(root, tones);
    this.scale = scale;
  }
  override getNotenums(): number[] {
    return this.tones.map(h => this.scale.get(this.root+h));
  }
}

export class ChordGenerator {
  generateRoots(tune: Tune): util.Timeline<number> {
    let probabilities = new Map<Cadence, util.WeightedItem<number>[]>([
      [Cadence.T, [{value: 0, weight: 1}] ],
      [Cadence.D, [{value: 4, weight: 1}] ],
      [Cadence.S, [{value: 3, weight: 1}] ],
    ]); // TODO: from setting
    let rand = new Random(4649,459); //TODO: seed setting
    let events = tune.cadence.list().map( c => {return {
      t: c.t,
      value: (new util.WeightedRandom(rand, probabilities.get(c.value))).get(),
    }} );
    return util.Timeline.fromItems(events);
  }
  generate(tune: Tune): util.Timeline<Chord> {
    let roots = this.generateRoots(tune);
    let rand = new Random(4649,459); //TODO: seed setting
    let rand_tones = new util.WeightedRandom(rand, [
      {value:[0,2,4], weight: 1},
    ]); //TODO: setting
    let events = roots.list().map( c => {return {
      t: c.t,
      value: new Chord(
        tune.scale,
        c.value,
        rand_tones.get()
      )
    }} );
    return util.Timeline.fromItems(events);
  }
}
export class ChordModifier {}
