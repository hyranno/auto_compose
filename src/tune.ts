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

export type notenum = number; //MIDI note number on 12 equal temperament
export type degree = number; //index on scale or chord

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

export type Note = {
  pitch: notenum;
  duration: number;
}
