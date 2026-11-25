import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {build} from 'vite';

test('server locale reads use the current request across awaits without component or browser context', async () => {
  const entry = '\0request-locale-test';
  const client = fileURLToPath(new URL('../dist/client.js',import.meta.url));
  const result = await build({
    configFile:false,logLevel:'silent',ssr:{noExternal:true},
    plugins:[{
      name:'request-locale-host',enforce:'pre',
      resolveId(id) {if (id === entry || id.startsWith('$app/')) return id;},
      load(id) {
        if (id === entry) return `export {getRequestLocale} from ${JSON.stringify(client)}; export {run} from '$app/server';`;
        if (id === '$app/environment') return 'export const browser = false;';
        if (id === '$app/state') return 'export const page = new Proxy({}, {get(){throw new Error("Server locale reads must not use component context");}});';
        if (id === '$app/navigation') return 'export async function goto(){}';
        if (id === '$app/server') return `import {AsyncLocalStorage} from 'node:async_hooks';
          const requests = new AsyncLocalStorage();
          export const getRequestEvent = () => requests.getStore();
          export const run = (locale,callback) => requests.run({locals:{linguini:locale === undefined ? undefined : {locale}}},callback);`;
      },
    }],
    build:{ssr:true,write:false,target:'es2022',rollupOptions:{input:entry,preserveEntrySignatures:'strict'}},
  });
  const chunks = result.output.filter(item => item.type === 'chunk');
  assert.equal(chunks.length,1);
  const module = await import('data:text/javascript;base64,' + Buffer.from(chunks[0].code).toString('base64'));
  await Promise.all(['fr','ru','en','de',undefined].map(locale => module.run(locale,async () => {
    assert.equal(module.getRequestLocale(),locale);
    await Promise.resolve();
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(module.getRequestLocale(),locale);
  })));
  assert.doesNotMatch(chunks[0].code,/document\.documentElement/);
});
