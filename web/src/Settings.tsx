import { createSignal, For, Show } from 'solid-js';
import type { Layer } from 'auto-composer-wasm';

import {
  Field,
  Group,
  IntegerListInput,
  LayerSeedInput,
  NumberInput,
  SeedInput,
  Select,
  SmoothstepInput,
  WeightedList,
} from './fields';
import { download, readChosenFile } from './file';
import { HELP } from './help';
import {
  CADENCES,
  defaultTune,
  defaultVariant,
  LAYERS,
  MAJOR_TONES,
  MINOR_TONES,
  RESOLUTION_LABELS,
  RESOLUTIONS,
  type Parameters,
} from './params';

/** Buttons to save a part of the parameters as JSON, to load it back, and to load its defaults. */
function JsonFile(props: {
  filename: string;
  save: () => unknown;
  load: (json: unknown) => void;
  loadDefault: () => unknown;
}) {
  const [error, setError] = createSignal<string>();
  let input!: HTMLInputElement;
  /** Loads the JSON that `read` gives, or shows why it is not loaded. */
  const load = async (read: () => unknown) => {
    try {
      const json = await read();
      if (json === undefined) return;
      props.load(json);
      setError(undefined);
    } catch (e) {
      setError(`Not loaded: ${e instanceof Error ? e.message : String(e)}`);
    }
  };
  const loadChosenFile = () =>
    load(async () => {
      try {
        const text = await readChosenFile(input);
        return text === undefined ? undefined : JSON.parse(text);
      } finally {
        // Lets the same file be chosen again after it is edited.
        input.value = '';
      }
    });
  return (
    <div class="json-file">
      <button
        type="button"
        onClick={() => download(JSON.stringify(props.save(), null, 2), props.filename, 'application/json')}
      >
        save JSON
      </button>
      <button type="button" onClick={() => input.click()}>
        load JSON
      </button>
      <input ref={input} type="file" accept=".json,application/json" hidden onChange={loadChosenFile} />
      <button type="button" onClick={() => load(props.loadDefault)}>
        load default
      </button>
      <Show when={error()}>{(message) => <p role="alert">{message()}</p>}</Show>
    </div>
  );
}

export function TuneSettings(props: { parameters: Parameters }) {
  const tune = () => props.parameters.params.tune;
  const set = props.parameters.setParams;
  return (
    <Group title="Tune" open>
      <JsonFile
        filename="TuneGenParams.json"
        save={() => props.parameters.snapshot().tune}
        load={props.parameters.loadTune}
        loadDefault={defaultTune}
      />
      <Field label="seed" help={HELP.tune.seed}>
        <SeedInput value={tune().seed} onChange={(v) => set('tune', 'seed', v)} />
      </Field>
      <Field label="key (note number)" help={HELP.tune.key}>
        <NumberInput integer value={tune().scale.key} onChange={(v) => set('tune', 'scale', 'key', v)} />
      </Field>
      <Field label="scale tones" help={HELP.tune.tones}>
        <IntegerListInput value={tune().scale.tones} onChange={(v) => set('tune', 'scale', 'tones', v)} />
        <button type="button" onClick={() => set('tune', 'scale', 'tones', [...MAJOR_TONES])}>
          Major
        </button>
        <button type="button" onClick={() => set('tune', 'scale', 'tones', [...MINOR_TONES])}>
          minor
        </button>
      </Field>
      <Field label="beats per bar" help={HELP.tune.beatsPerBar}>
        <NumberInput
          integer
          min={1}
          value={tune().time_measure.beats_per_bar}
          onChange={(v) => set('tune', 'time_measure', 'beats_per_bar', v)}
        />
      </Field>
      <Field label="bars" help={HELP.tune.bars}>
        <NumberInput
          integer
          min={1}
          value={tune().time_measure.bars}
          onChange={(v) => set('tune', 'time_measure', 'bars', v)}
        />
      </Field>
      <Field label="resolution" help={HELP.tune.resolution}>
        <Select
          options={RESOLUTIONS}
          labels={RESOLUTION_LABELS}
          value={tune().resolution}
          onChange={(v) => set('tune', 'resolution', v)}
        />
      </Field>

      <Group title="Cadence">
        <Field label="seed" help={HELP.cadence.seed}>
          <LayerSeedInput value={tune().cadence.seed} onChange={(v) => set('tune', 'cadence', 'seed', v)} />
        </Field>
        <Field label="duration (beats)" help={HELP.cadence.duration}>
          <NumberInput
            min={0}
            value={tune().cadence.duration}
            onChange={(v) => set('tune', 'cadence', 'duration', v)}
          />
        </Field>
        <Field label="transitions" help={HELP.cadence.transitions}>
          <table class="matrix">
            <thead>
              <tr>
                <th>from \ to</th>
                <For each={CADENCES}>{(to) => <th>{to}</th>}</For>
              </tr>
            </thead>
            <tbody>
              <For each={CADENCES}>
                {(from) => (
                  <tr>
                    <th>{from}</th>
                    <For each={CADENCES}>
                      {(to) => (
                        <td>
                          <NumberInput
                            label={`${from} to ${to}`}
                            min={0}
                            value={tune().cadence.transitions[from][to]}
                            onChange={(v) => set('tune', 'cadence', 'transitions', from, to, v)}
                          />
                        </td>
                      )}
                    </For>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </Field>
      </Group>

      <Group title="Chord">
        <Field label="seed" help={HELP.chord.seed}>
          <LayerSeedInput value={tune().chord.seed} onChange={(v) => set('tune', 'chord', 'seed', v)} />
        </Field>
        <For each={CADENCES}>
          {(cadence) => (
            <Field label={`roots on ${cadence} (degree)`} help={HELP.chord.roots}>
              <WeightedList
                value={tune().chord.roots[cadence]}
                onChange={(v) => set('tune', 'chord', 'roots', cadence, v)}
                newItem={() => ({ value: 0, weight: 1 })}
                renderValue={(value, onChange) => (
                  <NumberInput label="degree" integer value={value()} onChange={onChange} />
                )}
              />
            </Field>
          )}
        </For>
        <Field label="tones (degrees from the root)" help={HELP.chord.tones}>
          <WeightedList
            value={tune().chord.tones}
            onChange={(v) => set('tune', 'chord', 'tones', v)}
            newItem={() => ({ value: [0, 2, 4], weight: 1 })}
            renderValue={(value, onChange) => (
              <IntegerListInput label="degrees" size={10} value={value()} onChange={onChange} />
            )}
          />
        </Field>
      </Group>

      <Group title="Rhythm">
        <Field label="seed" help={HELP.rhythm.seed}>
          <LayerSeedInput value={tune().rhythm.seed} onChange={(v) => set('tune', 'rhythm', 'seed', v)} />
        </Field>
        <Field label="max beat division depth" help={HELP.rhythm.maxDepth}>
          <NumberInput
            integer
            min={0}
            value={tune().rhythm.max_beat_division_depth}
            onChange={(v) => set('tune', 'rhythm', 'max_beat_division_depth', v)}
          />
        </Field>
        <Field label="p_branch edges" help={HELP.rhythm.pBranch}>
          <SmoothstepInput value={tune().rhythm.p_branch} onChange={(v) => set('tune', 'rhythm', 'p_branch', v)} />
        </Field>
        <Field label="p_note_on edges" help={HELP.rhythm.pNoteOn}>
          <SmoothstepInput value={tune().rhythm.p_note_on} onChange={(v) => set('tune', 'rhythm', 'p_note_on', v)} />
        </Field>
      </Group>

      <Group title="Pitch">
        <Field label="seed" help={HELP.pitch.seed}>
          <LayerSeedInput value={tune().pitch.seed} onChange={(v) => set('tune', 'pitch', 'seed', v)} />
        </Field>
        <Field label="abs_pitch_factor edges" help={HELP.pitch.absPitchFactor}>
          <SmoothstepInput
            value={tune().pitch.abs_pitch_factor}
            onChange={(v) => set('tune', 'pitch', 'abs_pitch_factor', v)}
          />
        </Field>
        <Field label="rel_pitch_factor edges" help={HELP.pitch.relPitchFactor}>
          <SmoothstepInput
            value={tune().pitch.rel_pitch_factor}
            onChange={(v) => set('tune', 'pitch', 'rel_pitch_factor', v)}
          />
        </Field>
        <Field label="factor_in_scale" help={HELP.pitch.factorInScale}>
          <NumberInput
            value={tune().pitch.factor_in_scale}
            onChange={(v) => set('tune', 'pitch', 'factor_in_scale', v)}
          />
        </Field>
        <Field label="factor_in_chord" help={HELP.pitch.factorInChord}>
          <NumberInput
            value={tune().pitch.factor_in_chord}
            onChange={(v) => set('tune', 'pitch', 'factor_in_chord', v)}
          />
        </Field>
        <Field label="rhythm_exponent_factor" help={HELP.pitch.rhythmExponentFactor}>
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
    <Group title="Variant" open>
      <JsonFile
        filename="VariantGenParams.json"
        save={() => props.parameters.snapshot().variant}
        load={props.parameters.loadVariant}
        loadDefault={defaultVariant}
      />
      <Field label="seed" help={HELP.variant.seed}>
        <SeedInput value={variant().seed} onChange={(v) => set('variant', 'seed', v)} />
      </Field>
      <Field label="resolution" help={HELP.variant.resolution}>
        <Select
          options={RESOLUTIONS}
          labels={RESOLUTION_LABELS}
          value={variant().resolution}
          onChange={(v) => set('variant', 'resolution', v)}
        />
      </Field>
      <Field label="timespan (beats)" help={HELP.variant.timespan}>
        <WeightedList
          value={variant().timespan}
          onChange={(v) => set('variant', 'timespan', v)}
          newItem={() => ({ value: { start: 0, end: 4 }, weight: 1 })}
          renderValue={(value, onChange) => (
            <>
              <NumberInput
                label="start"
                min={0}
                value={value().start}
                onChange={(start) => onChange({ ...value(), start })}
              />
              <span class="separator">to</span>
              <NumberInput label="end" min={0} value={value().end} onChange={(end) => onChange({ ...value(), end })} />
            </>
          )}
        />
      </Field>
      <Field label="layer" help={HELP.variant.layer}>
        <WeightedList
          value={variant().layer}
          onChange={(v) => set('variant', 'layer', v)}
          newItem={() => ({ value: 'rhythm' as Layer, weight: 1 })}
          renderValue={(value, onChange) => (
            <Select label="layer" options={LAYERS} value={value()} onChange={onChange} />
          )}
        />
      </Field>
    </Group>
  );
}
