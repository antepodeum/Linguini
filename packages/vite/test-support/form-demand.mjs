import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {mkdir, mkdtemp, readFile, rm, writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {promisify} from 'node:util';

const run = promisify(execFile);

/** The same native form graph must shake correctly on every supported Vite backend. */
export async function verifyFormDemand({build, compilePageBundles, compiler, typecheck}) {
  const root = await mkdtemp(path.join(tmpdir(), 'linguini-form-demand-'));
  try {
    await mkdir(path.join(root, 'schema'));
    await mkdir(path.join(root, 'locales/main'), {recursive:true});
    await mkdir(path.join(root, 'src'));
    await writeFile(path.join(root, 'package.json'), '{"type":"module"}');
    await writeFile(path.join(root, 'schema/main.lgs'), `
enum Fruit { apple, pear }
enum Callable { entry }
enum Eager { entry }
type Item = Fruit
gender(fruit: Item)
nom(fruit: Fruit, count: Number)
copy(fruit: Fruit)
callable(value: Callable, count: Number)
eager(value: Eager)
`);
    const forms = `
let marker = EAGER_VALUE
fn Echo(value: String) { _ => {value} }
impl Fruit {
  apple {
    Gender = neuter
    emoji = UNUSED_EMOJI
    form nom(Plural) { one => apple
      _ => apples }
    form gen(Plural) { _ => UNUSED_GEN }
    form copy(other: Fruit) { _ => {other.Gender} }
  }
  pear {
    Gender = female
    emoji = UNUSED_EMOJI
    form nom(Plural) { one => pear
      _ => pears }
    form gen(Plural) { _ => UNUSED_GEN }
    form copy(other: Fruit) { _ => {other.Gender} }
  }
}
impl Callable { entry {
  one => ONE
  _ => MANY
  label = CALLABLE
} }
impl Eager { entry { label = {Echo(marker)} } }
`;
    await writeFile(path.join(root, 'locales/main/en.lgl'), forms + `
gender = {fruit.Gender}
nom = {fruit.nom(count)}
copy = {fruit.copy(fruit)}
callable = {value(count)}:{value.label}
eager = {value.label}
`);
    await writeFile(path.join(root, 'locales/main/fr.lgl'), forms + 'gender = FR:{fruit.Gender}\n');
    await writeFile(path.join(root, 'linguini.toml'), `
[project]
name = "form-demand"
default_locale = "en"
locales = ["en", "fr"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "generated"
[targets.ts.bundler]
sources = ["src"]
locale_loading = "dynamic"
`);
    const selections = [
      ['gender',['main.gender']], ['nom',['main.nom']], ['copy',['main.copy']],
      ['callable',['main.callable']], ['eager',['main.eager']],
      ['mixed',['main.gender','main.nom']]
    ];
    const pages = [];
    for (const [name,names] of selections) {
      const entry = path.join(root, 'src', name + '.js');
      await writeFile(entry, names.map((name,index) => `import {message as m${index}} from "virtual:linguini/message/${Buffer.from(name).toString('hex')}"; export {m${index}};`).join('\n'));
      pages.push({id:'/' + name,entries:[entry]});
    }
    await run(compiler, ['build'], {cwd:root});
    const first = await readFile(path.join(root, 'generated/bundler/manifest.json'), 'utf8');
    await run(compiler, ['build'], {cwd:root});
    assert.equal(await readFile(path.join(root, 'generated/bundler/manifest.json'), 'utf8'), first);
    const native = JSON.parse(first);
    const names = Object.keys(native.messages).sort();
    assert.deepEqual(names, ['main.callable','main.copy','main.eager','main.gender','main.nom']);
    const messages = new Map(Object.entries(native.messages).map(([name,value]) => [name, {
      locales:new Map(Object.entries(value.locales).map(([locale,leaf]) => [locale, {...leaf,module:path.resolve(root,'generated',leaf.module)}]))
    }]));
    let receipt;
    const result = await build({
      root, configFile:false, logLevel:'silent',
      plugins:[{
        name:'native-form-demand',
        resolveId(id) { if (id.startsWith('virtual:linguini/message/')) return '\0' + id; },
        load(id) { if (id.startsWith('\0virtual:linguini/message/')) return 'export function message() { return globalThis.unknown(); }'; },
        async generateBundle() {
          receipt = await compilePageBundles(this,pages,{
            messages,baseLocale:'en',effectiveLocales:['en','fr']
          },{root,build:{target:'es2022',minify:true,sourcemap:false,outDir:'dist',assetsDir:'assets'}});
        }
      }],
      build:{write:false,rollupOptions:{input:pages.flatMap(page => page.entries),preserveEntrySignatures:'strict'}}
    });
    assert.deepEqual(Object.keys(receipt.deferred), []);
    const assets = result.output.filter(item => item.type === 'asset');
    assert.equal(assets.length, 12, 'one self-contained payload per page and locale');
    for (const [page,selected] of selections) {
      for (const locale of ['en','fr']) {
        const ref = receipt.pages['/' + page][locale];
        const asset = assets.find(item => item.fileName === ref.file);
        assert.ok(asset, JSON.stringify(ref));
        const code = String(asset.source);
        assert.doesNotMatch(code, /UNUSED_EMOJI|UNUSED_GEN|\bimport\b/);
        if (page !== 'eager') assert.doesNotMatch(code, /EAGER_VALUE/);
        if (page !== 'callable') assert.doesNotMatch(code, /CALLABLE|MANY/);
        if (page === 'gender' || page === 'copy') assert.doesNotMatch(code, /apples|pears|8192|BigInt|plural/);
        if (page === 'nom') assert.doesNotMatch(code, /neuter|female|FR:/);
        const output = path.join(root, `${page}-${locale}.js`);
        await writeFile(output, code);
        const payload = await import(pathToFileURL(output).href);
        assert.deepEqual(Object.keys(payload.messages), selected.map(name => String(names.indexOf(name))));
        const message = name => payload.messages[names.indexOf('main.' + name)];
        if (message('gender')) {
          assert.equal(message('gender')('apple'), locale === 'fr' ? 'FR:neuter' : 'neuter');
          assert.equal(message('gender')({fruit:'pear'}), locale === 'fr' ? 'FR:female' : 'female');
        }
        if (message('nom')) assert.equal(message('nom')({fruit:'apple',count:2}), 'apples');
        if (message('copy')) assert.equal(message('copy')('pear'), 'female');
        if (message('callable')) {
          assert.equal(message('callable')('entry',1), 'ONE:CALLABLE');
          assert.equal(message('callable')({value:'entry',count:2}), 'MANY:CALLABLE');
        }
        if (message('eager')) assert.equal(message('eager')('entry'), 'EAGER_VALUE');
      }
    }
    if (typecheck) {
      const physical = names.map((name,index) => `import {message as m${index}} from ${JSON.stringify('./generated/' + native.messages[name].locales.en.module)};`);
      const ordinary = [`import {lgl} from './generated/index.js';`];
      for (const [index,name] of names.entries()) {
        const key = name.slice('main.'.length);
        const args = key === 'callable' ? "'entry',2" : key === 'eager' ? "'entry'" : key === 'nom' ? "'apple',2" : "'apple'";
        const invalid = key === 'callable' || key === 'nom' ? "'missing',2" : "'missing'";
        const object = key === 'callable' ? "{value:'entry',count:2}" : key === 'eager' ? "{value:'entry'}" : key === 'nom' ? "{fruit:'apple',count:2}" : "{fruit:'apple'}";
        for (const [source,call] of [[physical,'m'+index],[ordinary,'lgl.main.'+key]]) {
          source.push(`${call}(${args}); ${call}(${object});\n// @ts-expect-error invalid enum member\n${call}(${invalid});`);
          if (key === 'callable' || key === 'nom') {
            source.push(`// @ts-expect-error invalid plural input\n${call}(${key === 'callable' ? "'entry'" : "'apple'"},false);`);
          }
        }
      }
      for (const [kind,source] of [['physical',physical],['ordinary',ordinary]]) {
        await writeFile(path.join(root,kind+'-consumer.ts'), source.join('\n'));
        const config = path.join(root,kind+'.json');
        await writeFile(config,JSON.stringify({compilerOptions:{strict:true,noEmit:true,target:'ES2022',module:'NodeNext',moduleResolution:'NodeNext',types:[],allowJs:kind==='physical',checkJs:kind==='physical'},files:[kind+'-consumer.ts']}));
        await typecheck(config);
      }
    }
  } finally {
    await rm(root, {recursive:true, force:true});
  }
}
