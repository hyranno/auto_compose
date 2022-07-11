import {Random} from 'reliable-random';
import * as util from './util';

export class Tune {
  time_measure: [number, number];
  max_beat_division_depth: number;
  key: number;
  scale: Scale;
  cadence: util.Timeline<Cadence>;
  //chord: util.Timeline<Chord>;
  //Timeline<Rhythm>
  //Timeline<Note>
}


export class Scale {
  notes: number[];
  constructor(key: number, heights: number[]) {
    this.notes = heights.map((h)=> (key+h)%12);
  }
  static major(key: number): Scale {
    return new Scale(key, [0,2,4,5,7,9,11]);
  }
  static minor(key: number): Scale {
    return new Scale(key, [0,2,3,5,7,8,10]);
  }
  includes(note: number): boolean {
    return this.notes.includes(note%12);
  }
  isRoot(note: number): boolean {
    return this.notes[0] == note%12;
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

export class Chord {
  notes: number[];
  constructor(scale: Scale, root: number, heights: number[]) {
  }
  includes(note: number): boolean {
    return this.notes.includes(note%12);
  }
  isRoot(note: number): boolean {
    return this.notes[0] == note%12;
  }
}
