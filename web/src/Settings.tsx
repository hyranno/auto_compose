import { createSignal, Show } from 'solid-js';

import { Field, Group, LayerSeedInput, NumberInput, SeedInput, Select, SmoothstepInput } from './fields';
import { download, readChosenFile } from './file';
import { MAJOR_TONES, MINOR_TONES, RESOLUTION_LABELS, RESOLUTIONS, type Parameters } from './params';

/** Buttons to save a part of the parameters as JSON and to load it back. */
function JsonFile(props: { filename: string; save: () => unknown; load: (json: unknown) => void }) {
  const [error, setError] = createSignal<string>();
  const load = async (input: HTMLInputElement) => {
    try {
      const text = await readChosenFile(input);
      if (text === undefined) return;
      props.load(JSON.parse(text));
      setError(undefined);
    } catch (e) {
      setError(`Not loaded: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      // Lets the same file be chosen again after it is edited.
      input.value = '';
    }
  };
  return (
    <div class="json-file">
      <button
        type="button"
        onClick={() => download(JSON.stringify(props.save(), null, 2), props.filename, 'application/json')}
      >
        save as file
      </button>
      <label>
        load from file
        <input type="file" accept=".json,application/json" onChange={(e) => load(e.currentTarget)} />
      </label>
      <Show when={error()}>{(message) => <p role="alert">{message()}</p>}</Show>
    </div>
  );
}

export function TuneSettings(props: { parameters: Parameters }) {
  const tune = () => props.parameters.params.tune;
  const set = props.parameters.setParams;
  return (
    <Group title="Settings">
      <JsonFile
        filename="TuneGenParams.json"
        save={() => props.parameters.snapshot().tune}
        load={props.parameters.loadTune}
      />
      <Field label="seed">
        <SeedInput value={tune().seed} onChange={(v) => set('tune', 'seed', v)} />
      </Field>
      <Group title="scale">
        <Field label="key">
          <NumberInput step={1} value={tune().scale.key} onChange={(v) => set('tune', 'scale', 'key', v)} />
        </Field>
        <Field label="tones">
          [{tune().scale.tones.join(',')}]
          <button type="button" onClick={() => set('tune', 'scale', 'tones', [...MAJOR_TONES])}>
            Major
          </button>
          <button type="button" onClick={() => set('tune', 'scale', 'tones', [...MINOR_TONES])}>
            minor
          </button>
        </Field>
      </Group>
      <Field label="resolution">
        <Select
          options={RESOLUTIONS}
          labels={RESOLUTION_LABELS}
          value={tune().resolution}
          onChange={(v) => set('tune', 'resolution', v)}
        />
      </Field>
      <Field label="beats per bar, bars">
        <NumberInput
          step={1}
          value={tune().time_measure.beats_per_bar}
          onChange={(v) => set('tune', 'time_measure', 'beats_per_bar', v)}
        />
        <NumberInput
          step={1}
          value={tune().time_measure.bars}
          onChange={(v) => set('tune', 'time_measure', 'bars', v)}
        />
      </Field>
      <Group title="Cadence Generator">
        <Field label="seed">
          <LayerSeedInput value={tune().cadence.seed} onChange={(v) => set('tune', 'cadence', 'seed', v)} />
        </Field>
        <Field label="duration">
          <NumberInput value={tune().cadence.duration} onChange={(v) => set('tune', 'cadence', 'duration', v)} />
        </Field>
      </Group>
      <Group title="Chord Generator">
        <Field label="seed">
          <LayerSeedInput value={tune().chord.seed} onChange={(v) => set('tune', 'chord', 'seed', v)} />
        </Field>
      </Group>
      <Group title="Rhythm Generator">
        <Field label="seed">
          <LayerSeedInput value={tune().rhythm.seed} onChange={(v) => set('tune', 'rhythm', 'seed', v)} />
        </Field>
        <Field label="max beat division depth">
          <NumberInput
            step={1}
            value={tune().rhythm.max_beat_division_depth}
            onChange={(v) => set('tune', 'rhythm', 'max_beat_division_depth', v)}
          />
        </Field>
        <Field label="p_branch edges">
          <SmoothstepInput value={tune().rhythm.p_branch} onChange={(v) => set('tune', 'rhythm', 'p_branch', v)} />
        </Field>
        <Field label="p_note_on edges">
          <SmoothstepInput value={tune().rhythm.p_note_on} onChange={(v) => set('tune', 'rhythm', 'p_note_on', v)} />
        </Field>
      </Group>
      <Group title="Pitch Generator">
        <Field label="seed">
          <LayerSeedInput value={tune().pitch.seed} onChange={(v) => set('tune', 'pitch', 'seed', v)} />
        </Field>
        <Field label="abs_pitch_factor edges">
          <SmoothstepInput
            value={tune().pitch.abs_pitch_factor}
            onChange={(v) => set('tune', 'pitch', 'abs_pitch_factor', v)}
          />
        </Field>
        <Field label="rel_pitch_factor edges">
          <SmoothstepInput
            value={tune().pitch.rel_pitch_factor}
            onChange={(v) => set('tune', 'pitch', 'rel_pitch_factor', v)}
          />
        </Field>
        <Field label="factor_in_scale">
          <NumberInput
            value={tune().pitch.factor_in_scale}
            onChange={(v) => set('tune', 'pitch', 'factor_in_scale', v)}
          />
        </Field>
        <Field label="factor_in_chord">
          <NumberInput
            value={tune().pitch.factor_in_chord}
            onChange={(v) => set('tune', 'pitch', 'factor_in_chord', v)}
          />
        </Field>
        <Field label="rhythm_exponent_factor">
          <NumberInput
            value={tune().pitch.rhythm_exponent_factor}
            onChange={(v) => set('tune', 'pitch', 'rhythm_exponent_factor', v)}
          />
        </Field>
      </Group>
    </Group>
  );
}

export function VariantSettings(props: { parameters: Parameters }) {
  const variant = () => props.parameters.params.variant;
  const set = props.parameters.setParams;
  return (
    <Group title="Settings (Variant)">
      <JsonFile
        filename="VariantGenParams.json"
        save={() => props.parameters.snapshot().variant}
        load={props.parameters.loadVariant}
      />
      <Field label="seed">
        <SeedInput value={variant().seed} onChange={(v) => set('variant', 'seed', v)} />
      </Field>
      <Field label="resolution">
        <Select
          options={RESOLUTIONS}
          labels={RESOLUTION_LABELS}
          value={variant().resolution}
          onChange={(v) => set('variant', 'resolution', v)}
        />
      </Field>
    </Group>
  );
}
