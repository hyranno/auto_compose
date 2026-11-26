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

  // On an error, the previous tune stays on the page with the message.
  const generate = () => {
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
      <TuneSettings parameters={parameters} />
      <VariantSettings parameters={parameters} />
      <button type="button" onClick={generate}>
        generate
      </button>
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
              <NumberInput value={bpm()} onChange={setBpm} />
            </Field>
            <TuneOutput tune={t()} bpm={bpm()} />
          </>
        )}
      </Show>
    </main>
  );
}
