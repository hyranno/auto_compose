import { For, type JSX } from 'solid-js';
import type { RandomSeed, SmoothstepParams } from 'auto-composer-wasm';

// Inputs bound to a value of the parameter store: each takes the current value and a setter.

type FieldProps<T> = { value: T; onChange: (value: T) => void };

/**
 * A number input. An empty or malformed input gives `NaN`, which the generators reject
 * with an error message instead of producing a tune.
 */
export function NumberInput(props: FieldProps<number> & { step?: number }) {
  return (
    <input
      type="number"
      step={props.step ?? 'any'}
      value={props.value}
      onInput={(e) => props.onChange(e.currentTarget.valueAsNumber)}
    />
  );
}

export function Select<T extends string>(props: FieldProps<T> & { options: readonly T[] }) {
  return (
    <select value={props.value} onInput={(e) => props.onChange(e.currentTarget.value as T)}>
      <For each={props.options}>{(option) => <option value={option}>{option}</option>}</For>
    </select>
  );
}

export function SeedInput(props: FieldProps<RandomSeed>) {
  return (
    <>
      <NumberInput step={1} value={props.value.state} onChange={(state) => props.onChange({ ...props.value, state })} />
      <NumberInput
        step={1}
        value={props.value.sequence}
        onChange={(sequence) => props.onChange({ ...props.value, sequence })}
      />
    </>
  );
}

export function SmoothstepInput(props: FieldProps<SmoothstepParams>) {
  return (
    <>
      <NumberInput value={props.value.edge0} onChange={(edge0) => props.onChange({ ...props.value, edge0 })} />
      <NumberInput value={props.value.edge1} onChange={(edge1) => props.onChange({ ...props.value, edge1 })} />
    </>
  );
}

/** A labelled row of inputs. */
export function Field(props: { label: string; children: JSX.Element }) {
  return (
    <label>
      <span class="field-label">{props.label}</span>
      {props.children}
    </label>
  );
}

/** A collapsible group of fields. */
export function Group(props: { title: string; children: JSX.Element }) {
  return (
    <details>
      <summary>{props.title}</summary>
      {props.children}
    </details>
  );
}
