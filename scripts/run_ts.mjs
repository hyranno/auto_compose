// Runs a TypeScript module with Vite's module runner, which resolves the
// extensionless imports used under src/ (plain node and ts-node --esm do not).
// Usage: node scripts/run_ts.mjs <file.ts>
import {runnerImport} from 'vite';

const target = process.argv[2];
if (!target) {
  console.error('usage: node scripts/run_ts.mjs <file.ts>');
  process.exit(1);
}
await runnerImport(target);
