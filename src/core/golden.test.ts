// Checks the current implementation against the golden fixtures (regenerate with `npm run gen_golden`).
import { describe, test, expect } from 'vitest';
import {readdirSync, readFileSync} from 'node:fs';
import path from 'node:path';

import * as TuneGenerator from './tune_generator';
import * as VariantGenerator from './variant';
import {snapshot} from './snapshot';

const goldenDir = path.resolve(__dirname, '../../fixtures/golden');

function loadCases(sub: string): [string, any][] {
  const dir = path.join(goldenDir, sub);
  return readdirSync(dir).filter(f => f.endsWith('.json')).sort()
    .map(f => [f, JSON.parse(readFileSync(path.join(dir, f), 'utf8'))]);
}

describe('golden tune', () => {
  const cases = loadCases('tune');
  test('fixtures exist', () => expect(cases.length).toBeGreaterThan(0));
  test.each(cases)('%s', (_name, c) => {
    expect(snapshot(TuneGenerator.generate(c.params))).toStrictEqual(c.expected);
  });
});

describe('golden variant', () => {
  const cases = loadCases('variant');
  test('fixtures exist', () => expect(cases.length).toBeGreaterThan(0));
  test.each(cases)('%s', (_name, c) => {
    expect(snapshot(VariantGenerator.generate(c.params, c.base))).toStrictEqual(c.expected);
  });
});
