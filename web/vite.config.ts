import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  // Relative URLs, so the built page works under any path (e.g. GitHub Pages at /auto_compose/).
  base: './',
  server: {
    // Listen on all addresses of the container: the port forwarding of the devcontainer
    // does not reach a server that listens only on `localhost`.
    host: true,
    port: 5173,
    strictPort: true,
    fs: {
      // `auto-composer-wasm` is linked from outside this package (`file:` dependency),
      // and the default parameters of the page are read from `resources/`.
      allow: ['.', '../crates/auto-composer-wasm/pkg', '../resources'],
    },
  },
  build: {
    target: 'esnext',
  },
  optimizeDeps: {
    // The package loads its `.wasm` relative to `import.meta.url`; pre-bundling would break the path.
    exclude: ['auto-composer-wasm'],
  },
});
