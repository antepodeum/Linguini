import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {build} from 'vite';

test('metadata-only request hooks shake catalog barrels and unselected resolver modules', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-metadata-graph-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const web = fileURLToPath(new URL('../../web/dist/',import.meta.url));
  const server = fileURLToPath(new URL('../dist/server.js',import.meta.url));
  await writeFile(path.join(root,'catalog.js'), 'export const messages={title:"FORBIDDEN_CATALOG_DATA"}; export const createLinguini=()=>messages;');
  await writeFile(path.join(root,'runtime.js'), 'export const locales=["en","fr"]; export const baseLocale="en"; export {messages,createLinguini} from "./catalog.js";');
  await writeFile(path.join(root,'entry.js'), `
import {baseLocale,locales} from './runtime.js';
import {createWebLocaleI18n,pathLocale} from '@antepodeum/linguini-web';
import {createHandle,createLoad} from ${JSON.stringify(server)};
const web=createWebLocaleI18n({baseLocale,locales},{routing:{localePrefix:'always'},locale:{sources:[pathLocale()]}});
export const handle=createHandle(web);
export const load=createLoad();
`);
  const result = await build({configFile:false,root,logLevel:'silent',
    resolve:{alias:[{find:'@antepodeum/linguini-web/server',replacement:path.join(web,'server.js')},
      {find:'@antepodeum/linguini-web',replacement:path.join(web,'index.js')}]},
    ssr:{noExternal:true},
    build:{ssr:path.join(root,'entry.js'),write:false,minify:true,sourcemap:true,
      rollupOptions:{external:[/^virtual:linguini\//,'$app/paths'],preserveEntrySignatures:'strict'}},
  });
  const chunks = result.output.filter(item => item.type === 'chunk');
  assert.equal(chunks.length,1);
  const chunk = chunks[0];
  assert.deepEqual(chunk.imports,[]);
  assert.deepEqual(chunk.dynamicImports,[]);
  assert.doesNotMatch(chunk.code,/FORBIDDEN_CATALOG_DATA|createLinguini|prepareLinguini/);
  for (const [id, module] of Object.entries(chunk.modules)) {
    if (/(?:\/catalog\.js|\/sources\/(?:cookie|local-storage|accept-language)\.js)$/.test(id)) {
      assert.equal(module.renderedLength,0,`unexpected metadata-hook dependency: ${id}`);
    }
  }
  const output = await import('data:text/javascript;base64,' + Buffer.from(chunk.code).toString('base64'));
  for (const locale of ['fr','en']) {
    const event={url:new URL(`https://example.test/${locale}/page`),request:new Request(`https://example.test/${locale}/page`),locals:{}};
    const response=await output.handle({event,resolve:async (_event,{transformPageChunk}) => new Response(transformPageChunk({html:'<html lang="%linguini.lang%">%linguini.locale%</html>',done:true}))});
    assert.equal(await response.text(),`<html lang="${locale}">${locale}</html>`);
    assert.equal(output.load(event).linguini.locale,locale);
    assert.equal(Object.hasOwn(event.locals.linguini,'messages'),false);
  }
});
