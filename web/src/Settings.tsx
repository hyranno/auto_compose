import { createSignal, Show } from 'solid-js';

import { Field, Group, NumberInput, SeedInput, Select, SmoothstepInput } from './fields';
import { download, readChosenFile } from './file';
import { MAJOR_TONES, MINOR_TONES, RESOLUTIONS, type Parameters } from './params';

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
        <Select options={RESOLUTIONS} value={tune().resolution} onChange={(v) => set('tune', 'resolution', v)} />
      </Field>
      <Field label="time measure">
        <NumberInput step={1} value={tune().time_measure[0]} onChange={(v) => set('tune', 'time_measure', 0, v)} />
        <NumberInput step={1} value={tune().time_measure[1]} onChange={(v) => set('tune', 'time_measure', 1, v)} />
      </Field>
      <Group title="Cadence Generator">
        <Field label="seeds">
          <SeedInput value={tune().cadence.seed} onChange={(v) => set('tune', 'cadence', 'seed', v)} />
        </Field>
        <Field label="duration">
          <NumberInput value={tune().cadence.duration} onChange={(v) => set('tune', 'cadence', 'duration', v)} />
        </Field>
      </Group>
      <Group title="Chord Generator">
        <Field label="root seeds">
          <SeedInput value={tune().chord.root.seed} onChange={(v) => set('tune', 'chord', 'root', 'seed', v)} />
        </Field>
        <Field label="tone seeds">
          <SeedInput
            value={tune().chord.tone.rand_tones.seed}
            onChange={(v) => set('tune', 'chord', 'tone', 'rand_tones', 'seed', v)}
          />
        </Field>
      </Group>
      <Group title="Note Generator">
        <Group title="Rhythm">
          <Field label="seeds">
            <SeedInput
              value={tune().note.rhythm.seed}
              onChange={(v) => set('tune', 'note', 'rhythm', 'seed', v)}
            />
          </Field>
          <Field label="max beat division depth">
            <NumberInput
              step={1}
              value={tune().note.rhythm.max_beat_division_depth}
              onChange={(v) => set('tune', 'note', 'rhythm', 'max_beat_division_depth', v)}
            />
          </Field>
          <Field label="pBranch edges">
            <SmoothstepInput
              value={tune().note.rhythm.pBranch}
              onChange={(v) => set('tune', 'note', 'rhythm', 'pBranch', v)}
            />
          </Field>
          <Field label="pNoteOn edges">
            <SmoothstepInput
              value={tune().note.rhythm.pNoteOn}
              onChange={(v) => set('tune', 'note', 'rhythm', 'pNoteOn', v)}
            />
          </Field>
        </Group>
        <Group title="Pitch Weight">
          <Field label="absPitchFactor edges">
            <SmoothstepInput
              value={tune().note.pitchWeight.absPitchFactor}
              onChange={(v) => set('tune', 'note', 'pitchWeight', 'absPitchFactor', v)}
            />
          </Field>
          <Field label="relPitchFactor edges">
            <SmoothstepInput
              value={tune().note.pitchWeight.relPitchFactor}
              onChange={(v) => set('tune', 'note', 'pitchWeight', 'relPitchFactor', v)}
            />
          </Field>
          <Field label="factorInScale">
            <NumberInput
              value={tune().note.pitchWeight.factorInScale}
              onChange={(v) => set('tune', 'note', 'pitchWeight', 'factorInScale', v)}
            />
          </Field>
          <Field label="factorInChord">
            <NumberInput
              value={tune().note.pitchWeight.factorInChord}
              onChange={(v) => set('tune', 'note', 'pitchWeight', 'factorInChord', v)}
            />
          </Field>
          <Field label="rhythmExponentFactor">
            <NumberInput
              value={tune().note.pitchWeight.rhythmExponentFactor}
              onChange={(v) => set('tune', 'note', 'pitchWeight', 'rhythmExponentFactor', v)}
            />
          </Field>
        </Group>
        <Field label="pitch seeds">
          <SeedInput value={tune().note.seed} onChange={(v) => set('tune', 'note', 'seed', v)} />
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
      <Field label="seeds">
        <SeedInput value={variant().seed} onChange={(v) => set('variant', 'seed', v)} />
      </Field>
      <Field label="resolution">
        <Select options={RESOLUTIONS} value={variant().resolution} onChange={(v) => set('variant', 'resolution', v)} />
      </Field>
    </Group>
  );
}
