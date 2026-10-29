import assert from 'node:assert/strict';
import {mkdirSync,mkdtempSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {normalizePath,parseAst} from 'vite';
import {clientPages} from '../dist/build/routes.js';

function fixture(context, dictionary) {
  const root = mkdtempSync(path.join(tmpdir(),'linguini-kit-routes-'));
  context.after(() => rmSync(root,{recursive:true,force:true}));
  const generated = path.join(root,'generated/client-optimized');
  mkdirSync(path.join(generated,'nodes'),{recursive:true});
  const app = normalizePath(path.join(generated,'app.js'));
  const source = `export const nodes = [${[0,1,2,3,4,5].map(index => `() => import('./nodes/${index}')`).join(',')}];
export const dictionary = ${dictionary};`;
  writeFileSync(app,source);
  const modules = new Map([[app,{exports:['nodes','dictionary']}]]);
  for (let index=0;index<6;index++) {
    const id = normalizePath(path.join(generated,`nodes/${index}.js`));
    const universal = [0,3,4].includes(index);
    writeFileSync(id,`export {default as component} from './component.svelte';${universal ? `import * as universal from './universal-${index}.js'; export {universal};` : ''}`);
    modules.set(id,{exports:universal ? ['component','universal'] : ['component']});
    if (universal) modules.set(normalizePath(path.join(generated,`nodes/universal-${index}.js`)),{exports:index===3 ? ['csr','_unused'] : ['load','_unused']});
  }
  return {
    app,source,modules,
    entry(index, load=false) {return {id:normalizePath(path.join(generated,`nodes/${index}.js`)),exports:load ? [['component'],['universal','load']] : [['component']]};},
    context:{parse:parseAst,getModuleIds:() => modules.keys(),getModuleInfo:id => modules.get(id) ?? null},
  };
}

test('Kit route descriptors retain composed roots, holes, complements, and only consumed universal exports', context => {
  const model = fixture(context,'{"/": [~3], "/nested/[slug]": [4,[2,,2]], "/reset": [5]}');
  assert.deepEqual(clientPages(model.context),[
    {id:'/',entries:[model.entry(0,true),model.entry(3)]},
    {id:'/nested/[slug]',entries:[model.entry(0,true),model.entry(2),model.entry(4,true)]},
    {id:'/reset',entries:[model.entry(0,true),model.entry(5)]},
  ]);
});

test('unsupported Kit manifest shapes fail closed rather than widening a page', context => {
  for (const dictionary of [
    'routes()', '{[route]: [3]}', '{"/": [nodes()] }', '{"/": [40]}',
    '{"/": [-3]}', '{"/": [3, layouts]}',
  ]) {
    const model = fixture(context,dictionary);
    assert.throws(() => clientPages(model.context),/SvelteKit/);
  }
  const model = fixture(context,'{"/": [3]}');
  assert.throws(() => clientPages({...model.context,getModuleIds:() => []}),/one SvelteKit/);
  model.modules.get(model.entry(0).id).exports = null;
  assert.throws(() => clientPages(model.context),/node exports/);
  const serverRoutes = fixture(context,'{}');
  serverRoutes.modules.set('\0virtual:linguini/message/00',{});
  assert.throws(() => clientPages(serverRoutes.context),/server-only route resolution/);
});
