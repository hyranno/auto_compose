import * as util from './util';

export class Tune {
  length: number = 0;
  time_measure: [number, number] = [4,4];
  max_beat_division_depth: number = 3;
  scale: Scale = Scale.major(64);
  resolution = new util.Timeline<Resolution>();
  cadence = new util.Timeline<Cadence>();
  chord = new util.Timeline<Chord>();
  notes = new util.Timeline<Note>();
  merge(src: Tune): Tune {
    let dest = new Tune();
    dest.length = this.length + src.length;
    dest.time_measure = this.time_measure;
    dest.max_beat_division_depth = this.max_beat_division_depth;
    dest.scale = new Scale(this.scale.root, this.scale.tones);
    dest.cadence.merge(0, this.cadence);
    dest.cadence.merge(this.length, src.cadence);
    dest.chord.merge(0, this.chord);
    dest.chord.merge(this.length, src.chord);
    dest.notes.merge(0, this.notes);
    dest.notes.merge(this.length, src.notes);
    return dest;
  }
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
    return notenums[util.modulo(pitch, notenums.length)] + 12*Math.floor(pitch / notenums.length);
  }
  center(): number {
    let notenums = this.getNotenums();
    return notenums.reduce((prev, curr) => prev+curr) / notenums.length;
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

export const Resolution = {
  Perfect: "Perfect authentic",
  Imperfect: "Imperfect authentic",
  Deceptive: "Deceptive",
  Plagal: "Plagal",
  Half: "Half",
} as const;
export type Resolution = (typeof Resolution)[keyof (typeof Resolution)];

export const Cadence = {
  T: "Tonic",
  D: "Dominant",
  S: "Subdominant",
} as const;
export type Cadence = (typeof Cadence)[keyof (typeof Cadence)];
export const CadencePostfix: Map<Resolution, Cadence[]> = new Map([
  [Resolution.Perfect, [Cadence.D, Cadence.T]],
  [Resolution.Imperfect, [Cadence.D, Cadence.T]],
  [Resolution.Deceptive, [Cadence.D, Cadence.T]],
  [Resolution.Plagal, [Cadence.S, Cadence.T]],
  [Resolution.Half, [Cadence.D]],
]);

export class Chord extends RootedTones {
  scale: Scale;
  constructor(scale: Scale, root: degree, tones: degree[]) {
    super(root, tones);
    this.scale = scale;
  }
  override getNotenums(): notenum[] {
    return this.tones.map(h => this.scale.get(this.root+h));
  }
  rot(scale: Scale, times: number): Chord {
    const octave = Math.floor(times / this.tones.length);
    const rest = util.modulo(times, this.tones.length);
    let rot_tones: degree[] = this.tones.map((h,i) => {
      const p = (i < rest)? 1 : 0;
      return h + (octave+p)*scale.tones.length;
    });
    return new Chord(scale, this.root, rot_tones);
  }
}

export type Note = {
  pitch: notenum;
  duration: number;
  isNoteOn: boolean;
}
