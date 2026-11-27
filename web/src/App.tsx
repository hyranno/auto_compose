import { createSignal, Show } from 'solid-js';
import { compose, type TuneSnapshot } from 'auto-composer-wasm';

import { Field, NumberInput } from './fields';
import { PianoRoll } from './PianoRoll';
import { createParameters } from './params';
import { TuneSettings, VariantSettings } from './Settings';
import { TuneOutput } from './Output';
import './App.css';

/** The rows of the piano roll around the key of the scale. */
const ROLL_RANGE = 14;

export default function App() {
  const parameters = createParameters();
  const [tune, setTune] = createSignal<TuneSnapshot>();
  const [error, setError] = createSignal<string>();
  const [bpm, setBpm] = createSignal(100);

  let form!: HTMLFormElement;

  // On an error, the previous tune stays on the page with the message.
  const generate = () => {
    // An input whose text does not parse keeps the last valid value in the store;
    // generating from it would ignore what is shown, so the form is refused instead.
    const invalid = form?.querySelector<HTMLInputElement>(':invalid');
    if (invalid) {
      for (let d = invalid.closest('details'); d; d = d.parentElement?.closest('details') ?? null) d.open = true;
      form.reportValidity();
      setError('Some fields are invalid.');
      return;
    }
    try {
      const { tune, variant } = parameters.snapshot();
      setTune(compose(tune, variant));
      setError(undefined);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };
  generate();

  return (
    <main>
      <form
        ref={form}
        noValidate
        onSubmit={(e) => {
          e.preventDefault();
          generate();
        }}
      >
        <TuneSettings parameters={parameters} />
        <VariantSettings parameters={parameters} />
        <button type="submit" class="generate">
          generate
        </button>
      </form>
      <Show when={error()}>{(message) => <p role="alert">{message()}</p>}</Show>
      <Show when={tune()}>
        {(t) => (
          <>
            <PianoRoll
              tune={t()}
              noteBottom={t().scale.key - ROLL_RANGE}
              noteTop={t().scale.key + ROLL_RANGE - 1}
            />
            <Field label="BPM">
              <NumberInput min={1} value={bpm()} onChange={setBpm} />
            </Field>
            <TuneOutput tune={t()} bpm={bpm()} />
          </>
        )}
      </Show>
    </main>
  );
}
