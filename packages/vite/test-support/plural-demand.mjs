import assert from "node:assert/strict";
import {execFile} from "node:child_process";
import {mkdir, mkdtemp, readFile, rm, writeFile} from "node:fs/promises";
import {tmpdir} from "node:os";
import path from "node:path";
import {pathToFileURL} from "node:url";
import {promisify} from "node:util";
import {gzipSync,brotliCompressSync} from "node:zlib";

const run = promisify(execFile);

/** Exercise native metadata, prebuilt artifacts, selected classifiers, and physical references. */
export async function verifyPluralDemand({build, linguini, validatePluralRuntimes, compiler}) {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-plural-pages-"));
  try {
    await mkdir(path.join(root,"schema"));
    await mkdir(path.join(root,"locales/main"),{recursive:true});
    await mkdir(path.join(root,"src"));
    await writeFile(path.join(root,"package.json"),'{"type":"module"}');
    await writeFile(path.join(root,"linguini.toml"),`
[project]
name = "plural-pages"
default_locale = "ru"
locales = ["ru","ru-RU","fr"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "generated"
[targets.ts.bundler]
sources = ["src"]
locale_loading = "dynamic"
`);
    await writeFile(path.join(root,"schema/main.lgs"),`
enum Fruit { apple }
enum Callable { entry }
one(value: Number)
few(value: Number)
category(value: Number)
named(fruit: Fruit, value: Number)
direct(fruit: Callable, value: Number)
other(value: Number)
number(value: Number)
literal
unused
`);
    await writeFile(path.join(root,"locales/main/ru.lgl"),`
fn One(Plural) {
 one => ONE_SELECTED
 _ => ONE_FALLBACK
}
fn Few(Plural) {
 few => FEW_SELECTED
 _ => FEW_FALLBACK
}
fn Other(Plural) {
 one => OTHER_ONE
 other => EXPLICIT_OTHER
 _ => EXPLICIT_DEFAULT
}
impl Fruit { apple {
 form label(Plural) {
  one => APPLE_ONE
  _ => APPLE_FALLBACK
 }
 unused = UNUSED_ATTRIBUTE
} }
impl Callable { entry {
 one => DIRECT_ONE
 _ => DIRECT_FALLBACK
} }
one = {One(value)}
few = {Few(value)}
category = {Plural(value)}
named = {fruit.label(value)}
direct = {fruit(value)}
other = {Other(value)}
number = {value @number}
literal = STATIC_RU
unused = UNUSED_MESSAGE
`);
    await writeFile(path.join(root,"locales/main/ru-RU.lgl"),"literal = STATIC_REGIONAL\n");
    await writeFile(path.join(root,"locales/main/fr.lgl"),"literal = STATIC_FR\n");
    const selections = [
      ["one",["one","number"]],
      ["mixed",["one","few","named","direct"]],
      ["public",["one","category"]],
      ["other",["other"]],
      ["literal",["literal"]],
    ];
    const virtual = name => "virtual:linguini/message/" + Buffer.from("main." + name).toString("hex");
    const pages = [];
    for (const [page,names] of selections) {
      const entry = path.join(root,"src",page + ".js");
      let source = names.map((name,index) => `export {message as render${index}} from ${JSON.stringify(virtual(name))};`).join("\n");
      if (page === "one") source += '\nexport async function later() { const {render} = await import("./later.js"); return render; }';
      await writeFile(entry,source);
      pages.push({id:"/" + page,entries:[entry]});
    }
    await writeFile(path.join(root,"src/later.js"),`export {message as render} from ${JSON.stringify(virtual("few"))};`);
    await run(compiler,["build"],{cwd:root});
    const raw = JSON.parse(await readFile(path.join(root,"generated/bundler/manifest.json"),"utf8"));
    const resolvePath = value => path.resolve(root,"generated",value);
    const messages = new Map(Object.entries(raw.messages).map(([name,message]) => [name,{
      ...message,locales:new Map(Object.entries(message.locales).map(([locale,leaf]) => [locale,{...leaf,module:resolvePath(leaf.module)}])),
    }]));
    const runtimes = new Map(Object.entries(raw.message_runtimes).map(([locale,runtime]) => [locale,{...runtime,module:resolvePath(runtime.module)}]));
    const pluralRuntimes = validatePluralRuntimes(raw.plural_runtimes,{messages,runtimes,resolvePath,context:"native"});
    assert.ok(pluralRuntimes.has("ru"));
    assert.ok(pluralRuntimes.has("ru-RU"));
    assert.equal(pluralRuntimes.get("ru").messages.get("main.category"),null);
    assert.deepEqual(pluralRuntimes.get("ru").messages.get("main.one"),["one"]);
    const names = [...messages.keys()].sort();
    const reference = new Map();
    for (const locale of raw.effective_locales) {
      reference.set(locale,new Map(await Promise.all(names.map(async name => {
        const leaf = messages.get(name).locales.get(locale) ?? messages.get(name).locales.get(raw.base_locale);
        return [name,(await import(pathToFileURL(leaf.module).href)).message];
      }))));
    }
    const costs = [];
    const scenarios = [false,true,"hidden","inline"].map(sourcemap => ({sourcemap,minify:false,legacy:false}));
    scenarios.push({sourcemap:false,minify:true,legacy:false},{sourcemap:false,minify:true,legacy:true});
    for (const {sourcemap,minify,legacy} of scenarios) {
      if (legacy) {
        const previous = {...raw};
        delete previous.plural_runtimes;
        await writeFile(path.join(root,"generated/bundler/manifest.json"),JSON.stringify(previous));
      }
      let receipt;
      const integration = linguini({root,buildOnStart:false});
      const result = await build({
        root,configFile:false,logLevel:"silent",
        plugins:[{
          name:"plural-pages",
          enforce:"pre",
          resolveId(id) { if (id.startsWith("virtual:")) return "\0" + id; },
          load(id) { if (id.startsWith("\0virtual:")) return "export function message() { return globalThis.unknown(); }"; },
          async generateBundle() {
            receipt = await integration.api.compilePageBundles(this,pages);
          },
        },integration],
        build:{write:false,target:"es2022",minify,sourcemap,assetsDir:"assets",rollupOptions:{input:pages.flatMap(page => page.entries),preserveEntrySignatures:"strict"}},
      });
      const assets = new Map(result.output.filter(item => item.type === "asset").map(item => [item.fileName,item.source]));
      const verify = async (ref,locale,page) => {
        const source = String(assets.get(ref.file));
        const code = source.replace(/\n?\/\/# sourceMappingURL=[^\n]*\n?$/,"\n");
        // Vite 8 preserves JSDoc import types. The payload compiler checks executable
        // imports through its parser; executing this self-contained data URL checks linking.
        assert.doesNotMatch(code,/UNUSED_ATTRIBUTE|UNUSED_MESSAGE/);
        if (!legacy && !minify && page === "one" && locale.startsWith("ru")) {
          assert.doesNotMatch(code,/FEW_SELECTED|APPLE_ONE|DIRECT_ONE|EXPLICIT_OTHER|return "few"|return "many"/);
          assert.match(code,/ONE_SELECTED/);
          assert.match(code,/parseGeneratedDecimal/,"formatter exports must retain their original owner");
        }
        if (!legacy && !minify && page === "mixed" && locale.startsWith("ru")) {
          assert.equal((code.match(/function pluralRu(?:Ru)?\(/g) ?? []).length,1,"mixed selectors share one classifier");
          assert.doesNotMatch(code,/EXPLICIT_OTHER|return "many"/);
        }
        if (page === "literal") assert.doesNotMatch(code,/8192|BigInt|plural|parseGeneratedDecimal/);
        if (minify) costs.push({page,locale,legacy,bytes:Buffer.byteLength(code),gzip:gzipSync(code).length,brotli:brotliCompressSync(code).length});
        if (sourcemap) {
          const encoded = /sourceMappingURL=data:application\/json;base64,([^\n]+)/.exec(source)?.[1];
          const map = sourcemap === "inline" ? JSON.parse(Buffer.from(encoded,"base64")) : JSON.parse(assets.get(ref.file + ".map"));
          for (const [index,original] of map.sources.entries()) {
            if (original.includes("\0")) continue;
            const file = path.resolve(root,"dist",path.dirname(ref.file),original).split("?")[0];
            assert.equal(await readFile(file,"utf8"),map.sourcesContent[index]);
          }
        }
        const output = await import("data:text/javascript;base64," + Buffer.from(code).toString("base64"));
        assert.deepEqual(Object.keys(output.messages).sort(),ref.messages.map(name => String(names.indexOf(name))).sort());
        for (const name of ref.messages) {
          const fn = output.messages[names.indexOf(name)];
          const ordinary = reference.get(locale).get(name);
          const invoke = (fn,value) => name === "main.named" ? fn("apple",value) : name === "main.direct" ? fn("entry",value) : fn(value);
          const observe = (fn,value) => {
            try { return ["value",invoke(fn,value)]; }
            catch(error) { return [error.constructor.name,error.message]; }
          };
          for (const value of [0,1,2,5,21,1.5,"1.0","1.2c6",123456789012345678901n,"zero","one","two","few","many","other","invalid",NaN,Infinity]) {
            assert.deepEqual(observe(fn,value),observe(ordinary,value),page + "/" + locale + "/" + name);
          }
          const trace = fn => {
            const hints = [];
            const result = observe(fn,{[Symbol.toPrimitive](hint) {hints.push(hint);return "21";}});
            return {result,hints};
          };
          assert.deepEqual(trace(fn),trace(ordinary),"coercion hints and counts must agree");
          if (name !== "main.literal") {
            const named = name === "main.named" ? {fruit:"apple",value:"21"} :
              name === "main.direct" ? {fruit:"entry",value:"21"} : {value:"21"};
            assert.equal(fn(named),ordinary(named),"named argument contracts must agree");
            const failure = new Error("caller coercion");
            assert.throws(() => invoke(fn,{toString() {throw failure;}}),error => error === failure);
          }
        }
      };
      for (const [page] of selections) {
        for (const locale of raw.effective_locales) await verify(receipt.pages["/" + page][locale],locale,page);
      }
      assert.equal(Object.keys(receipt.deferred).length,1);
      for (const boundary of Object.values(receipt.deferred)) {
        for (const locale of raw.effective_locales) {
          assert.deepEqual(boundary["/one"][locale].messages,["main.few"]);
          await verify(boundary["/one"][locale],locale,"deferred");
        }
      }
      const payloads = [...assets.keys()].filter(file => file.endsWith(".js"));
      assert.equal(payloads.length,(selections.length + 1) * raw.effective_locales.length);
    }
    return costs;
  } finally {
    await rm(root,{recursive:true,force:true});
  }
}
