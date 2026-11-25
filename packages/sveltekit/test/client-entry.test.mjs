import assert from 'node:assert/strict';
import {mkdtemp,writeFile,mkdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {build} from 'vite';
import {clientEntry} from '../dist/build/client-entry.js';

test('optional client capabilities use the existing app entry and preserve manifest identity', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-kit-entry-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const directory = path.join(root,'generated/client-optimized');
  await mkdir(directory,{recursive:true});
  const application = path.join(directory,'app.js');
  await writeFile(application,'export {value} from "./shared.js";');
  await writeFile(path.join(directory,'shared.js'),'import "./cycle.js"; export const value = 42;');
  await writeFile(path.join(directory,'cycle.js'),'import "./app.js"; export const unused = "FORBIDDEN";');
  for (const enabled of [false,true]) {
    let prepared = false;
    const result = await build({
      configFile:false,root,logLevel:'silent',
      plugins:[clientEntry(async context => {
        assert.deepEqual(context.getModuleInfo(application).exports,['value']);
        assert.ok(context.getModuleInfo(path.join(directory,'cycle.js'))?.code);
        prepared = true;
        return enabled ? 'export function optionalCapability(){return "CAPABILITY";}' : 'export {};';
      })],
      build:{write:false,minify:true,manifest:true,sourcemap:true,
        rollupOptions:{input:{app:application},preserveEntrySignatures:'strict'}},
    });
    assert.equal(prepared,true);
    const chunks = result.output.filter(item => item.type === 'chunk');
    assert.equal(chunks.length,1,'the app facade must not add a helper chunk');
    assert.deepEqual(chunks[0].imports,[]);
    assert.doesNotMatch(chunks[0].code,/FORBIDDEN/);
    const module = await import('data:text/javascript;base64,' + Buffer.from(chunks[0].code).toString('base64'));
    assert.equal(module.value,42);
    assert.equal(Object.hasOwn(module,'optionalCapability'),enabled);
    if (enabled) assert.equal(module.optionalCapability(),'CAPABILITY');
    else assert.doesNotMatch(chunks[0].code,/CAPABILITY/);
    const manifest = JSON.parse(result.output.find(item => item.fileName === '.vite/manifest.json').source);
    assert.deepEqual(Object.keys(manifest),['generated/client-optimized/app.js']);
    assert.equal(manifest['generated/client-optimized/app.js'].src,'generated/client-optimized/app.js');
    assert.equal(manifest['generated/client-optimized/app.js'].file,chunks[0].fileName);
    const map = JSON.parse(result.output.find(item => item.fileName.endsWith('.js.map')).source);
    assert.ok(map.sources.some(source => source.endsWith('/shared.js')));
  }
});
