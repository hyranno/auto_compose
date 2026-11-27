import { createStore, reconcile, unwrap } from 'solid-js/store';
import {
  defaultVariantParameters,
  generateTune,
  generateVariant,
  type Layer,
  type PerCadence,
  type Resolution,
  type TuneParameters,
  type VariantParameters,
} from 'auto-composer-wasm';

import minorTuneParameters from '../../resources/TuneGenParams_minor.json';

/**
 * All the parameters of the page, as the plain JSON the wasm functions take.
 * The defaults come from `resources/`: the tune parameters of the page are the minor example,
 * and the variant parameters are the defaults of the wasm package.
 */
export type PageParameters = { tune: TuneParameters; variant: VariantParameters };

export const defaultTune = (): TuneParameters => structuredClone(minorTuneParameters) as TuneParameters;
export const defaultVariant = (): VariantParameters => defaultVariantParameters();

/** The tempo the page starts with. */
export const DEFAULT_BPM = 120;

export function createParameters() {
  const [params, setParams] = createStore<PageParameters>({
    tune: defaultTune(),
    variant: defaultVariant(),
  });

  /** A plain copy (not a store proxy) to pass to wasm or to `JSON.stringify`. */
  const snapshot = (): PageParameters => structuredClone(unwrap(params));

  /**
   * Replaces the tune parameters with `json` if they are valid, and throws otherwise.
   * They are checked by generating a tune from them.
   */
  const loadTune = (json: unknown) => {
    generateTune(json as TuneParameters);
    setParams('tune', reconcile(json as TuneParameters));
  };

  /**
   * Replaces the variant parameters with `json` if they are valid with the current tune parameters,
   * and throws otherwise. They are checked by generating a variant from them.
   */
  const loadVariant = (json: unknown) => {
    generateVariant(json as VariantParameters, snapshot().tune);
    setParams('variant', reconcile(json as VariantParameters));
  };

  return { params, setParams, snapshot, loadTune, loadVariant };
}

export type Parameters = ReturnType<typeof createParameters>;

// The keys list every resolution; the compiler reports a missing or unknown one.
export const RESOLUTION_LABELS: Record<Resolution, string> = {
  perfect_authentic: 'Perfect authentic',
  imperfect_authentic: 'Imperfect authentic',
  deceptive: 'Deceptive',
  plagal: 'Plagal',
  half: 'Half',
};
export const RESOLUTIONS = Object.keys(RESOLUTION_LABELS) as Resolution[];

export const MAJOR_TONES = [0, 2, 4, 5, 7, 9, 11];
export const MINOR_TONES = [0, 2, 3, 5, 7, 8, 10];

export type Cadence = keyof PerCadence<unknown>;
export const CADENCES: readonly Cadence[] = ['tonic', 'dominant', 'subdominant'];

export const LAYERS: readonly Layer[] = ['cadence', 'chord', 'rhythm', 'pitch'];
