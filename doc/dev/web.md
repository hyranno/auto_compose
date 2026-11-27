# Example web page

`web/` is the example page of auto-composer. It is an independent npm package
(Vite, Solid, TypeScript) that calls the core crate through the WebAssembly bindings
described in [wasm.md](wasm.md).

## Commands

Run them in `web/`:

```sh
npm install         # once; links ../crates/auto-composer-wasm/pkg, which must already exist
npm run dev         # builds the wasm package, then serves the page on http://localhost:5173
npm run build       # builds the wasm package, type-checks and writes dist/
npm run typecheck   # tsc --noEmit only
npm run wasm        # wasm-pack build ../crates/auto-composer-wasm --target web
```

`predev` and `prebuild` rebuild the wasm package, so changes in the crates are picked up
by restarting `npm run dev`.

## How the wasm package is wired

- `package.json` depends on `"auto-composer-wasm": "file:../crates/auto-composer-wasm/pkg"`,
  so `npm install` needs the package built once (`npm run wasm` or `wasm-pack build`).
- `src/index.tsx` awaits the default export `init()` of the package before rendering;
  after that, its functions are synchronous.
- `vite.config.ts` excludes the package from dependency pre-bundling, because it loads its `.wasm`
  relative to `import.meta.url`, and adds its directory to `server.fs.allow`, because the linked package
  lies outside `web/`.

## Source layout

| File | Contents |
|---|---|
| `src/params.ts` | The parameter store: all the parameters as one Solid store of the plain JSON the wasm functions take |
| `src/help.ts` | The explanations of the fields, shown as tooltips |
| `src/Settings.tsx` | The forms of the tune and variant parameters, and saving and loading them as JSON |
| `src/fields.tsx` | Inputs bound to a value of the store (number, integer list, seed, smoothstep, select, weighted list) |
| `src/PianoRoll.tsx` | The piano roll of a `TuneSnapshot` |
| `src/Output.tsx` | Playing the tune (Tone.js) and exporting it as MIDI (@tonejs/midi) |
| `src/tune.ts` | Helpers on `TuneSnapshot`: lookups on the timelines, pitch classes, the melody and chord tracks |
| `src/file.ts` | Downloading and reading files |

The defaults of the page come from `resources/`: the tune parameters are
`resources/TuneGenParams_minor.json`, imported as a JSON module (hence `../resources` in
`server.fs.allow`), and the variant parameters come from `defaultVariantParameters()`.
They are what the page starts with and what the load default buttons restore.
The page starts at 120 BPM (`DEFAULT_BPM` in `src/params.ts`). The forms cover all the parameters.

The inputs (`ParsedInput` in `src/fields.tsx`) keep what the user types. Numbers are
`type="number"` inputs (a partial text such as `-` shows there as `validity.badInput`), and lists of
integers are text inputs. A text that parses
is written to the store at once; one that does not is marked with `setCustomValidity` and leaves the
store at its last valid value, and `generate` refuses to run while such an input remains.
Only the syntax (a number, an integer, the range of a seed) is checked there;
the generators check the rest and report it as an error message.
A loaded file replaces the parameters only if a tune (or a variant) can be generated from it;
otherwise the page shows the error and keeps the current parameters.

## Dev server and the Claude Code sandbox

A server started inside the Claude Code sandbox cannot be reached from the browser.
Start `npm run dev` outside the sandbox (for example from a terminal of the devcontainer);
port 5173 is forwarded by the devcontainer.
The server listens on all addresses (`server.host` in `vite.config.ts`), because the forwarding
does not reach a server that listens only on `localhost`.
