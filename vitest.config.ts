import { defineConfig } from 'vitest/config';

// Tests cover only the UI-independent .ts modules, so the Solid plugin
// (which switches the environment to jsdom) is intentionally not loaded.
export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
});
