import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import test from 'node:test';
import {build} from 'vite';
import {clientEntry} from '../dist/build/client-entry.js';
import {clientNodes,SHARED_RUNTIME} from '../dist/build/client-nodes.js';

test('client capabilities stay in their owning page and share one runtime only when several pages need it', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-kit-node-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const directory = path.join(root,'generated/client-optimized');
  await mkdir(path.join(directory,'nodes'),{recursive:true});
  await writeFile(path.join(root,'package.json'),'{"type":"module"}');
  const application = path.join(directory,'app.js');
  await writeFile(application,'export const nodes = [() => import("./nodes/0.js"),() => import("./nodes/1.js")];');
  for (const index of [0,1]) {
    await writeFile(path.join(directory,`nodes/${index}.js`),`export {component} from '../page${index}.js';`);
    await writeFile(path.join(directory,`page${index}.js`),`export const component = () => 'PAGE${index}';`);
  }
  const runtime = 'export function optionalCapability(){return "CAPABILITY";}';
  for (const count of [0,1,2]) {
    let release;
    const ready = new Promise(resolve => {release=resolve;});
    const result = await build({
      configFile:false,root,logLevel:'silent',
      plugins:[clientEntry(async context => {
        for (const index of [0,1]) assert.ok(context.getModuleInfo(path.join(directory,`nodes/${index}.js`)).exports.includes('component'));
        if (count === 2) context.emitFile({type:'chunk',id:SHARED_RUNTIME,name:'shared-capability',preserveSignature:'strict'});
        release();
        return 'export {};';
      }),clientNodes(async (_context,owner) => {
        await ready;
        if (count === 2) return `import ${JSON.stringify(SHARED_RUNTIME)};`;
        return count === 1 && owner.endsWith('/0.js') ? runtime : 'export {};';
      },() => runtime)],
      build:{write:true,minify:true,manifest:true,sourcemap:true,
        rollupOptions:{input:{app:application,...Object.fromEntries([0,1].map(index => ['node' + index,path.join(directory,`nodes/${index}.js`)]))},
          preserveEntrySignatures:'strict'}},
    });
    const chunks = result.output.filter(item => item.type === 'chunk');
    assert.equal(chunks.length,count === 2 ? 4 : 3);
    const manifest = JSON.parse(result.output.find(item => item.fileName === '.vite/manifest.json').source);
    const app = manifest['generated/client-optimized/app.js'];
    assert.ok(app?.isEntry);
    assert.doesNotMatch(chunks.find(chunk => chunk.fileName === app.file).code,/CAPABILITY/);
    for (const index of [0,1]) {
      const node = manifest[`generated/client-optimized/nodes/${index}.js`];
      assert.ok(node?.isEntry);
      const code = chunks.find(chunk => chunk.fileName === node.file).code;
      if (count !== 1 || index !== 0) assert.doesNotMatch(code,/CAPABILITY/);
    }
    const module = await import(pathToFileURL(path.join(root,'dist',app.file)).href);
    const one = await module.nodes[0]();
    const two = await module.nodes[1]();
    assert.equal(one.component(),'PAGE0');
    assert.equal(two.component(),'PAGE1');
    assert.equal(Object.hasOwn(one,'optionalCapability'),count === 1);
    assert.equal(Object.hasOwn(two,'optionalCapability'),false);
    if (count === 1) assert.equal(one.optionalCapability(),'CAPABILITY');
    assert.equal(chunks.filter(chunk => chunk.code.includes('CAPABILITY')).length,count ? 1 : 0);
  }
});
