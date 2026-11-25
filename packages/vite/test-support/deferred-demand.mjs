import assert from 'node:assert/strict';
import {mkdtemp,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';

/** Run the same dependency-closure contract against the consuming Vite host. */
export async function verifyDeferredDemand({build,pageMessagePlan}) {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-deferred-plan-'));
  try {
    const virtual = name => 'virtual:linguini/message/' + Buffer.from(name).toString('hex');
    const files = {
      'shared.js':['first','one','two','unused'].map(name =>
        `import {message as ${name}Message} from ${JSON.stringify(virtual(name))}; export function ${name}(){ return ${name}Message(); }`).join('\n'),
      'one.js':`import {one} from './shared.js'; import {message} from ${JSON.stringify(virtual('effect'))};
        globalThis.effect = message();
        export async function render(){ const {message} = await import('./later.js'); return one() + message(); }
        export {unused as debug} from './shared.js'; export async function dead(){ return import('./never.js'); }`,
      'two.js':'export {two as default,unused as debug} from "./shared.js";',
      'never.js':`export {message} from ${JSON.stringify(virtual('never'))};`,
      'later.js':`export {message} from ${JSON.stringify(virtual('later'))};`,
      'entry.js':`export {first as render} from './shared.js';
        export async function one(){ const {render} = await import('./one.js'); return render(); }
        export async function two(){ return (await import('./two.js')).default(); }
        export async function escape(){ return import('./two.js'); }`,
    };
    for (const [name,source] of Object.entries(files)) await writeFile(path.join(root,name),source);
    for (const [escape,target] of [[false,'es2018'],[false,'es2022'],[true,'es2022']]) {
      let receipt;
      await build({
        configFile:false,root,logLevel:'silent',
        plugins:[{
          name:'deferred-plan-fixture',
          resolveId(id) {if (id.startsWith('virtual:')) return '\0' + id;},
          load(id) {if (id.startsWith('\0virtual:')) return 'export function message(){ return globalThis.message(); }';},
          async generateBundle() {
            receipt = await pageMessagePlan(this,[{id:path.join(root,'entry.js'),
              exports:(escape ? ['render','one','two','escape'] : ['render','one','two']).map(name => [name])}],
            {root,build:{target}});
          },
        }],
        build:{write:false,rollupOptions:{input:path.join(root,'entry.js'),preserveEntrySignatures:'strict'}},
      });
      assert.deepEqual(receipt.messages,['first']);
      assert.deepEqual(receipt.deferred.map(boundary => ({
        id:path.basename(boundary.id),exports:boundary.exports,messages:boundary.messages,deferred:boundary.deferred.map(id => path.basename(id)),
      })),[
        {id:'later.js',exports:[['message']],messages:['later'],deferred:[]},
        {id:'one.js',exports:[['render']],messages:['effect','one'],deferred:['later.js']},
        {id:'two.js',exports:escape ? [['debug'],['default']] : [['default']],
          messages:escape ? ['two','unused'] : ['two'],deferred:[]},
      ]);
    }
  } finally {
    await rm(root,{recursive:true,force:true});
  }
}
