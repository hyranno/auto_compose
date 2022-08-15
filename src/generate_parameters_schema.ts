import * as path from 'path';
import { readFileSync, writeFileSync } from 'fs';

import * as TJS from 'typescript-json-schema';


const settings: TJS.PartialArgs = {
    required: true,
};
const tsconfig = JSON.parse(readFileSync('./tsconfig.json', 'utf8'));
const compilerOptions: TJS.CompilerOptions = {
  ...tsconfig.compilerOptions,
};
const basePath = "./src";

function generateSchemaFile(sourcePath: string, typename: string) {
  const program = TJS.getProgramFromFiles(
    [path.resolve(sourcePath)],
    compilerOptions,
    basePath
  );
  const schema = TJS.generateSchema(program, typename, settings);
  const schemaString = JSON.stringify(schema, null, 2);

  const saveName = typename + ".json";
  const dest = path.resolve('schemas', saveName);
  writeFileSync(dest, schemaString);
}

generateSchemaFile("./src/TuneGenerator.tsx", "TuneGeneratorParameters");
generateSchemaFile("./src/VariantGenerator.tsx", "VariantGeneratorParameters");

/*
const program = TJS.getProgramFromFiles(
  [path.resolve("./src/TuneGenerator.tsx")],
  compilerOptions,
  basePath
);
const typename = "TuneGeneratorParameters";
const schema = TJS.generateSchema(program, typename, settings);
const schemaString = JSON.stringify(schema, null, 2);

const saveName = typename + ".json";
const dest = path.resolve('schemas', saveName);
writeFileSync(dest, schemaString);
*/

//console.log(schemaString);
