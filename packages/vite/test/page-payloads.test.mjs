import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import {stripVTControlCharacters} from 'node:util';
import { build } from "vite";
import {verifyDeferredDemand} from '../test-support/deferred-demand.mjs';
import { compilePagePayloads, compilePageBundles, pageMessageUsage, pageMessagePlan } from "../src/page-payloads.js";

for (const sourcemap of [false,true,"hidden","inline"]) {
test(`page payloads shake composed roots, preserve fallback/helpers, and support ${sourcemap} maps`, async context => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-pages-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  const virtual = name => "virtual:linguini/message/" + Buffer.from(name).toString("hex");
  const files = {
    "one.js": 'export { first as render, unused as debug } from "./shared.js"; export { component } from "./child.js";',
    "two.js": 'export { second as render } from "./shared.js";',
    "layout.js": `import { message } from ${JSON.stringify(virtual("layout"))}; export const render = () => message();`,
    "child.js": `import { message } from ${JSON.stringify(virtual("child"))}; export const component = () => message(); export const deferred = () => import("./later.js");`,
    "later.js": `import { message } from ${JSON.stringify(virtual("later"))}; export const render = () => message();`,
    "shared.js": ["first", "second", "unused"].map(name => `import { message as ${name}Message } from ${JSON.stringify(virtual(name))}; export function ${name}() { return ${name}Message(); }`).join("\n"),
    "helpers.js": 'export const suffix = value => value + ":SHARED_HELPER"; export const unused = () => "FORBIDDEN_HELPER";'
  };
  const messages = new Map();
  for (const name of ["first", "second", "layout", "child", "later", "unused"]) {
    const locales = new Map();
    for (const locale of ["en", "fr"]) {
      if (name === "child" && locale === "fr") continue;
      const file = path.join(root, `${name}-${locale}.js`);
      files[path.basename(file)] = `import { suffix } from "./helpers.js"; export const message = () => suffix(${JSON.stringify(name + ":" + locale)});`;
      locales.set(locale, { module: file });
    }
    messages.set(name, { locales });
  }
  await mkdir(root, { recursive: true });
  for (const [name, source] of Object.entries(files)) await writeFile(path.join(root, name), source);
  let receipt;
  const result = await build({
    configFile: false, root, logLevel: "silent",
    plugins: [{
      name: "page-fixture",
      resolveId(id) { if (id.startsWith("virtual:")) return "\0" + id; },
      load(id) { if (id.startsWith("\0virtual:")) return 'export function message() { return globalThis.unknown(); }'; },
      async generateBundle() {
        receipt = await compilePagePayloads(this, [
          { id: "/one", entries: [path.join(root, "layout.js"), {id:path.join(root, "one.js"),exports:[["render"],["component"]]}] },
          { id: "/two", entries: [path.join(root, "layout.js"), path.join(root, "two.js")] }
        ], { messages, baseLocale: "en", effectiveLocales: ["en", "fr"] }, {
          root, build: { target: "es2022", minify: true, sourcemap, outDir: "dist", assetsDir: "assets" }
        });
      }
    }],
    build: { write: false, rollupOptions: {
      input: ["one.js", "two.js", "layout.js"].map(name => path.join(root, name)),
      preserveEntrySignatures: "strict"
    } }
  });
  const assets = new Map(result.output.filter(item => item.type === "asset").map(item => [item.fileName, item.source]));
  for (const [page, expected] of [["/one", ["child", "first", "layout"]], ["/two", ["layout", "second"]]]) {
    for (const locale of ["en", "fr"]) {
      const payload = receipt[page][locale];
      assert.deepEqual(payload.messages, expected);
      const source = assets.get(payload.file);
      assert.equal(typeof source, "string");
      const executable = source.replace(/\n?\/\/# sourceMappingURL=[^\n]*\n?$/, "\n");
      assert.doesNotMatch(executable, /FORBIDDEN_HELPER|unused|later|\bimport\b/);
      if (sourcemap) {
        let map;
        if (sourcemap === 'inline') {
          const encoded = /sourceMappingURL=data:application\/json;base64,([^\n]+)/.exec(source)?.[1];
          assert.ok(encoded);
          map = JSON.parse(Buffer.from(encoded,'base64').toString('utf8'));
          assert.equal(assets.has(payload.file + '.map'),false);
        } else {
          map = JSON.parse(assets.get(payload.file + '.map'));
          if (sourcemap === 'hidden') assert.doesNotMatch(source,/sourceMappingURL/);
          else assert.match(source,new RegExp('sourceMappingURL=' + locale + '\\.js\\.map'));
        }
        assert.equal(map.file,locale + '.js');
        assert.ok(map.sources.length > 1);
        for (const [index,original] of map.sources.entries()) {
          if (original.includes('\0')) continue;
          const resolved = path.resolve(root,'dist',path.dirname(payload.file),original);
          assert.equal(await readFile(resolved,'utf8'),map.sourcesContent[index]);
        }
      } else {
        assert.doesNotMatch(source,/sourceMappingURL/);
      }
      const module = await import("data:text/javascript;base64," + Buffer.from(source).toString("base64"));
      const ids = {child:0,first:1,later:2,layout:3,second:4,unused:5};
      assert.deepEqual(Object.keys(module.messages).sort(), expected.map(name => String(ids[name])).sort());
      for (const name of expected) assert.equal(module.messages[ids[name]](), `${name}:${name === "child" ? "en" : locale}:SHARED_HELPER`);
      assert.equal((source.match(/SHARED_HELPER/g) ?? []).length, 1);
    }
  }
  assert.equal(assets.size, sourcemap && sourcemap !== 'inline' ? 8 : 4);
});

}

test('deferred boundaries cannot silently omit new messages or widen the initial payload', async context => {
  const root = await mkdtemp(path.join(tmpdir(), 'linguini-deferred-'));
  context.after(() => rm(root, {recursive:true,force:true}));
  const virtual = name => 'virtual:linguini/message/' + Buffer.from(name).toString('hex');
  await writeFile(path.join(root,'first.js'), 'export const message = "FIRST";');
  await writeFile(path.join(root,'later.js'), 'export const message = "LATER";');
  await writeFile(path.join(root,'unused.js'), 'export const message = "UNUSED_DEFERRED";');
  await writeFile(path.join(root,'deferred.js'), `export {message as render} from ${JSON.stringify(virtual('later'))}; export {message as unused} from ${JSON.stringify(virtual('unused'))};`);
  const messages = new Map(['first','later','unused'].map(name => [name,{locales:new Map([
    ['en',{module:path.join(root,name + '.js')}]]
  )}]));
  for (const sameMessage of [false,true]) {
    await writeFile(path.join(root,'entry.js'), `export {message as render} from ${JSON.stringify(virtual(sameMessage ? 'later' : 'first'))}; export async function action() { const {render} = await import('./deferred.js'); return render; }`);
    let receipt;
    const run = (prepareDeferred = false) => build({
      configFile:false,root,logLevel:'silent',
      plugins:[{
        name:'deferred-fixture',
        resolveId(id) {if (id.startsWith('virtual:')) return '\0' + id;},
        load(id) {if (id.startsWith('\0virtual:')) return 'export const message = globalThis.unknown;';},
        async generateBundle() {
          const pages = (prepareDeferred ? ['/','/copy'] : ['/']).map(id => ({id,entries:[path.join(root,'entry.js')]}));
          receipt = await (prepareDeferred ? compilePageBundles : compilePagePayloads)(this,pages,
            {messages,baseLocale:'en',effectiveLocales:['en']},
            {root,build:{target:'es2022',minify:true,assetsDir:'assets'}});
        },
      }],
      build:{write:false,rollupOptions:{input:path.join(root,'entry.js'),preserveEntrySignatures:'strict'}},
    });
    if (sameMessage) {
      await run();
      assert.deepEqual(receipt['/'].en.messages,['later']);
    } else {
      await assert.rejects(run,error => {
        assert.match(stripVTControlCharacters(error.message),/cannot yet prepare deferred messages later for page \/$/m);
        return true;
      });
    }
    const output = await run(true);
    assert.deepEqual(receipt.pages['/'].en.messages,[sameMessage ? 'later' : 'first']);
    assert.equal(receipt.pages['/'].en.file,receipt.pages['/copy'].en.file);
    const boundaries = Object.values(receipt.deferred);
    assert.equal(boundaries.length,sameMessage ? 0 : 1);
    if (!sameMessage) {
      assert.deepEqual(boundaries[0]['/'].en.messages,['later']);
      assert.equal(boundaries[0]['/'].en.file,boundaries[0]['/copy'].en.file);
    }
    const assets = output.output.filter(item => item.type === 'asset');
    assert.equal(assets.length,sameMessage ? 1 : 2,'identical closures must share one compiled artifact');
    const values = [];
    for (const asset of assets) {
      assert.doesNotMatch(asset.source,/UNUSED_DEFERRED/);
      const module = await import('data:text/javascript;base64,' + Buffer.from(asset.source).toString('base64'));
      values.push(...Object.values(module.messages));
    }
    assert.deepEqual(values.sort(),sameMessage ? ['LATER'] : ['FIRST','LATER']);
  }
});

test('page analysis preserves the host module side-effect decisions', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-page-effects-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  await writeFile(path.join(root,'entry.js'),'import "./effect.js"; export const render = () => "PAGE";');
  await writeFile(path.join(root,'effect.js'),'import {message} from "virtual:linguini/message/656666656374"; globalThis.effect = message();');
  for (const moduleSideEffects of [false,true]) {
    let receipt;
    await build({
      configFile:false,root,logLevel:'silent',
      plugins:[{
        name:'host-side-effects',
        resolveId(id) {if (id.startsWith('virtual:')) return '\0' + id;},
        load(id) {
          if (id.startsWith('\0virtual:')) return 'export function message() { return globalThis.message(); }';
          if (id === path.join(root,'effect.js')) return {code:'import {message} from "virtual:linguini/message/656666656374"; globalThis.effect = message();',moduleSideEffects};
        },
        async generateBundle() {
          receipt = await pageMessageUsage(this,[path.join(root,'entry.js')],{root,build:{target:'es2022'}});
        },
      }],
      build:{write:false,rollupOptions:{input:path.join(root,'entry.js'),preserveEntrySignatures:'strict'}},
    });
    assert.deepEqual(receipt,{messages:moduleSideEffects ? ['effect'] : [],deferred:[]});
  }
});

test('deferred plans retain caller-selected exports across shared eager/lazy modules and pure re-exports', async () => {
  await verifyDeferredDemand({build,pageMessagePlan});
});

test('nested payloads exclude guaranteed parent messages but retain them for a direct import path', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-nested-payloads-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const virtual = name => 'virtual:linguini/message/' + Buffer.from(name).toString('hex');
  const files = {
    'entry.js':`export {message as render} from ${JSON.stringify(virtual('initial'))};
      export function parent(){return import('./parent.js');}
      export async function direct(){const {render} = await import('./child.js');return render();}`,
    'parent.js':`import {message} from ${JSON.stringify(virtual('parent'))};
      export const render = () => message();
      export async function next(){const {next} = await import('./transit.js');return next();}`,
    'transit.js':`export async function next(){const {render} = await import('./child.js');return render();}`,
    'child.js':['parent','child'].map(name => `import {message as ${name}} from ${JSON.stringify(virtual(name))};`).join('\n')
      + 'export const render = () => parent() + child();',
  };
  const messages = new Map();
  for (const name of ['initial','parent','child','unused']) {
    const locales = new Map();
    for (const locale of ['en','fr']) {
      const file = path.join(root,name + '-' + locale + '.js');
      files[path.basename(file)] = `export const message = ${JSON.stringify(name.toUpperCase() + '_' + locale)};`;
      locales.set(locale,{module:file});
    }
    messages.set(name,{locales});
  }
  for (const [name,source] of Object.entries(files)) await writeFile(path.join(root,name),source);
  let receipt;
  const output = await build({
    configFile:false,root,logLevel:'silent',
    plugins:[{
      name:'nested-payload-fixture',
      resolveId(id) {if (id.startsWith('virtual:')) return '\0' + id;},
      load(id) {if (id.startsWith('\0virtual:')) return 'export function message(){return globalThis.unknown();}';},
      async generateBundle() {
        receipt = await compilePageBundles(this,[
          {id:'/nested',entries:[{id:path.join(root,'entry.js'),exports:[['render'],['parent']]}]},
          {id:'/direct',entries:[path.join(root,'entry.js')]},
        ],{messages,baseLocale:'en',effectiveLocales:['en','fr']},
        {root,build:{target:'es2022',minify:true,assetsDir:'assets'}});
      },
    }],
    build:{write:false,rollupOptions:{input:path.join(root,'entry.js'),preserveEntrySignatures:'strict'}},
  });
  const parent = receipt.deferred[path.join(root,'parent.js')];
  const child = receipt.deferred[path.join(root,'child.js')];
  assert.equal(receipt.deferred[path.join(root,'transit.js')],undefined,'transit-only imports must not emit empty payloads');
  for (const route of ['/nested','/direct']) {
    assert.deepEqual(receipt.deferredImports[route].map(id => path.basename(id)),['child.js','parent.js','transit.js']);
  }
  const assets = new Map(output.output.filter(item => item.type === 'asset').map(item => [item.fileName,item.source]));
  assert.equal(assets.size,8,'four distinct closures across two locales, without transit payloads');
  const ids = {child:0,initial:1,parent:2};
  for (const locale of ['en','fr']) {
    assert.equal(receipt.pages['/nested'][locale].file,receipt.pages['/direct'][locale].file);
    assert.equal(parent['/nested'][locale].file,parent['/direct'][locale].file);
    for (const [payload,names] of [[receipt.pages['/nested'][locale],['initial']],
      [parent['/nested'][locale],['parent']],[child['/nested'][locale],['child']],
      [child['/direct'][locale],['child','parent']]]) {
      assert.deepEqual(payload.messages,names);
      const source = assets.get(payload.file);
      assert.doesNotMatch(source,/UNUSED|\bimport\b/);
      const module = await import('data:text/javascript;base64,' + Buffer.from(source).toString('base64'));
      assert.deepEqual(Object.keys(module.messages).sort(),names.map(name => String(ids[name])).sort());
      for (const name of names) assert.equal(module.messages[ids[name]],name.toUpperCase() + '_' + locale);
    }
  }
});
