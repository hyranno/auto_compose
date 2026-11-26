import { createStore, reconcile, unwrap } from 'solid-js/store';
import {
  defaultTuneParameters,
  defaultVariantParameters,
  generateTune,
  generateVariant,
  type Resolution,
  type TuneParameters,
  type VariantParameters,
} from 'auto-composer-wasm';

/**
 * All the parameters of the page, as the plain JSON the wasm functions take.
 * The defaults come from the wasm package, whose single source is `resources/*.json`.
 */
export type PageParameters = { tune: TuneParameters; variant: VariantParameters };

export function createParameters() {
  const [params, setParams] = createStore<PageParameters>({
    tune: defaultTuneParameters(),
    variant: defaultVariantParameters(),
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
const resolutions: Record<Resolution, true> = {
  'Perfect authentic': true,
  'Imperfect authentic': true,
  Deceptive: true,
  Plagal: true,
  Half: true,
};
export const RESOLUTIONS = Object.keys(resolutions) as Resolution[];

export const MAJOR_TONES = [0, 2, 4, 5, 7, 9, 11];
export const MINOR_TONES = [0, 2, 3, 5, 7, 8, 10];
