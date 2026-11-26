import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {mkdir, mkdtemp, readFile, rm, symlink, writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {before, test} from 'node:test';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {promisify} from 'node:util';
import {build} from 'vite';
import {compilePageBundles} from '../src/page-payloads.js';
import {buildNativeCompiler} from '../../../tests/support/native-vite.mjs';

const run = promisify(execFile);
const packageRoot = fileURLToPath(new URL('../', import.meta.url));
const compiler = fileURLToPath(new URL(`../../../target/debug/linguini${process.platform === 'win32' ? '.exe' : ''}`, import.meta.url));
before(buildNativeCompiler);

test('native form demand omits unused attributes and helpers from complete locale payloads', async context => {
  const root = await mkdtemp(path.join(tmpdir(), 'linguini-form-demand-'));
  context.after(() => rm(root, {recursive:true, force:true}));
  await symlink(path.join(packageRoot, 'node_modules'), path.join(root, 'node_modules'), 'junction');
  await mkdir(path.join(root, 'schema'));
  await mkdir(path.join(root, 'locales/main'), {recursive:true});
  await mkdir(path.join(root, 'src'));
  await writeFile(path.join(root, 'package.json'), '{"type":"module"}');
  const forms = `
impl Fruit {
  apple {
    Gender = neuter
    emoji = UNUSED_EMOJI
    form nom(Plural) { one => apple
      _ => apples }
    form gen(Plural) { _ => UNUSED_GEN }
  }
  pear {
    Gender = female
    emoji = UNUSED_EMOJI
    form nom(Plural) { one => pear
      _ => pears }
    form gen(Plural) { _ => UNUSED_GEN }
  }
}
`;
  await writeFile(path.join(root, 'locales/main/fr.lgl'), forms + 'gender = FR:{fruit.Gender}\n');
  for (const mixed of [false,true]) {
    const names = mixed ? ['main.gender','main.nom'] : ['main.gender'];
    await writeFile(path.join(root, 'schema/main.lgs'), 'enum Fruit { apple, pear }\ntype Item = Fruit\ngender(fruit: Item)\n' + (mixed ? 'nom(fruit: Fruit, count: Number)\n' : ''));
    await writeFile(path.join(root, 'locales/main/en.lgl'), forms + 'gender = {fruit.Gender}\n' + (mixed ? 'nom = {fruit.nom(count)}\n' : ''));
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
    const entry = path.join(root, 'src/page.js');
    await writeFile(entry, names.map((name, index) => `import {message as m${index}} from "virtual:linguini/message/${Buffer.from(name).toString('hex')}"; export {m${index}};`).join('\n'));
    await run(compiler, ['build'], {cwd:root});
    const first = await readFile(path.join(root, 'generated/bundler/manifest.json'), 'utf8');
    await run(compiler, ['build'], {cwd:root});
    assert.equal(await readFile(path.join(root, 'generated/bundler/manifest.json'), 'utf8'), first);
    const native = JSON.parse(first);
    const messages = new Map(Object.entries(native.messages).map(([name, value]) => [name, {
      locales:new Map(Object.entries(value.locales).map(([locale, leaf]) => [locale, {...leaf,module:path.resolve(root,'generated',leaf.module)}]))
    }]));
    assert.deepEqual([...messages.keys()].sort(), names);
    let receipt;
    const result = await build({
      root, configFile:false, logLevel:'silent',
      plugins:[{
        name:'native-form-demand',
        resolveId(id) { if (id.startsWith('virtual:linguini/message/')) return '\0' + id; },
        load(id) { if (id.startsWith('\0virtual:linguini/message/')) return 'export function message() { return globalThis.unknown(); }'; },
        async generateBundle() {
          receipt = await compilePageBundles(this,[{id:'/page',entries:[entry]}],{
            messages,baseLocale:'en',effectiveLocales:['en','fr']
          },{root,build:{target:'es2022',minify:true,sourcemap:false,outDir:'dist',assetsDir:'assets'}});
        }
      }],
      build:{write:false,rollupOptions:{input:entry,preserveEntrySignatures:'strict'}}
    });
    assert.deepEqual(Object.keys(receipt.deferred), []);
    const assets = result.output.filter(item => item.type === 'asset');
    assert.equal(assets.length, 2, 'one self-contained payload per locale');
    for (const locale of ['en','fr']) {
      const ref = receipt.pages['/page'][locale];
      const asset = assets.find(item => item.fileName === ref.file);
      assert.ok(asset, JSON.stringify(ref));
      const code = String(asset.source);
      assert.doesNotMatch(code, /UNUSED_EMOJI|UNUSED_GEN|import\(/);
      if (!mixed) assert.doesNotMatch(code, /apples|pears|8192|BigInt|plural/);
      const output = path.join(root, `${mixed}-${locale}.js`);
      await writeFile(output, code);
      const payload = await import(pathToFileURL(output).href);
      assert.deepEqual(Object.keys(payload.messages), names.map((_,index) => String(index)));
      assert.equal(payload.messages[0]('apple'), locale === 'fr' ? 'FR:neuter' : 'neuter');
      assert.equal(payload.messages[0]({fruit:'pear'}), locale === 'fr' ? 'FR:female' : 'female');
      if (mixed) assert.equal(payload.messages[1]({fruit:'apple',count:2}), 'apples');
    }
  }
});
