import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 5173,
    strictPort: true,
    fs: {
      // `auto-composer-wasm` is linked from outside this package (`file:` dependency).
      allow: ['.', '../crates/auto-composer-wasm/pkg'],
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
