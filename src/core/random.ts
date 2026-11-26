// Random number source abstraction.
// Generators depend on `Rng` instead of reliable-random's `Random`, so that tests can
// feed scripted values and record the consumption order (to compare with the Rust port).
import {Random} from 'reliable-random';

import type {RandomSeed} from './util';

/** A source of uniform random numbers in [0, 1). */
export interface Rng {
  random(): number;
}

/**
 * Creates an Rng from a seed. `label` names the consumer (e.g. "cadence", "chord.root")
 * and is used only by diagnostic factories such as `recordingRngFactory`.
 */
export type RngFactory = (seed: RandomSeed, label: string) => Rng;

export const defaultRngFactory: RngFactory = (s) => new Random(s.state, s.sequence);

/** Options shared by the generate functions. */
export type GenerateContext = {
  rngFactory: RngFactory;
};
export const defaultContext: GenerateContext = {rngFactory: defaultRngFactory};

/** Returns the given values in order. Throws when exhausted. */
export class ScriptedRng implements Rng {
  private index = 0;
  constructor(private readonly values: number[]) {}
  random(): number {
    if (this.index >= this.values.length) {
      throw new Error(`ScriptedRng exhausted after ${this.values.length} values`);
    }
    return this.values[this.index++];
  }
  get consumed(): number { return this.index; }
}

export type RngRecord = {label: string, value: number};

/** Delegates to `inner` and appends each value with `label` to `log`. */
export class RecordingRng implements Rng {
  constructor(
    private readonly inner: Rng,
    private readonly label: string,
    private readonly log: RngRecord[],
  ) {}
  random(): number {
    const value = this.inner.random();
    this.log.push({label: this.label, value});
    return value;
  }
}

/** Wraps every Rng created by `inner` with a RecordingRng writing to `log`. */
export function recordingRngFactory(log: RngRecord[], inner: RngFactory = defaultRngFactory): RngFactory {
  return (seed, label) => new RecordingRng(inner(seed, label), label, log);
}
