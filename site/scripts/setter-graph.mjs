import assert from 'node:assert/strict';
import {mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {build,normalizePath} from 'vite';
import {compileModule} from 'svelte/compiler';
import linguini from '@antepodeum/linguini-svelte/vite';

/** Test the shared Svelte adapter separately from the full Kit route graph below. */
export async function verifySetterGraph(root) {
  for (const generate of ['server','client']) await verifySetterTarget(root,generate);
}

async function verifySetterTarget(root,generate) {
  const entry = '\0linguini-setter-graph';
  const generated = normalizePath(join(root,'src/lib/generated/linguini')) + '/';
  const result = await build({
    configFile:false,root,logLevel:'silent',ssr:{noExternal:true},
    plugins:[linguini({root,buildOnStart:false}),{
      name:'setter-graph-host',enforce:'pre',
      resolveId(id) {if (id === entry || id.startsWith('$app/')) return id;},
      load(id) {
        if (id === entry) return `export {setLocale} from ${JSON.stringify(generated + 'svelte.js')};`;
        if (id === '$app/state') return 'export const page = {data:{linguini:{locale:"en"}}};';
        if (id === '$app/environment') return 'export const browser = true;';
        if (id === '$app/navigation') return 'export async function goto() {}';
        if (id === '$app/server') return 'export function getRequestEvent(){throw new Error("SSR setters must not read a request locale");}';
      },
      transform(code,id) {
        if (id.endsWith('/state.svelte.js')) return compileModule(code,{generate,filename:id}).js.code;
      },
    }],
    build:{ssr:generate === 'server',write:false,target:'es2022',minify:true,rollupOptions:{input:entry,preserveEntrySignatures:'strict'}},
  });
  const chunks = result.output.filter(item => item.type === 'chunk');
  assert.equal(chunks.length,1,'a locale-only import must not emit locale catalog chunks');
  assert.deepEqual(chunks[0].dynamicImports,[]);
  const rendered = Object.entries(chunks[0].modules)
    .filter(([id,module]) => id.startsWith(generated) && module.renderedLength > 0)
    .map(([id]) => id.slice(generated.length)).sort();
  assert.deepEqual(rendered,['locale.js','svelte-locale.svelte.js'],
    'only locale metadata and state may survive a setter-only import');
  const output = join(root,`.svelte-kit/linguini/setter-graph-${generate}.js`);
  mkdirSync(join(root,'.svelte-kit/linguini'),{recursive:true});
  writeFileSync(output,chunks[0].code);
  try {
    const {setLocale} = await import(pathToFileURL(output).href);
    assert.equal(await setLocale('FR'),'fr');
    assert.equal(await setLocale('unconfigured'),'en');
  } finally {
    rmSync(output);
  }
  console.log(`[setter-graph] target=${generate} chunks=1 catalog-modules=0 bytes=${Buffer.byteLength(chunks[0].code)}`);
}
