import { createEffect, For, Index, on, Show, untrack, type JSX } from 'solid-js';
import type { Seed, SmoothstepParams, WeightedItem } from 'auto-composer-wasm';

// Inputs bound to a value of the parameter store: each takes the current value and a setter.
//
// The text inputs keep what the user types. A text that does not parse is marked invalid
// (with `setCustomValidity`, so the form refuses to submit) and is not written to the store,
// which keeps its last valid value; a text that parses is written at once.

type FieldProps<T> = { value: T; onChange: (value: T) => void };

type Parsed<T> = { value: T } | { error: string };

/** A text input of a value of type `T`, written as `format` gives and read back by `parse`. */
function ParsedInput<T>(
  props: FieldProps<T> & {
    format: (value: T) => string;
    parse: (text: string) => Parsed<T>;
    label?: string;
    placeholder?: string;
    size?: number;
  },
) {
  let input!: HTMLInputElement;
  // Rewrites the text only when the value differs from what the text means,
  // so that a text being typed, such as `-` or `1.`, is not replaced.
  // The value is tracked through its JSON, because loading a file changes the arrays
  // of the store in place.
  createEffect(
    on(
      () => JSON.stringify(props.value),
      (json) => {
        const parsed = props.parse(input.value);
        if (!('value' in parsed) || JSON.stringify(parsed.value) !== json) {
          input.value = props.format(untrack(() => props.value));
          input.setCustomValidity('');
        }
      },
    ),
  );
  const onInput = () => {
    const parsed = props.parse(input.value);
    if ('value' in parsed) {
      input.setCustomValidity('');
      props.onChange(parsed.value);
    } else {
      input.setCustomValidity(parsed.error);
    }
  };
  return (
    <input
      ref={input}
      type="text"
      autocomplete="off"
      size={props.size ?? 6}
      aria-label={props.label}
      title={props.label}
      placeholder={props.placeholder}
      onInput={onInput}
    />
  );
}

export type NumberRule = { integer?: boolean; min?: number; max?: number };

function parseNumber(text: string, rule: NumberRule = {}): Parsed<number> {
  const trimmed = text.trim();
  const value = Number(trimmed);
  if (trimmed === '' || !Number.isFinite(value)) return { error: 'Enter a number.' };
  if (rule.integer && !Number.isInteger(value)) return { error: 'Enter an integer.' };
  if (rule.min !== undefined && value < rule.min) return { error: `Enter ${rule.min} or more.` };
  if (rule.max !== undefined && value > rule.max) return { error: `Enter ${rule.max} or less.` };
  return { value };
}

export function NumberInput(props: FieldProps<number> & NumberRule & { label?: string; size?: number }) {
  return (
    <ParsedInput
      value={props.value}
      onChange={props.onChange}
      label={props.label}
      size={props.size}
      format={String}
      parse={(text) => parseNumber(text, props)}
    />
  );
}

/** Integers separated by commas or spaces, such as `0, 2, 4`. */
export function IntegerListInput(props: FieldProps<number[]> & { label?: string; size?: number }) {
  const parse = (text: string): Parsed<number[]> => {
    const items = text.split(/[\s,]+/).filter((item) => item !== '');
    const values: number[] = [];
    for (const item of items) {
      const parsed = parseNumber(item, { integer: true });
      if (!('value' in parsed)) return { error: 'Enter integers separated by commas.' };
      values.push(parsed.value);
    }
    return { value: values };
  };
  return (
    <ParsedInput
      value={props.value}
      onChange={props.onChange}
      label={props.label}
      size={props.size ?? 16}
      format={(values) => values.join(', ')}
      parse={parse}
    />
  );
}

const SEED_RULE: NumberRule = { integer: true, min: 0, max: 2 ** 32 - 1 };

/** A seed: an integer from 0 to 2^32 - 1. */
export function SeedInput(props: FieldProps<Seed>) {
  return <NumberInput {...SEED_RULE} size={12} value={props.value} onChange={props.onChange} />;
}

/** The seed of a layer. Left empty, the layer uses the seed of the tune. */
export function LayerSeedInput(props: FieldProps<Seed | undefined>) {
  return (
    <ParsedInput
      value={props.value}
      onChange={props.onChange}
      size={12}
      placeholder="tune seed"
      format={(value) => (value === undefined ? '' : String(value))}
      parse={(text) => (text.trim() === '' ? { value: undefined } : parseNumber(text, SEED_RULE))}
    />
  );
}

export function SmoothstepInput(props: FieldProps<SmoothstepParams>) {
  return (
    <>
      <NumberInput
        label="edge0"
        value={props.value.edge0}
        onChange={(edge0) => props.onChange({ ...props.value, edge0 })}
      />
      <span class="separator">to</span>
      <NumberInput
        label="edge1"
        value={props.value.edge1}
        onChange={(edge1) => props.onChange({ ...props.value, edge1 })}
      />
    </>
  );
}

export function Select<T extends string>(
  props: FieldProps<T> & { options: readonly T[]; labels?: Record<T, string>; label?: string },
) {
  // `selected` on each option rather than `value` on the select, which would be set before the options exist.
  return (
    <select aria-label={props.label} onChange={(e) => props.onChange(e.currentTarget.value as T)}>
      <For each={props.options}>
        {(option) => (
          <option value={option} selected={option === props.value}>
            {props.labels?.[option] ?? option}
          </option>
        )}
      </For>
    </select>
  );
}

/**
 * A list of weighted items: a row of `renderValue` and the weight per item,
 * with buttons to remove an item and to add one (a copy of `newItem()`).
 */
export function WeightedList<T>(
  props: FieldProps<WeightedItem<T>[]> & {
    newItem: () => WeightedItem<T>;
    renderValue: (value: () => T, onChange: (value: T) => void) => JSX.Element;
  },
) {
  const update = (index: number, item: WeightedItem<T>) =>
    props.onChange(props.value.map((old, i) => (i === index ? item : old)));
  return (
    <div class="weighted-list">
      {/* `Index`, not `For`: a row keeps its inputs (and their focus) while its item is replaced. */}
      <Index each={props.value}>
        {(item, index) => (
          <div class="row">
            {props.renderValue(() => item().value, (value) => update(index, { ...item(), value }))}
            <span class="separator">weight</span>
            <NumberInput
              label="weight"
              min={0}
              value={item().weight}
              onChange={(weight) => update(index, { ...item(), weight })}
            />
            <button
              type="button"
              title="remove"
              onClick={() => props.onChange(props.value.filter((_, i) => i !== index))}
            >
              ×
            </button>
          </div>
        )}
      </Index>
      <button type="button" onClick={() => props.onChange([...props.value, props.newItem()])}>
        add
      </button>
    </div>
  );
}

/** A labelled row of inputs, with the explanation `help` shown as a tooltip of a `?` after the label. */
export function Field(props: { label: string; help?: string; children: JSX.Element }) {
  return (
    <div class="field">
      <span class="field-label">
        {props.label}
        <Show when={props.help}>
          {(help) => (
            // Focusable, so that the tooltip shows on a tap or from the keyboard as well as on hover.
            <span class="help" tabindex="0" aria-label={help()}>
              ?<span class="tooltip" role="tooltip">{help()}</span>
            </span>
          )}
        </Show>
      </span>
      <div class="field-inputs">{props.children}</div>
    </div>
  );
}

/** A collapsible group of fields. */
export function Group(props: { title: string; open?: boolean; children: JSX.Element }) {
  return (
    <details open={props.open}>
      <summary>{props.title}</summary>
      <div class="group-body">{props.children}</div>
    </details>
  );
}
