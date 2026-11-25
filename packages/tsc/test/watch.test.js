import assert from 'node:assert/strict';
import {existsSync,mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
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
