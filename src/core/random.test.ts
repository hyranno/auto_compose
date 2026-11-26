import { describe, test, expect } from 'vitest';
import {readFileSync} from 'node:fs';
import path from 'node:path';

import {Random} from 'reliable-random';
import {ScriptedRng, RecordingRng, recordingRngFactory, defaultRngFactory} from './random';
import type {RngRecord} from './random';
import * as TuneGenerator from './tune_generator';
import * as VariantGenerator from './variant';
import {snapshot} from './snapshot';

const resources = path.resolve(__dirname, '../../resources');
const loadResource = (name: string) => JSON.parse(readFileSync(path.join(resources, name), 'utf8'));

describe('ScriptedRng', () => {
  test('returns the values in order and then throws', () => {
    const r = new ScriptedRng([0, 0.5, 0.999]);
    expect([r.random(), r.random(), r.random()]).toStrictEqual([0, 0.5, 0.999]);
    expect(r.consumed).toBe(3);
    expect(() => r.random()).toThrow();
  });
});

describe('RecordingRng', () => {
  test('passes values through and records them with the label', () => {
    const log: RngRecord[] = [];
    const r = new RecordingRng(new ScriptedRng([0.25, 0.75]), 'x', log);
    expect([r.random(), r.random()]).toStrictEqual([0.25, 0.75]);
    expect(log).toStrictEqual([{label: 'x', value: 0.25}, {label: 'x', value: 0.75}]);
  });
});

describe('defaultRngFactory', () => {
  test('matches reliable-random', () => {
    const seed = {state: 4649, sequence: 459};
    const a = defaultRngFactory(seed, 'any');
    const b = new Random(seed.state, seed.sequence);
    for (let i = 0; i < 8; i++) expect(a.random()).toBe(b.random());
  });
});

describe('recordingRngFactory', () => {
  test('does not change the tune output', () => {
    const params = loadResource('TuneGenParams_default.json');
    const log: RngRecord[] = [];
    const tune = snapshot(TuneGenerator.generate(params, {rngFactory: recordingRngFactory(log)}));
    expect(tune).toStrictEqual(snapshot(TuneGenerator.generate(params)));
    // Generators run in this order, each consuming its own Rng.
    const labels = [...new Set(log.map(r => r.label))];
    expect(labels).toStrictEqual(['cadence', 'chord.root', 'chord.tone', 'rhythm', 'note.pitch']);
    // One root and one tone draw per chord.
    const count = (l: string) => log.filter(r => r.label == l).length;
    expect(count('chord.root')).toBe(tune.chord.length);
    expect(count('chord.tone')).toBe(tune.chord.length);
  });
  test('does not change the variant output', () => {
    const base = loadResource('TuneGenParams_default.json');
    const params = {
      ...loadResource('VariantGenParams_default.json'),
      timespan: [{value: [2, 8], weight: 1}],
      layer: [{value: 3, weight: 1}],
    };
    const log: RngRecord[] = [];
    const tune = VariantGenerator.generate(params, base, {rngFactory: recordingRngFactory(log)});
    expect(snapshot(tune)).toStrictEqual(snapshot(VariantGenerator.generate(params, base)));
    expect(log[0].label).toBe('variant');
    expect(log[1].label).toBe('variant');
  });
});

// fixtures/rng.json pins the raw output of reliable-random; the Rust port reproduces it.
describe('reliable-random (fixtures/rng.json)', () => {
  const fixture = JSON.parse(readFileSync(path.resolve(__dirname, '../../fixtures/rng.json'), 'utf8'));
  const cases: [number, number, number[]][] = fixture.cases.map((c: any) => [c.state, c.sequence, c.u32]);

  // Reference implementation of the algorithm the Rust port follows.
  // It differs from the standard PCG32 output function: xorshifted is not masked to 32 bits,
  // and the left rotation is `rot ^ 31` (= 31 - rot) instead of `(-rot) & 31`.
  const U32 = (1n << 32n) - 1n;
  const U64 = (1n << 64n) - 1n;
  function reference(state: number, sequence: number, count: number): number[] {
    const inc = ((BigInt(sequence) << 1n) | 1n) & U64;
    let s = 0n;
    const next = () => {
      const old = s;
      s = (old * 6364136223846793005n + inc) & U64;
      const xorshifted = ((old >> 18n) ^ old) >> 27n;
      const rot = old >> 59n;
      return Number(((xorshifted >> rot) | (xorshifted << (rot ^ 31n))) & U32);
    };
    next();
    s = (s + BigInt(state)) & U64;
    next();
    return [...Array(count)].map(next);
  }

  test('fixtures exist', () => expect(cases.length).toBeGreaterThan(0));
  test.each(cases)('state=%d sequence=%d', (state, sequence, u32) => {
    const r = new Random(state, sequence);
    expect(u32.map(() => Number(r._random_b()))).toStrictEqual(u32);
    expect(reference(state, sequence, u32.length)).toStrictEqual(u32);
    const f = new Random(state, sequence);
    expect(u32.map(() => f.random())).toStrictEqual(u32.map(v => v / 2 ** 32));
  });
  test('non-integer seeds throw', () => {
    expect(() => new Random(0.5, 0)).toThrow();
  });
});
