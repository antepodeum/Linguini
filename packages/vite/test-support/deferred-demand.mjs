import assert from 'node:assert/strict';
import {mkdtemp,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';

/** Run the same dependency-closure contract against the consuming Vite host. */
export async function verifyDeferredDemand({build,pageMessagePlan,compilePageBundles}) {
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
    for (const [escape,target] of [[false,'es2018'],[false,'es2022'],[true,'es2022'],['mixed','es2022']]) {
      let receipt;
      const run = () => build({
        configFile:false,root,logLevel:'silent',
        plugins:[{
          name:'deferred-plan-fixture',
          resolveId(id) {if (id.startsWith('virtual:')) return '\0' + id;},
          load(id) {if (id.startsWith('\0virtual:')) return 'export function message(){ return globalThis.message(); }';},
          async generateBundle() {
            receipt = await pageMessagePlan(this,[{id:path.join(root,'entry.js'),
              exports:(escape === 'mixed' ? ['render','one','two','escape'] : escape ? ['render','one','escape'] : ['render','one','two']).map(name => [name])}],
            {root,build:{target}});
          },
        }],
        build:{write:false,rollupOptions:{input:path.join(root,'entry.js'),preserveEntrySignatures:'strict'}},
      });
      if (escape === 'mixed') {
        await assert.rejects(run,/require different message sets; call-site preparation is not supported yet/);
        continue;
      }
      await run();
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
    await verifyPreparedVariants(root,{build,pageMessagePlan,compilePageBundles});
  } finally {
    await rm(root,{recursive:true,force:true});
  }
}


async function verifyPreparedVariants(root,{build,pageMessagePlan,compilePageBundles}) {
  const virtual=name=>'virtual:linguini/message/'+Buffer.from(name).toString('hex');
  const messages=new Map();
  for (const name of ['a','b','c','initial','unused']) {
    const locales=new Map();
    for (const locale of ['en','fr']) {
      if (name==='b'&&locale==='fr') continue;
      const file=path.join(root,`${name}-${locale}.js`);
      await writeFile(file,`export const message=()=>${JSON.stringify(name==='unused'?'FORBIDDEN_VARIANT':name+':'+locale)};`);
      locales.set(locale,{module:file});
    }
    messages.set(name,{locales});
  }
  const manifest={messages,baseLocale:'en',effectiveLocales:['en','fr']};
  const ids=new Map([...messages.keys()].sort().map((name,index)=>[name,String(index)]));
  for (const extra of [false,true]) for (const coverage of ['parent','partial','direct','cycle']) {
    const entry=path.join(root,'entry.js');
    const parent=path.join(root,'parent.js');
    const child=path.join(root,'child.js');
    const files={
      'entry.js':`export {message as initial} from ${JSON.stringify(virtual('initial'))};
        export async function parent(){const {render,one,two}=await import('./parent.js');return {render,one,two};}
        ${coverage==='direct'?"export async function direct(){const {two}=await import('./child.js');return two();}":''}`,
      'parent.js':`import {message as a} from ${JSON.stringify(virtual('a'))};
        ${coverage!=='partial'?'import {message as b} from '+JSON.stringify(virtual('b'))+';':''}
        export const render=()=>a()${coverage!=='partial'?'+b()':''};
        export async function one(){const {one${coverage==='cycle'?',cycle':''}}=await import('./child.js');return ${coverage==='cycle'?'[one(),cycle]':'one()'};}
        export async function two(){const {two}=await import('./child.js');return two();}`,
      'child.js':`import {message as a} from ${JSON.stringify(virtual('a'))};
        import {message as b} from ${JSON.stringify(virtual('b'))};
        ${extra?'import {message as c} from '+JSON.stringify(virtual('c'))+';':''}
        export const one=()=>a()${extra?'+c()':''}; export const two=()=>b()${extra?'+c()':''};
        export {message as debug} from ${JSON.stringify(virtual('unused'))};
        ${coverage==='cycle'?"export async function cycle(){const {one}=await import('./parent.js');return one();}":''}`,
    };
    for (const [name,source] of Object.entries(files)) await writeFile(path.join(root,name),source);
    let receipt,plan;
    const run=()=>build({
      configFile:false,root,logLevel:'silent',
      plugins:[{
        name:'prepared-variant-fixture',
        resolveId(id){if(id.startsWith('virtual:'))return '\0'+id;},
        load(id){if(id.startsWith('\0virtual:'))return 'export function message(){return globalThis.unknown();}';},
        async generateBundle(){
          const entries=[entry];
          plan=await pageMessagePlan(this,entries,{root,build:{target:'es2022'}});
          receipt=await compilePageBundles(this,[{id:'/',entries}],manifest,
            {root,build:{target:'es2022',minify:true,assetsDir:'assets'}});
        },
      }],
      build:{write:false,rollupOptions:{input:entry,preserveEntrySignatures:'strict'}},
    });
    if (coverage!=='parent') {
      await assert.rejects(run,/require different message sets; call-site preparation is not supported yet/);
      continue;
    }
    const result=await run();
    const planned=plan.deferred.find(boundary=>boundary.id===child);
    assert.deepEqual(planned.available,['a','b','initial']);
    assert.deepEqual(planned.messages,extra?['a','b','c']:['a','b']);
    assert.deepEqual(plan.messages,['initial']);
    const assets=new Map(result.output.filter(item=>item.type==='asset').map(item=>[item.fileName,item.source]));
    assert.equal(assets.size,extra?6:4,'one asset per actual initial/parent/additional closure and locale');
    if (!extra) assert.equal(receipt.deferred[child],undefined,'already prepared variants emit no extra payload');
    for (const locale of ['en','fr']) {
      const payloads=[receipt.pages['/'][locale],receipt.deferred[parent]['/'][locale]];
      assert.deepEqual(payloads.map(payload=>payload.messages),[['initial'],['a','b']]);
      if (extra) {
        const additional=receipt.deferred[child]['/'][locale];
        assert.deepEqual(additional.messages,['c'],'both export variants share only their unprepared demand');
        payloads.push(additional);
      }
      for (const payload of payloads) {
        const code=assets.get(payload.file);
        assert.doesNotMatch(code,/FORBIDDEN_VARIANT|\bimport\b/);
        const module=await import('data:text/javascript;base64,'+Buffer.from(code).toString('base64'));
        assert.deepEqual(Object.keys(module.messages).sort(),payload.messages.map(name=>ids.get(name)).sort());
        for (const name of payload.messages) assert.equal(module.messages[ids.get(name)](),name+':'+(name==='b'?'en':locale));
      }
    }
  }
}
