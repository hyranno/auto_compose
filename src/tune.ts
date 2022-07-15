import {Random} from 'reliable-random';
import * as util from './util';

export class Tune {
  time_measure: [number, number];
  max_beat_division_depth: number;
  scale: Scale;
  cadence: util.Timeline<Cadence>;
  chord: util.Timeline<Chord>;
  notes: util.Timeline<Note>;
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

type Rhythm = {
  isNoteOn: boolean,
  t: number,
  duration: number,
}
export class RhythmGenerator {
  generateNode(rand: Random, depthcount: number, dest: Rhythm[], candidate: Rhythm) {
    let pLeaf = 1 - util.smoothstep(1/2, 3, candidate.duration); //TODO: setting
    let isLeaf = (rand.random() < pLeaf) || (depthcount <= 0);
    if (isLeaf) {
      let pNoteOn = util.smoothstep(0, 1.2, candidate.duration);//TODO: setting
      let isNoteOn = rand.random() < pNoteOn;
      candidate.isNoteOn = isNoteOn;
      dest.push(candidate);
    } else {
      let c0 = {isNoteOn: true, t: candidate.t, duration: candidate.duration/2};
      let c1 = {isNoteOn: true, t: c0.t + c0.duration, duration: c0.duration};
      this.generateNode(rand, depthcount-1, dest, c0);
      this.generateNode(rand, depthcount-1, dest, c1);
    }
  }
  generate(tune: Tune): Rhythm[] {
    let rand = new Random(4649,459); //TODO: seed setting
    let res = [];
    [...util.rangeIterator(0,tune.time_measure[1],1)].forEach( i =>
      this.generateNode(rand, 3, res,
        {isNoteOn:true, t:i*tune.time_measure[0], duration: tune.time_measure[0]}
      )
    );
    return res;
  }
}

export type Note = {
  pitch: notenum;
  duration: number;
}
export class NoteGenerator {
  calcAbsolutePitchFactor(centor: notenum, candidate: Note): number {
    let distance = Math.abs(candidate.pitch - centor);
    return 1 - util.smoothstep(4,12, distance); //TODO: setting
  }
  calcRelativePitchFactor(prev: Note, candidate: Note): number {
    let distance = Math.abs(candidate.pitch - prev.pitch);
    return 1 - util.smoothstep(6,12, distance); //TODO: setting
  }
  calcFactorInScale(scale: Scale, candidate: Note): number {
    let Cs = 8; //TODO: setting
    return scale.includes(candidate.pitch)? Cs : 1;
  }
  calcFactorInChord(chord: Chord, candidate: Note): number {
    let Cc = 4; //TODO: setting
    return chord.includes(candidate.pitch)? Cc : 1;
  }
  calcRhythmModifier(duration: number): number {
    return duration / 2; //TODO: setting
  }
  calcWeight(tune: Tune, t: number, prev: Note, candidate: Note): number {
    let absPitchFactor = this.calcAbsolutePitchFactor(tune.scale.root, candidate);
    let relPitchFactor = this.calcRelativePitchFactor(prev, candidate);
    let factorInScale = this.calcFactorInScale(tune.scale, candidate);
    let factorInChord = this.calcFactorInChord(tune.chord.get(t).value, candidate);
    let rhythmModifier = this.calcRhythmModifier(candidate.duration);
    let regularity = 1; //TODO: setting
    return Math.pow(
      absPitchFactor * relPitchFactor * factorInScale * factorInChord,
      rhythmModifier + regularity
    );
  }
  generateNote(rand: Random, tune: Tune, prev: Note, rhythm: Rhythm): Note{
    let candidates: Note[] = [...util.rangeIterator(prev.pitch-12, prev.pitch+12, 1)].map(p => {return {
      pitch: p,
      duration: rhythm.duration
    }});
    let rand_notes = new util.WeightedRandom(
      rand,
      candidates.map((c) => {return {weight: this.calcWeight(tune, rhythm.t, prev, c), value: c}})
    );
    return rand_notes.get();
  }
  generate(tune: Tune): util.Timeline<Note> {
    let rhythmGen = new RhythmGenerator();
    let rhythms = rhythmGen.generate(tune);
    let rand = new Random(4649,459); //TODO: seed setting
    let prev = {pitch: tune.scale.root, duration: 0}; //dummy
    let events = rhythms.map(r => {
      prev = this.generateNote(rand, tune, prev, r);
      return {t:r.t, value:prev};
    });
    return util.Timeline.fromItems(events);
  }
}
