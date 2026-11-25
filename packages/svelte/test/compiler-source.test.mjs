import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import test from 'node:test';
import MagicString from 'magic-string';
import {compile,parse} from 'svelte/compiler';
import {svelteAdapter} from '../dist/vite.js';

test('generic Svelte source accepts scoped messages, links, and cleanup in both compiler targets', async () => {
  const source = await readFile(new URL('./fixtures/generic-component.svelte',import.meta.url),'utf8');
  const filename = 'Generic.svelte';
  const ast = parse(source,{modern:true});
  const original = ast.instance.content.body.find(node => node.type === 'ImportDeclaration');
  const scope = svelteAdapter.applicationScope(source,original.start,filename);
  const insertion = Buffer.from(source).subarray(0,scope.insertionByte).toString('utf8').length;
  assert.equal(insertion,ast.instance.content.start);
  const magic = new MagicString(source);
  magic.remove(original.start,original.end);
  magic.prependLeft(insertion,'import { message } from "./message.js";\n');
  magic.overwrite(source.indexOf('l.main.title'),source.indexOf('l.main.title') + 'l.main.title'.length,'message()');
  svelteAdapter.applyStaticLinks(magic,source,filename,'generated',svelteAdapter.findStaticLinks(source,filename),'./links.js');
  svelteAdapter.injectLocaleLoaderLifecycle(magic,source,{relative:filename},{
    prelude:'const release = () => {};\n',release:'release',allocateAlias:prefix => prefix + '0',
  });
  const transformed = magic.toString();
  const updated = parse(transformed,{modern:true});
  const imports = updated.instance.content.body.filter(node => node.type === 'ImportDeclaration');
  assert.deepEqual(imports.map(node => node.source.value).sort(),['./links.js','./message.js','svelte']);
  assert.equal(updated.module.content.body.length,1,'injected imports must not enter the module script');
  for (const generate of ['server','client']) {
    const result = compile(transformed,{filename,generate});
    assert.deepEqual(result.warnings,[]);
    assert.match(result.js.code,/message\(\)/);
    assert.match(result.js.code,/localizeTransformedHref/);
    assert.match(result.js.code,/onDestroy/);
  }
});
