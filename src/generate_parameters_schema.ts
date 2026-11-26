import * as path from 'path';
import { readFileSync, writeFileSync } from 'fs';

import * as TJS from 'typescript-json-schema';


const settings: TJS.PartialArgs = {
  required: true,
  uniqueNames: true,
};
const tsconfig = JSON.parse(readFileSync('./tsconfig.json', 'utf8'));
const compilerOptions: TJS.CompilerOptions = {
  ...tsconfig.compilerOptions,
};
const basePath = "./src";

// `uniqueNames` appends a hash to every definition name ("Resolution.6cac220e").
// Drop the hash wherever the plain name is still unambiguous.
function stripUniqueNameSuffixes(text: string, schema: TJS.Definition): string {
  const names = Object.keys(schema.definitions ?? {});
  const base = (n: string) => n.replace(/\.[0-9a-f]{8}$/, '');
  for (const name of names) {
    if (base(name) == name) continue;
    if (names.filter(n => base(n) == base(name)).length > 1) continue;
    text = text.replaceAll(`"#/definitions/${name}"`, `"#/definitions/${base(name)}"`)
               .replaceAll(`"${name}": {`, `"${base(name)}": {`);
  }
  return text;
}

// Every generator module exports its own `Parameters` type, so the symbol is
// looked up by both name and declaring file.
function generateSchemaFile(sourcePath: string, typename: string, saveBasename: string) {
  const absPath = path.resolve(sourcePath);
  const program = TJS.getProgramFromFiles([absPath], compilerOptions, basePath);
  const generator = TJS.buildGenerator(program, settings);
  if (!generator) throw new Error(`failed to build generator for ${sourcePath}`);
  const symbol = generator.getSymbols(typename)
    .find(s => path.resolve(s.fullyQualifiedName.replace(/^"(.*)"\..*$/, '$1')) + path.extname(absPath) == absPath);
  if (!symbol) throw new Error(`type ${typename} not found in ${sourcePath}`);
  const schema = generator.getSchemaForSymbol(symbol.name);
  const schemaString = stripUniqueNameSuffixes(JSON.stringify(schema, null, 2), schema);

  const dest = path.resolve('schemas', saveBasename + ".json");
  writeFileSync(dest, schemaString);
}

generateSchemaFile("./src/core/tune_generator.ts", "Parameters", "TuneGeneratorParameters");
generateSchemaFile("./src/core/variant.ts", "Parameters", "VariantGeneratorParameters");
