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
| `src/Settings.tsx` | The forms of the tune and variant parameters, and saving and loading them as JSON |
| `src/fields.tsx` | Inputs bound to a value of the store (number, seed, smoothstep, select) |
| `src/PianoRoll.tsx` | The piano roll of a `TuneSnapshot` |
| `src/Output.tsx` | Playing the tune (Tone.js) and exporting it as MIDI (@tonejs/midi) |
| `src/tune.ts` | Helpers on `TuneSnapshot`: lookups on the timelines, pitch classes, the melody and chord tracks |
| `src/file.ts` | Downloading and reading files |

The defaults of the store come from `defaultTuneParameters()` and `defaultVariantParameters()`,
so the page holds no defaults of its own. The forms cover only part of the parameters
(seeds, scale, measures, rhythm, pitch, ...); the others, such as the probability tables,
are changed by editing a saved JSON file and loading it.
A loaded file replaces the parameters only if a tune (or a variant) can be generated from it;
otherwise the page shows the error and keeps the current parameters.

## Dev server and the Claude Code sandbox

A server started inside the Claude Code sandbox cannot be reached from the browser.
Start `npm run dev` outside the sandbox (for example from a terminal of the devcontainer);
port 5173 is forwarded by the devcontainer.
The server listens on all addresses (`server.host` in `vite.config.ts`), because the forwarding
does not reach a server that listens only on `localhost`.
