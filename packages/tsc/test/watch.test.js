import assert from 'node:assert/strict';
import fs from 'node:fs';
import {EventEmitter} from 'node:events';
import {existsSync,mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import ts from 'typescript';
import {watchNative} from '../src/watch.js';

test('native watches accept normalized, relative, and ancestor events while excluding sibling sources and disposing resources', context => {
  const root = mkdtempSync(path.join(tmpdir(),'linguini-native-watch-'));
  context.after(() => rmSync(root,{recursive:true,force:true}));
  const schema = path.join(root,'schema');
  const locale = path.join(root,'locale');
  const missing = path.join(root,'future','schema');
  mkdirSync(schema);
  mkdirSync(locale);
  writeFileSync(path.join(schema,'main.lgs'),'title\n');
  writeFileSync(path.join(locale,'en.lgl'),'title = Hello\n');
  const files = new Map();
  const directories = new Map();
  let closed = 0;
  let changes = 0;
  const close = watchNative({configFiles:new Set([path.join(root,'tsconfig.json')])},[
    {configFile:path.join(root,'linguini.toml'),sources:[schema,locale,missing]},
  ],() => {changes++;},{
    directoryExists:existsSync,
    watchFile(file,callback) {files.set(file,callback);return {close(){closed++;}};},
    watchDirectory(directory,callback,recursive) {
      assert.equal(recursive,true);
      directories.set(directory,callback);
      return {close(){closed++;}};
    },
  });
  assert.deepEqual([...directories.keys()].sort(),[root,schema,locale].sort());
  assert.equal(files.size,4,'watch both configuration files and the discovered schema/locale files');
  directories.get(schema)('new.lgs');
  directories.get(locale)(path.join(locale,'new.lgl').replaceAll('\\','/'));
  assert.equal(changes,2);
  const spelling = path.dirname(schema) + path.sep + path.sep + path.basename(schema) + '/secondary.lgs';
  directories.get(schema)(spelling);
  assert.equal(changes,3,'normalize callback paths before testing source membership');
  directories.get(root)(path.join(root,'future'));
  assert.equal(changes,4,'creation of a missing source ancestor must trigger rediscovery');
  for (const directory of ['schema-sibling','unrelated']) directories.get(root)(path.join(root,directory,'new.lgs'));
  assert.equal(changes,4,'unrelated siblings must not trigger a native rebuild');
  close();
  assert.equal(closed,files.size + directories.size);
});

test('native source edits are observed when filesystem event watchers silently drop changes', async context => {
  const root = mkdtempSync(path.join(tmpdir(),'linguini-polling-watch-'));
  context.after(() => rmSync(root,{recursive:true,force:true}));
  const source = path.join(root,'locale');
  mkdirSync(source);
  const file = path.join(source,'en.lgl');
  writeFileSync(file,'title = Before\n');
  const configFile = path.join(root,'linguini.toml');
  writeFileSync(configFile,'');
  // A successfully opened event watcher can still report no events. Polling
  // must observe the real file without relying on an error-triggered fallback.
  context.mock.method(fs,'watch',() => {
    const watcher = new EventEmitter();
    watcher.close = () => {};
    return watcher;
  });
  let observe;
  const changed = new Promise(resolve => {observe = resolve;});
  const close = watchNative({configFiles:new Set()},[{configFile,sources:[source]}],observe,{
    ...ts.sys,
    watchDirectory() {return {close(){}};},
  });
  let timer;
  try {
    writeFileSync(file,'title = After\n');
    const modified = new Date(Date.now() + 1000);
    fs.utimesSync(file,modified,modified);
    const observed = await Promise.race([
      changed,
      new Promise((_,reject) => {timer = setTimeout(() => reject(new Error('Source edit was lost without filesystem events')),3000);}),
    ]);
    assert.equal(observed,file);
    assert.equal(fs.readFileSync(file,'utf8'),'title = After\n');
  } finally {
    clearTimeout(timer);
    close();
  }
});
