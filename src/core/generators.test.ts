// Unit tests of the generators. Inputs and expectations are kept in tables so that
// they can be ported to the Rust tests as they are.
import { describe, test, expect } from 'vitest';

import type {Rng, GenerateContext} from './random';
import {ScriptedRng, recordingRngFactory} from './random';
import type {RngRecord} from './random';
import * as util from './util';
import {Tune, Scale, Chord, Cadence, CadencePostfix, Resolution} from './tune';
import * as CadenceGenerator from './cadence';
import * as ChordGenerator from './chord';
import * as RhythmGenerator from './rhythm';
import * as PitchWeight from './pitch_weight';
import * as NoteGenerator from './note';
import * as TuneGenerator from './tune_generator';
import * as VariantGenerator from './variant';

import defaultJson from '../../resources/TuneGenParams_default.json';
import simpleJson from '../../resources/TuneGenParams_simple.json';

const defaultParams = defaultJson as TuneGenerator.Parameters;
const simpleParams = simpleJson as TuneGenerator.Parameters;

/** Returns the same value forever and counts the draws. */
class ConstantRng implements Rng {
  consumed = 0;
  constructor(private readonly value: number) {}
  random(): number { this.consumed++; return this.value; }
}
const contextOf = (rng: Rng): GenerateContext => ({rngFactory: () => rng});

function emptyTune(timeMeasure: [number, number], resolution: Resolution = Resolution.Half, depth = 3): Tune {
  const tune = new Tune();
  tune.time_measure = timeMeasure;
  tune.length = timeMeasure[0] * timeMeasure[1];
  tune.scale = new Scale(defaultParams.scale.key, defaultParams.scale.tones);
  tune.max_beat_division_depth = depth;
  tune.resolution = util.Timeline.fromItems([{t: 0, value: resolution}]);
  return tune;
}
const seeds: [number, number][] = [[4649, 459], [1, 0], [-1, -7], [13, 21], [100, 3]];
const withSeed = <T extends {seed: util.RandomSeed}>(p: T, [state, sequence]: [number, number]): T =>
  ({...p, seed: {state, sequence}});

describe('cadence', () => {
  const resolutions = Object.values(Resolution);
  const measures: [number, number][] = [[4, 4], [3, 4], [2, 4]];
  const durations = [1, 2];
  const cases = resolutions.flatMap(r => measures.flatMap(m => durations.map(d => [r, m, d] as const)));
  test.each(cases)('%s %j duration=%d', (resolution, measure, duration) => {
    const tune = emptyTune(measure, resolution);
    const postfix = CadencePostfix.get(resolution)!;
    for (const seed of seeds) {
      const params = withSeed({...defaultParams.cadence, duration}, seed);
      const list = CadenceGenerator.generate(tune, params).list();
      expect(list.length).toBe(measure[0] * measure[1] / duration);
      expect(list.map(e => e.t)).toStrictEqual(list.map((_e, i) => i * duration));
      expect(list[0].value).toBe(Cadence.T);
      expect(list.slice(-postfix.length).map(e => e.value)).toStrictEqual(postfix);
    }
  });
  test('draws once per slot between the prefix and the postfix', () => {
    // [resolution, expected draws] for 4/4 with duration 2 (8 slots, prefix of 1)
    const cases: [Resolution, number][] = [[Resolution.Half, 6], [Resolution.Perfect, 5], [Resolution.Plagal, 5]];
    for (const [resolution, draws] of cases) {
      const rng = new ConstantRng(0);
      const list = CadenceGenerator.generate(emptyTune([4, 4], resolution), defaultParams.cadence, contextOf(rng)).list();
      expect(rng.consumed).toBe(draws);
      // A draw of 0 always picks the first state (Tonic).
      expect(list.slice(1, 1 + draws).every(e => e.value == Cadence.T)).toBe(true);
    }
  });
  test('the prefix wins when there are fewer slots than prefix + postfix', () => {
    const tune = emptyTune([2, 2], Resolution.Perfect); // 2 slots, postfix [D, T]
    const list = CadenceGenerator.generate(tune, defaultParams.cadence, contextOf(new ScriptedRng([]))).list();
    expect(list.map(e => e.value)).toStrictEqual([Cadence.T, Cadence.T]);
  });
  test('the slot before the postfix never transitions to an impossible state', () => {
    // Dominant -> Subdominant has probability 0, so the slot just before a Plagal postfix [S, T]
    // is never Dominant.
    const tune = emptyTune([4, 4], Resolution.Plagal);
    for (let s = 0; s < 50; s++) {
      const list = CadenceGenerator.generate(tune, withSeed(defaultParams.cadence, [s, 1])).list();
      expect(list[list.length - 3].value).not.toBe(Cadence.D);
    }
  });
});

describe('chord', () => {
  const tune = emptyTune([4, 4]);
  tune.cadence = util.Timeline.fromItems(
    [Cadence.T, Cadence.S, Cadence.D, Cadence.T].map((value, i) => ({t: i * 4, value}))
  );
  test('scripted draws pick root and tones from the tables', () => {
    // Root draws (one per cadence): T 0.7 -> 0 (ratio exactly 0.7), S 0.95 -> 1 (0.6 + 0.35 == 0.95),
    // D 0.9 -> 6, T 1-2^-32 -> 2
    const rootRng = new ScriptedRng([0.7, 0.95, 0.9, 1 - 2 ** -32]);
    // Tone draws (one per chord): 0.8 -> [0,2,4], 0.81 -> [0,2,4,6],
    // 0.95 -> [0,2,4,6] (0.8 + 0.15 == 0.9500000000000001), 0.96 -> [0,4]
    const toneRng = new ScriptedRng([0.8, 0.81, 0.95, 0.96]);
    const ctx: GenerateContext = {rngFactory: (_s, label) => label == 'chord.root' ? rootRng : toneRng};
    const list = ChordGenerator.generate(tune, defaultParams.chord, ctx).list();
    expect(list.map(e => [e.t, e.value.root, e.value.tones])).toStrictEqual([
      [0, 0, [0, 2, 4]], [4, 1, [0, 2, 4, 6]], [8, 6, [0, 2, 4, 6]], [12, 2, [0, 4]],
    ]);
    expect([rootRng.consumed, toneRng.consumed]).toStrictEqual([4, 4]);
  });
});

describe('rhythm', () => {
  const measures: [number, number][] = [[4, 4], [3, 4]];
  const depths = [0, 1, 2, 3];
  const cases = measures.flatMap(m => depths.map(d => [m, d] as const));
  test.each(cases)('%j depth=%d', (measure, depth) => {
    const tune = emptyTune(measure, Resolution.Half, depth);
    for (const seed of seeds) {
      const root = RhythmGenerator.generate(tune, withSeed(defaultParams.note.rhythm, seed));
      expect(root.duration).toBe(tune.length);
      expect(root.children.length).toBe(measure[1]);
      root.children.forEach((bar, i) => {
        expect(bar.t).toBe(i * measure[0]);
        expect(bar.children.reduce((a, n) => a + n.duration, 0)).toBe(measure[0]);
      });
      // Leaves tile the whole phrase without gaps or overlaps.
      const leaves = root.leaves();
      let t = 0;
      for (const leaf of leaves) {
        expect(leaf.t).toBe(t);
        expect(leaf.duration).toBeGreaterThanOrEqual(2 ** -depth);
        t += leaf.duration;
      }
      expect(t).toBe(tune.length);
    }
  });
  test('depth 0 with draws of 0: every beat is on and pairs of beats merge', () => {
    // Per bar: 4 isNoteOn draws (no branch draw at depth 0), then merge draws for the pairs
    // (0,1) and (2,3) (a merge skips the next pair). pMerge(2) = 0.3006... > 0.
    const rng = new ConstantRng(0);
    const root = RhythmGenerator.generate(emptyTune([4, 4], Resolution.Half, 0), defaultParams.note.rhythm, contextOf(rng));
    expect(rng.consumed).toBe(4 * (4 + 2));
    expect(root.getRhythms()).toStrictEqual(
      [0, 2, 4, 6, 8, 10, 12, 14].map(t => ({t, duration: 2, isNoteOn: true}))
    );
  });
  test('depth 0 with draws close to 1: every beat is off and nothing merges', () => {
    const rng = new ConstantRng(1 - 2 ** -32);
    const root = RhythmGenerator.generate(emptyTune([4, 4], Resolution.Half, 0), defaultParams.note.rhythm, contextOf(rng));
    expect(rng.consumed).toBe(4 * (4 + 3));
    expect(root.getRhythms()).toStrictEqual(
      [...Array(16).keys()].map(t => ({t, duration: 1, isNoteOn: false}))
    );
  });
  test('depth 3 with draws of 0: branching stops where pBranch is 0', () => {
    // pBranch(1) > 0 and pBranch(0.5) > 0 but pBranch(0.25) = 0 (edge0 = 0.25), so each beat makes
    // 1 + 2 * (1 + 2 * 2) = 11 draws. The merges then drop the subtrees of the merged beats.
    const rng = new ConstantRng(0);
    const root = RhythmGenerator.generate(emptyTune([4, 4], Resolution.Half, 3), defaultParams.note.rhythm, contextOf(rng));
    expect(rng.consumed).toBe(4 * (4 * 11 + 2));
    expect(root.getRhythms().map(r => [r.t, r.duration])).toStrictEqual(
      [0, 2, 4, 6, 8, 10, 12, 14].map(t => [t, 2])
    );
  });
});

describe('pitch weight', () => {
  const p = defaultParams.note.pitchWeight;
  const tune = emptyTune([4, 4]);
  tune.scale = Scale.major(69); // A major: 69 71 73 74 76 78 80
  tune.chord = util.Timeline.fromItems([
    {t: 0, value: new Chord(tune.scale, 0, [0, 2, 4])},  // 69 73 76
    {t: 8, value: new Chord(tune.scale, 4, [0, 2, 4])},  // 76 80 83
  ]);
  test('calcTimeHomoWeight', () => {
    // [prev, candidate, expected] = absPitchFactor * relPitchFactor * factorInScale
    const cases: [number, number, number][] = [
      [69, 69, 8],     // 1 * 1 * 8
      [69, 70, 1],     // not in scale
      [69, 73, 8],     // distance 4: both factors still 1
      [69, 74, 4],     // relative distance 5: 1 - smoothstep(4, 6, 5) = 0.5
      [69, 76, 0],     // relative distance 7 > 6
      [74, 76, 4],     // absolute distance 7: 1 - smoothstep(6, 8, 7) = 0.5
      [62, 61, 0],     // absolute distance 8
      [68, 63, 0.5],   // absolute distance 6 -> 1, relative 5 -> 0.5, not in scale
      [75, 64, 0],     // relative distance 11
    ];
    for (const [prev, candidate, expected] of cases) {
      expect(PitchWeight.calcTimeHomoWeight(tune, prev, candidate, p)).toBe(expected);
    }
  });
  test('calc raises the weight with the chord factor to 1 + duration * rhythmExponentFactor', () => {
    // [prev, pitch, duration, t, expected]
    const cases: [number, number, number, number, number][] = [
      [69, 69, 0.5, 0, (8 * 4) ** 2],  // in chord at t = 0
      [69, 69, 0.5, 8, 8 ** 2],        // not in the chord at t = 8
      [69, 74, 1, 0, 4 ** 3],
      [72, 76, 0.25, 9, (0.5 * 8 * 4) ** 1.5], // absolute distance 7 halves the weight
      [69, 70, 2, 0, 1],
      [69, 76, 1, 0, 0],
    ];
    for (const [prev, pitch, duration, t, expected] of cases) {
      expect(PitchWeight.calc(tune, prev, {pitch, duration, isNoteOn: true}, t, p)).toBe(expected);
    }
  });
});

describe('note', () => {
  test.each([['default', defaultParams], ['simple', simpleParams]] as const)('%s', (_name, base) => {
    for (const seed of seeds) {
      const params = {...base, note: {...withSeed(base.note, seed), rhythm: withSeed(base.note.rhythm, seed)}};
      const log: RngRecord[] = [];
      const tune = TuneGenerator.generate(params, {rngFactory: recordingRngFactory(log)});
      const notes = tune.notes.list();
      // One pitch draw per leaf, rests included.
      expect(log.filter(r => r.label == 'note.pitch').length).toBe(notes.length);
      let prev = tune.scale.root;
      for (const {value} of notes) {
        expect(value.pitch).toBeGreaterThanOrEqual(prev - 12);
        expect(value.pitch).toBeLessThanOrEqual(prev + 11);
        prev = value.pitch;
      }
    }
  });
  test('leaves and notes match', () => {
    const tune = TuneGenerator.generate(defaultParams);
    const tree = NoteGenerator.generateTree(tune, defaultParams.note);
    expect(tree.leaves().map(l => l.toTimelineItem())).toStrictEqual(tune.notes.list());
  });
});

describe('variant', () => {
  type Layer = VariantGenerator.Layer;
  const L = VariantGenerator.Layer;
  const timespans: [number, number][] = [[6, 11], [2, 8], [9, 15], [0, 16], [3, 5]];
  const variantParams = (layer: Layer, timespan: [number, number], seed: [number, number]): VariantGenerator.Parameters => ({
    seed: {state: seed[0], sequence: seed[1]},
    resolution: Resolution.Half,
    timespan: [{value: timespan, weight: 1}],
    layer: [{value: layer, weight: 1}],
  });
  const outside = ([t0, t1]: [number, number]) => (e: {t: number}) => e.t <= t0 || t1 <= e.t;
  const base = {...defaultParams, resolution: Resolution.Half};
  const original = TuneGenerator.generate(base);

  // [layer, the timelines that must equal the original outside the timespan, the timelines that must equal it everywhere]
  const layers: [Layer, ('cadence' | 'chord' | 'notes')[], ('cadence' | 'chord')[]][] = [
    [L.Cadence, ['cadence', 'chord'], []],
    [L.Chord, ['chord'], ['cadence']],
    [L.Pitch, ['notes'], ['cadence', 'chord']],
  ];
  test.each(layers)('layer %d keeps the original outside the timespan', (layer, outsideSame, allSame) => {
    for (const ts of timespans) {
      for (const seed of seeds) {
        const tune = VariantGenerator.generate(variantParams(layer, ts, seed), base);
        for (const key of outsideSame) {
          expect(tune[key].list().filter(outside(ts))).toStrictEqual(original[key].list().filter(outside(ts)));
        }
        for (const key of allSame) {
          expect(tune[key].list()).toStrictEqual(original[key].list());
        }
      }
    }
  });
  test('layer 3 (rhythm) keeps the notes before the timespan and the upper layers', () => {
    for (const ts of timespans) {
      for (const seed of seeds) {
        const tune = VariantGenerator.generate(variantParams(L.Rhythm, ts, seed), base);
        const before = (e: {t: number}) => e.t <= ts[0];
        expect(tune.notes.list().filter(before)).toStrictEqual(original.notes.list().filter(before));
        expect(tune.cadence.list()).toStrictEqual(original.cadence.list());
        expect(tune.chord.list()).toStrictEqual(original.chord.list());
      }
    }
  });
  test('the Variant RNG picks the timespan first and then the layer', () => {
    const params: VariantGenerator.Parameters = {
      seed: {state: 0, sequence: 0}, resolution: Resolution.Half,
      timespan: [{value: [2, 6], weight: 1}, {value: [6, 11], weight: 1}],
      layer: [{value: L.Cadence, weight: 1}, {value: L.Chord, weight: 1}],
    };
    const variantRng = new ScriptedRng([0.75, 0.25, ...Array(16).fill(0.5)]);
    const tune = VariantGenerator.generate(params, base, {
      rngFactory: (seed, label) => label == 'variant' ? variantRng : recordingRngFactory([])(seed, label),
    });
    // timespan [6, 11] and layer Cadence; the Cadence variant then draws for the slots strictly inside.
    expect(tune.cadence.list().filter(outside([6, 11]))).toStrictEqual(original.cadence.list().filter(outside([6, 11])));
    expect(variantRng.consumed).toBeGreaterThan(2);
  });
  // Known bug: RhythmVariantGenerator.splitRhythm reads the neighbours of the split leaf,
  // which do not exist for the last leaf.
  test('splitting the last leaf throws (known bug)', () => {
    const rng = new ConstantRng(0); // every split happens
    const params = variantParams(L.Rhythm, [8, 16], [0, 0]);
    expect(() => VariantGenerator.generate(params, base, {
      rngFactory: (seed, label) => label == 'variant' ? rng : recordingRngFactory([])(seed, label),
    })).toThrow(/pitch/);
  });
});
