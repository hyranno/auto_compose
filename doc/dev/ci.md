# Continuous integration and the web page

`.github/workflows/ci.yml` runs on pushes to `master`, on pull requests and by hand
(`workflow_dispatch`). It has three jobs:

| Job | Steps |
|---|---|
| `rust` | `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` |
| `web` | `wasm-pack test --node crates/auto-composer-wasm`, then builds the page in `web/` (type check and `vite build`) |
| `deploy` | Publishes `web/dist` from the `web` job to GitHub Pages; only on pushes to `master`, after both jobs above pass |

The checks of `rust` are the same as the ones to run before committing (see `golden-fixtures.md`).
Rust is the latest stable toolchain, and Node.js comes from `mise.toml` (via `jdx/mise-action`),
as in the devcontainer.

In the `web` job, `npm run wasm` comes before `npm ci`, because `npm ci` links
`crates/auto-composer-wasm/pkg`, and `npm run build --ignore-scripts` skips `prebuild` so the
wasm package is not built twice.

## GitHub Pages

The page is published at <https://hyranno.github.io/auto_compose/>.
`vite.config.ts` sets `base: './'`, so the built page uses relative URLs and works under any path.

The repository must use GitHub Actions as the source of Pages
(Settings → Pages → Build and deployment → Source: GitHub Actions).
The `github-pages` environment that the `deploy` job uses is created by GitHub with that setting;
by default it accepts deployments only from the default branch.
