// Generates the fixtures under fixtures/ from the current TS implementation:
// fixtures/rng.json (raw output of reliable-random) and fixtures/golden/ (generator input/output pairs).
// Usage: npm run gen_golden
// The fixtures are committed and checked by src/core/random.test.ts and src/core/golden.test.ts
// (and later by the Rust tests).
import {Random} from 'reliable-random';
import {mkdirSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import path from 'node:path';

import * as TuneGenerator from '../src/core/tune_generator';
import * as VariantGenerator from '../src/core/variant';
import {snapshot} from '../src/core/snapshot';
import type {RandomSeed} from '../src/core/util';
import {Resolution} from '../src/core/tune';

import defaultJson from '../resources/TuneGenParams_default.json';
import minorJson from '../resources/TuneGenParams_minor.json';
import simpleJson from '../resources/TuneGenParams_simple.json';
import variantJson from '../resources/VariantGenParams_default.json';

type TuneParams = TuneGenerator.Parameters;
type VariantParams = VariantGenerator.Parameters;

const presets: [string, TuneParams][] = [
  ['default', defaultJson as TuneParams],
  ['minor', minorJson as TuneParams],
  ['simple', simpleJson as TuneParams],
];
const baseVariant = variantJson as VariantParams;

type LayerSeeds = {cadence: RandomSeed, root: RandomSeed, tone: RandomSeed, rhythm: RandomSeed, note: RandomSeed};
function sameSeeds(state: number, sequence: number): LayerSeeds {
  const s = {state, sequence};
  return {cadence: s, root: s, tone: s, rhythm: s, note: s};
}
function withSeeds(params: TuneParams, seeds: LayerSeeds): TuneParams {
  const p = structuredClone(params);
  p.cadence.seed = {...seeds.cadence};
  p.chord.root.seed = {...seeds.root};
  p.chord.tone.rand_tones.seed = {...seeds.tone};
  p.note.rhythm.seed = {...seeds.rhythm};
  p.note.seed = {...seeds.note};
  return p;
}
function modified(params: TuneParams, f: (p: TuneParams) => void): TuneParams {
  const p = structuredClone(params);
  f(p);
  return p;
}

// --- tune cases ---
const seedSets: [string, LayerSeeds][] = [
  ['seed-preset', sameSeeds(4649, 459)],
  ['seed-1-0', sameSeeds(1, 0)],
  ['seed-negative', sameSeeds(-1, -7)],
  ['seed-large', sameSeeds(Number.MAX_SAFE_INTEGER, 2 ** 40 + 3)],
  ['seed-per-layer', {
    cadence: {state: 1, sequence: 1}, root: {state: 2, sequence: 3}, tone: {state: 5, sequence: 8},
    rhythm: {state: 13, sequence: 21}, note: {state: 34, sequence: 55},
  }],
];

const tuneCases: [string, TuneParams][] = [];
for (const [preset, params] of presets) {
  for (const [seedName, seeds] of seedSets) {
    tuneCases.push([`${preset}_${seedName}`, withSeeds(params, seeds)]);
  }
}
const resolutionNames: [string, Resolution][] = [
  ['perfect', Resolution.Perfect], ['imperfect', Resolution.Imperfect], ['deceptive', Resolution.Deceptive],
  ['plagal', Resolution.Plagal], ['half', Resolution.Half],
];
const defaultParams = presets[0][1];
for (const [resName, resolution] of resolutionNames) {
  for (const tm of [[4, 4], [3, 4]] as [number, number][]) {
    tuneCases.push([`default_${resName}_${tm[0]}-${tm[1]}`, modified(defaultParams, p => {
      p.resolution = resolution;
      p.time_measure = tm;
    })]);
  }
}
for (const [preset, params] of [presets[0], presets[2]]) {
  for (const depth of [0, 1]) {
    tuneCases.push([`${preset}_depth-${depth}`, modified(params, p => {
      p.note.rhythm.max_beat_division_depth = depth;
    })]);
  }
}

// --- variant cases ---
type VariantCase = [string, VariantParams, TuneParams];
const variantCases: VariantCase[] = [];
const timespans: [number, number][] = [[6, 11], [2, 8], [9, 15], [0, 16]];
for (const [preset, base] of [presets[0], presets[2]]) {
  for (const layer of [0, 1, 2, 3] as VariantGenerator.Layer[]) {
    for (const ts of timespans) {
      variantCases.push([`${preset}_layer-${layer}_span-${ts[0]}-${ts[1]}`, {
        ...structuredClone(baseVariant),
        timespan: [{value: ts, weight: 1}],
        layer: [{value: layer, weight: 1}],
      }, base]);
    }
  }
}
// Let the Variant RNG pick the timespan and the layer.
// The seeds are chosen so that every layer gets picked at least once.
for (const [state, sequence] of [[1, 2], [77, 5], [5, 1], [13, 1]]) {
  variantCases.push([`default_random-pick_seed-${state}-${sequence}`, {
    seed: {state, sequence},
    resolution: Resolution.Half,
    timespan: [{value: [2, 6], weight: 1}, {value: [6, 11], weight: 2}, {value: [8, 14], weight: 1}],
    layer: [0, 1, 2, 3].map(l => ({value: l as VariantGenerator.Layer, weight: 1})),
  }, presets[0][1]]);
}

// --- rng cases ---
// Seeds cover zero, small, negative (two's complement), and values beyond 32 bits.
const rngSeeds: [number, number][] = [
  [0, 0], [1, 0], [0, 1], [754, 489], [4649, 459], [-1, -7], [-4649, 459],
  [Number.MAX_SAFE_INTEGER, 2 ** 40 + 3], [-Number.MAX_SAFE_INTEGER, -(2 ** 52)],
];
const rngCount = 16;

// --- write ---
const root = path.resolve('fixtures/golden');
function resetDir(dir: string) {
  mkdirSync(dir, {recursive: true});
  for (const f of readdirSync(dir)) {
    if (f.endsWith('.json')) rmSync(path.join(dir, f));
  }
}
// Like JSON.stringify(data, null, 2), but objects and arrays that contain no nested
// objects (e.g. one timeline item) are kept on a single line to keep the diffs readable.
function format(data: unknown, indent = ''): string {
  const isFlat = (v: unknown): boolean => v === null || typeof v !== 'object'
    || (Array.isArray(v) && v.every(x => x === null || typeof x !== 'object'));
  if (data === null || typeof data !== 'object') return JSON.stringify(data);
  const entries: [string | null, unknown][] = Array.isArray(data)
    ? data.map(v => [null, v]) : Object.entries(data);
  const [open, close] = Array.isArray(data) ? ['[', ']'] : ['{', '}'];
  if (entries.length == 0) return open + close;
  const item = ([k, v]: [string | null, unknown], ind: string) =>
    (k === null ? '' : JSON.stringify(k) + ': ') + format(v, ind);
  if (entries.every(([, v]) => isFlat(v))) {
    return open + entries.map(e => item(e, '')).join(', ') + close;
  }
  const inner = indent + '  ';
  return open + '\n' + entries.map(e => inner + item(e, inner)).join(',\n') + '\n' + indent + close;
}
function write(file: string, data: unknown) {
  writeFileSync(file, format(data) + '\n');
}

write(path.resolve('fixtures/rng.json'), {
  description: 'First outputs of reliable-random Random(state, sequence)._random_b() (u32). random() is u32 / 2^32.',
  cases: rngSeeds.map(([state, sequence]) => {
    const r = new Random(state, sequence);
    return {state, sequence, u32: [...Array(rngCount)].map(() => Number(r._random_b()))};
  }),
});

resetDir(path.join(root, 'tune'));
for (const [name, params] of tuneCases) {
  write(path.join(root, 'tune', `${name}.json`), {params, expected: snapshot(TuneGenerator.generate(params))});
}

resetDir(path.join(root, 'variant'));
const excluded: {name: string, params: VariantParams, base: string, error: string}[] = [];
for (const [name, params, base] of variantCases) {
  let expected;
  try {
    expected = snapshot(VariantGenerator.generate(params, base));
  } catch (e) {
    excluded.push({name, params, base: name.split('_')[0], error: String(e)});
    continue;
  }
  write(path.join(root, 'variant', `${name}.json`), {params, base, expected});
}
// Cases that currently throw (see the known bugs) are not part of the golden set; list them for the record.
const excludedFile = path.join(root, 'variant_excluded.json');
rmSync(excludedFile, {force: true});
if (excluded.length > 0) write(excludedFile, excluded);

console.log(`tune: ${tuneCases.length}, variant: ${variantCases.length - excluded.length}, excluded: ${excluded.length}`);
for (const e of excluded) console.log(`  excluded ${e.name}: ${e.error}`);
