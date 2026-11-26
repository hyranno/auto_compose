import { describe, test, expect } from 'vitest';
import * as vectorious from 'vectorious';
import {Random} from 'reliable-random';

import * as util from './util';
import * as tunes from './tune';


describe("chord", ()=>{
  test("rot", ()=>{
    let scale = tunes.Scale.major(69);
    let org = new tunes.Chord(scale, 4, [0,2,4]);
    expect(org.getNotenums()).toEqual([76,80,83]);
    expect(org.rot(scale,1).getNotenums()).toEqual([88,80,83]);
    expect(org.rot(scale,2).getNotenums()).toEqual([88,92,83]);
    expect(org.rot(scale,3).getNotenums()).toEqual([88,92,95]);
    expect(org.rot(scale,4).getNotenums()).toEqual([100,92,95]);
    expect(org.rot(scale,5).getNotenums()).toEqual([100,104,95]);
    expect(org.rot(scale,6).getNotenums()).toEqual([100,104,107]);
    expect(org.rot(scale,-1).getNotenums()).toEqual([76,80,71]);
    expect(org.rot(scale,-2).getNotenums()).toEqual([76,68,71]);
    expect(org.rot(scale,-3).getNotenums()).toEqual([64,68,71]);
    expect(org.rot(scale,-4).getNotenums()).toEqual([64,68,59]);
    expect(org.rot(scale,-5).getNotenums()).toEqual([64,56,59]);
    expect(org.rot(scale,-6).getNotenums()).toEqual([52,56,59]);
  });
});

describe("Scale", () => {
  const scale = tunes.Scale.major(60);
  test("get wraps degrees into octaves", () => {
    // [degree, expected notenum]
    const cases: [number, number][] = [
      [0, 60], [1, 62], [6, 71], [7, 72], [8, 74], [14, 84], [-1, 59], [-7, 48], [-8, 47],
    ];
    expect(cases.map(([d]) => scale.get(d))).toStrictEqual(cases.map(c => c[1]));
  });
  test("minor", () => {
    expect(tunes.Scale.minor(57).getNotenums()).toStrictEqual([57, 59, 60, 62, 64, 65, 67]);
  });
  test("includes and isRoot ignore the octave", () => {
    const notes = [48, 60, 61, 71, 72, 73, 83];
    expect(notes.map(n => scale.includes(n))).toStrictEqual([true, true, false, true, true, false, true]);
    expect(notes.map(n => scale.isRoot(n))).toStrictEqual([true, true, false, false, true, false, false]);
  });
  test("center is the mean of the notenums", () => {
    expect(scale.center()).toBe(458 / 7);
  });
});

describe("Chord", () => {
  const scale = tunes.Scale.major(60);
  test("getNotenums takes the degrees on the scale", () => {
    // [root, tones, expected]
    const cases: [number, number[], number[]][] = [
      [0, [0, 2, 4], [60, 64, 67]],
      [4, [0, 2, 4, 6], [67, 71, 74, 77]],
      [5, [0, 2, 4], [69, 72, 76]],
      [6, [0, 4], [71, 77]],
      [-1, [0, 2, 4], [59, 62, 65]],
    ];
    expect(cases.map(([r, t]) => new tunes.Chord(scale, r, t).getNotenums())).toStrictEqual(cases.map(c => c[2]));
  });
  test("includes, isRoot and center use the chord notenums", () => {
    const chord = new tunes.Chord(scale, 5, [0, 2, 4]); // A minor: 69, 72, 76
    expect([57, 60, 64, 65].map(n => chord.includes(n))).toStrictEqual([true, true, true, false]);
    expect([57, 81, 60].map(n => chord.isRoot(n))).toStrictEqual([true, true, false]);
    expect(chord.center()).toBe(217 / 3);
  });
});

describe("Tune", () => {
  test("merge appends the source shifted by the length", () => {
    const a = new tunes.Tune();
    a.length = 4;
    a.scale = tunes.Scale.major(60);
    a.cadence = util.Timeline.fromItems([{t: 0, value: tunes.Cadence.T}, {t: 2, value: tunes.Cadence.D}]);
    a.chord = util.Timeline.fromItems([{t: 0, value: new tunes.Chord(a.scale, 0, [0, 2, 4])}]);
    a.notes = util.Timeline.fromItems([{t: 0, value: {pitch: 60, duration: 4, isNoteOn: true}}]);
    const b = new tunes.Tune();
    b.length = 2;
    b.cadence = util.Timeline.fromItems([{t: 0, value: tunes.Cadence.S}]);
    b.chord = util.Timeline.fromItems([{t: 0, value: new tunes.Chord(a.scale, 3, [0, 2, 4])}]);
    b.notes = util.Timeline.fromItems([{t: 1, value: {pitch: 65, duration: 1, isNoteOn: false}}]);

    const m = a.merge(b);
    expect(m.length).toBe(6);
    expect(m.scale.getNotenums()).toStrictEqual(a.scale.getNotenums());
    expect(m.cadence.list()).toStrictEqual([
      {t: 0, value: tunes.Cadence.T}, {t: 2, value: tunes.Cadence.D}, {t: 4, value: tunes.Cadence.S},
    ]);
    expect(m.chord.list().map(e => [e.t, e.value.root])).toStrictEqual([[0, 0], [4, 3]]);
    expect(m.notes.list().map(e => [e.t, e.value.pitch])).toStrictEqual([[0, 60], [5, 65]]);
    // resolution is not merged.
    expect(m.resolution.list()).toStrictEqual([]);
  });
});
