import { describe, test, expect } from 'vitest';
import {readFileSync} from 'node:fs';
import path from 'node:path';

import {Random} from 'reliable-random';
import {ScriptedRng, RecordingRng, recordingRngFactory, defaultRngFactory} from './random';
import type {RngRecord} from './random';
import * as TuneGenerator from './tune_generator';
import * as VariantGenerator from './variant';
import {snapshot} from './snapshot';

const goldenDir = path.resolve(__dirname, '../../fixtures/golden');
const loadFixture = (rel: string) => JSON.parse(readFileSync(path.join(goldenDir, rel), 'utf8'));

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
    const c = loadFixture('tune/default_seed-per-layer.json');
    const log: RngRecord[] = [];
    const tune = TuneGenerator.generate(c.params, {rngFactory: recordingRngFactory(log)});
    expect(snapshot(tune)).toStrictEqual(c.expected);
    // Generators run in this order, each consuming its own Rng.
    const labels = [...new Set(log.map(r => r.label))];
    expect(labels).toStrictEqual(['cadence', 'chord.root', 'chord.tone', 'rhythm', 'note.pitch']);
    // One root and one tone draw per chord.
    const count = (l: string) => log.filter(r => r.label == l).length;
    expect(count('chord.root')).toBe(c.expected.chord.length);
    expect(count('chord.tone')).toBe(c.expected.chord.length);
  });
  test('does not change the variant output', () => {
    const c = loadFixture('variant/default_layer-3_span-2-8.json');
    const log: RngRecord[] = [];
    const tune = VariantGenerator.generate(c.params, c.base, {rngFactory: recordingRngFactory(log)});
    expect(snapshot(tune)).toStrictEqual(c.expected);
    expect(log[0].label).toBe('variant');
    expect(log[1].label).toBe('variant');
  });
});
