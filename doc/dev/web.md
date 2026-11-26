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

## Dev server and the Claude Code sandbox

A server started inside the Claude Code sandbox cannot be reached from the browser.
Start `npm run dev` outside the sandbox (for example from a terminal of the devcontainer);
port 5173 is forwarded by the devcontainer.
