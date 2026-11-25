import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import {runtimeOutput} from '../src/projects.js';

test('native output arguments use portable project-relative paths on every host', () => {
  const root = path.resolve('output-argument-fixture');
  const layout = {root,generated:path.join(root,'src/generated')};
  const parsed = {fileNames:[path.join(root,'src/app.ts')],options:{
    configFilePath:path.join(root,'tsconfig.json'),rootDir:path.join(root,'src'),
    outDir:path.join(root,'dist'),declarationDir:path.join(root,'types'),
  }};
  assert.equal(runtimeOutput(parsed,layout),'dist/generated');
  assert.equal(runtimeOutput(parsed,layout,true),'types/generated');
  assert.equal(runtimeOutput({...parsed,options:{...parsed.options,outDir:root}},layout),'generated');
  assert.equal(runtimeOutput({...parsed,options:{...parsed.options,outDir:path.join(root,'src')}},layout),undefined);
});
