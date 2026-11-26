import { createSignal, Show } from 'solid-js';
import {
  compose,
  defaultTuneParameters,
  defaultVariantParameters,
  type TuneSnapshot,
} from 'auto-composer-wasm';

// A placeholder until the UI is ported (2C-3): composes a tune from the default parameters.
export default function App() {
  const [tune, setTune] = createSignal<TuneSnapshot>();
  const [error, setError] = createSignal<string>();

  const generate = () => {
    try {
      setTune(compose(defaultTuneParameters(), defaultVariantParameters()));
      setError(undefined);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <main>
      <h1>Auto Composer</h1>
      <button onClick={generate}>generate</button>
      <Show when={error()}>{(message) => <p role="alert">{message()}</p>}</Show>
      <Show when={tune()}>
        {(t) => (
          <p>
            {t().notes.length} notes, {t().chord.length} chords, length {t().length}
          </p>
        )}
      </Show>
    </main>
  );
}
