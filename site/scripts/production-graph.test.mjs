import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';

const siteRoot = join(fileURLToPath(new URL('..', import.meta.url)));
const buildRoot = join(siteRoot, 'build');
const generatedRoot = join(siteRoot, 'src/lib/generated/linguini');
const clientOutputRoot = join(siteRoot, '.svelte-kit/output/client');
const serverOutputRoot = join(siteRoot, '.svelte-kit/output/server');

function filesUnder(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...filesUnder(path));
    else files.push(path);
  }
  return files;
}

function readJson(path, label) {
  assert.equal(statSync(path).isFile(), true, `${label} is missing`);
  return JSON.parse(readFileSync(path, 'utf8'));
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function countMatches(source, pattern) {
  return [...source.matchAll(pattern)].length;
}

function siteRelative(path) {
  return path.slice(siteRoot.length).replace(/^[/\\]/, '').replaceAll('\\', '/');
}

assert.equal(statSync(buildRoot).isDirectory(), true, 'site/build is missing; run pnpm build:generated first');

const outputFiles = filesUnder(buildRoot);
const textFiles = outputFiles
  .filter((file) => /\.(?:html|js|json|css)$/.test(file))
  .map((file) => readFileSync(file, 'utf8'));
const output = textFiles.join('\n');
const english = readFileSync(join(buildRoot, 'en/index.html'), 'utf8');

const generatedManifest = readJson(
  join(generatedRoot, 'bundler/manifest.json'),
  'generated bundler manifest'
);
assert.equal(generatedManifest.version, 1);
assert.equal(generatedManifest.locale_loading, 'dynamic');

const effectiveLocales = new Set(generatedManifest.effective_locales);
assert.deepEqual(
  new Set(Object.keys(generatedManifest.message_runtimes)),
  effectiveLocales,
  'message runtimes do not exactly cover effective locales'
);
const sourceIds = new Set(generatedManifest.sources.map((source) => source.id));
const runtimeModuleIds = new Set();
for (const [locale, runtime] of Object.entries(generatedManifest.message_runtimes)) {
  assert.ok(Array.isArray(runtime.source_ids), `${locale} runtime source ids are missing`);
  assert.ok(
    runtime.source_ids.every((sourceId) => sourceIds.has(sourceId)),
    `${locale} runtime references an unknown source id`
  );
  const moduleId = `src/lib/generated/linguini/${runtime.module}`;
  assert.equal(runtimeModuleIds.has(moduleId), false, `duplicate runtime module ${moduleId}`);
  runtimeModuleIds.add(moduleId);
  assert.equal(
    statSync(join(siteRoot, moduleId)).isFile(),
    true,
    `${locale} runtime module is missing`
  );
}

assert.ok(
  Array.isArray(generatedManifest.message_semantics),
  'message semantics are missing'
);
const semanticKeys = new Set();
const semanticModuleIds = new Set();
const semanticBindings = new Set();
for (const semantic of generatedManifest.message_semantics) {
  const key = `${semantic.locale}\0${semantic.kind}\0${semantic.name}`;
  assert.equal(semanticKeys.has(key), false, `duplicate semantic descriptor ${key}`);
  semanticKeys.add(key);
  assert.ok(effectiveLocales.has(semantic.locale), `${key} has an unknown locale`);
  assert.ok(
    ['enum', 'variable', 'form', 'function'].includes(semantic.kind),
    `${key} has an unknown kind`
  );
  assert.ok(
    semantic.source_ids.every((sourceId) => sourceIds.has(sourceId)),
    `${key} references an unknown source id`
  );
  const moduleId = `src/lib/generated/linguini/${semantic.module}`;
  assert.equal(semanticModuleIds.has(moduleId), false, `duplicate semantic module ${moduleId}`);
  semanticModuleIds.add(moduleId);
  const modulePath = join(siteRoot, moduleId);
  assert.equal(statSync(modulePath).isFile(), true, `${key} semantic module is missing`);
  assert.equal(statSync(`${modulePath}.map`).isFile(), true, `${key} semantic map is missing`);
  const source = readFileSync(modulePath, 'utf8');
  const exportedBinding = semantic.kind === 'enum'
    ? /@typedef \{[^\n]+\} ([A-Za-z_$][\w$]*) \*\//.exec(source)?.[1]
    : /export (?:const|function) ([A-Za-z_$][\w$]*)/.exec(source)?.[1];
  assert.ok(exportedBinding, `${key} has no exported semantic contract`);
  if (semantic.kind === 'enum') {
    assert.match(source, /export \{\};/, `${key} must remain a type-only module`);
    assert.doesNotMatch(source, /export (?:const|function)/, `${key} emitted an enum runtime`);
  }
  semanticBindings.add(exportedBinding);
}
assert.ok(semanticModuleIds.size > 0, 'no shared semantic modules were generated');

// Static application references must stay exact. Locale loading is a separate
// concern and must not turn an exact message access into an unresolved or
// analyzer-dynamic reference.
const usedMessages = new Set();
let referenceCount = 0;
for (const [applicationId, application] of Object.entries(generatedManifest.applications)) {
  assert.deepEqual(application.unresolved, [], `${applicationId} has unresolved references`);
  assert.deepEqual(
    application.dynamic_references,
    [],
    `${applicationId} has dynamic message references`
  );
  const bindings = new Set(application.imports.map((binding) => binding.binding_id));
  for (const reference of application.references) {
    referenceCount += 1;
    usedMessages.add(reference.message);
    assert.ok(bindings.has(reference.binding_id), `${applicationId} reference binding is not exact`);
    assert.equal(reference.provenance?.kind, 'imported');
    assert.equal(reference.provenance?.module_specifier, '$lib/generated/linguini/svelte');
    assert.ok(Number.isInteger(reference.start) && Number.isInteger(reference.end));
    assert.ok(reference.start < reference.end, `${applicationId} has an invalid reference span`);
  }
}
assert.ok(referenceCount > 0, 'generated manifest has no application references');

const chunkLabApplicationId = 'src/routes/chunk-lab/+page.svelte';
const chunkLabApplication = generatedManifest.applications[chunkLabApplicationId];
assert.ok(chunkLabApplication, 'chunk-lab application is missing from the generated manifest');
const chunkLabExpectedMessages = [
  'main.hero.title',
  'main.hero.tagline',
  'main.nav.locale_label'
];
const chunkLabReferences = chunkLabApplication.references.map(reference => reference.message);
assert.deepEqual(chunkLabReferences, ['main.nav.locale_label']);
const childApplication = generatedManifest.applications['src/lib/components/ChunkLabCopy.svelte'];
assert.deepEqual(childApplication.references.map(reference => reference.message), ['main.hero.title']);
const sharedApplication = generatedManifest.applications['src/lib/chunk-lab-copy.ts'];
assert.deepEqual(sharedApplication.references.map(reference => reference.message), ['main.hero.tagline','main.codegen.intro']);
const labLayout = generatedManifest.applications['src/routes/chunk-lab/+layout.svelte'];
assert.deepEqual(labLayout.references.map(reference => reference.message), ['main.nav.locale_label']);

const localeModuleIds = new Set();
let runtimeImportCount = 0;
let semanticImportCount = 0;
let parameterlessLeafCount = 0;
const helperBodyPattern =
  /function (?:formatNumber|formatCurrency|formatDate|formatGeneratedNumber|parseGeneratedDecimal|coerceDate|plural[A-Z][A-Za-z0-9_]*)\s*\(/;
for (const message of Object.values(generatedManifest.messages)) {
  for (const entry of Object.values(message.locales)) {
    const moduleId = `src/lib/generated/linguini/${entry.module}`;
    localeModuleIds.add(moduleId);
    const source = readFileSync(join(siteRoot, moduleId), 'utf8');
    assert.doesNotMatch(source, helperBodyPattern, `${moduleId} contains an inline helper body`);
    for (const binding of semanticBindings) {
      assert.doesNotMatch(
        source,
        new RegExp(`^(?:const|function|type) ${escapeRegExp(binding)}\\b`, 'm'),
        `${moduleId} contains inline semantic binding ${binding}`
      );
    }
    if (/from ["'][^"']+\/_runtime\.js["']/.test(source)) runtimeImportCount += 1;
    if (source.includes('/semantic/')) semanticImportCount += 1;
    if (message.arity === 0) {
      parameterlessLeafCount += 1;
      assert.match(source, /export const message = /, `${moduleId} is not a value export`);
      assert.doesNotMatch(
        source,
        /export (?:const message = \([^)]*\) =>|function message\()/,
        `${moduleId} wraps a parameterless value in a function`
      );
    }
  }
}
assert.ok(runtimeImportCount > 0, 'no physical message imports a shared locale runtime');
assert.ok(semanticImportCount > 0, 'no physical message imports a shared semantic module');
assert.ok(parameterlessLeafCount > 0, 'no parameterless physical message leaves were checked');

for (const message of usedMessages) {
  assert.ok(generatedManifest.messages[message], `missing generated message ${message}`);
  for (const entry of Object.values(generatedManifest.messages[message].locales)) {
    const moduleId = `src/lib/generated/linguini/${entry.module}`;
    localeModuleIds.add(moduleId);
  }
}

const clientManifest = readJson(
  join(clientOutputRoot, '.vite/manifest.json'),
  'Vite client manifest'
);
const clientEntries = Object.entries(clientManifest);
const clientEntryById = new Map(clientEntries);
const {payloads} = await import(pathToFileURL(join(serverOutputRoot,'linguini-page-manifest.js')).href);
assert.deepEqual(Object.keys(payloads).sort(), ['/','/chunk-lab']);
const virtualLocaleModuleIds = new Set(Object.keys(clientManifest).filter(id => id.startsWith('virtual:linguini/locale/')));
assert.equal(virtualLocaleModuleIds.size, 0, 'source-scoped locale registries must not survive composed-page delivery');
// SvelteKit emits route components through generated client-optimized nodes.
// Resolve the lab node back to its source import so this check stays stable
// across route-node numbering changes.
const clientOptimizedNodeRoot = join(siteRoot, '.svelte-kit/generated/client-optimized/nodes');
const chunkLabNodeFiles = filesUnder(clientOptimizedNodeRoot).filter((file) => file.endsWith('.js'))
  .filter((file) => readFileSync(file, 'utf8').includes('src/routes/chunk-lab/+page.svelte'));
assert.equal(chunkLabNodeFiles.length, 1, 'chunk-lab has no unique generated client route node');
const chunkLabRouteNodeId = siteRelative(chunkLabNodeFiles[0]);
const chunkLabRouteEntry = clientManifest[chunkLabRouteNodeId];
assert.ok(chunkLabRouteEntry?.isEntry, 'chunk-lab client route entry is missing');

// Physical message leaves are implementation modules inside one dynamic locale
// entry. They must not survive as hundreds of browser-visible dynamic entries.
for (const moduleId of localeModuleIds) {
  assert.equal(
    clientManifest[moduleId],
    undefined,
    `physical locale leaf ${moduleId} escaped as its own client entry`
  );
}

const localeChunkFiles = new Set();
for (const [route,locales] of Object.entries(payloads)) {
  assert.deepEqual(Object.keys(locales).sort(), [...effectiveLocales].sort(), `${route} has incomplete locale coverage`);
  for (const file of Object.values(locales)) {
    assert.equal(statSync(join(clientOutputRoot,file)).isFile(), true);
    localeChunkFiles.add(file);
  }
}
assert.equal(localeChunkFiles.size, effectiveLocales.size * 2, 'each composed page must have exactly one file per locale');
const mainRouteNodeFile = filesUnder(clientOptimizedNodeRoot)
  .filter(file => file.endsWith('.js'))
  .find(file => readFileSync(file, 'utf8').includes('src/routes/+page.svelte'));
assert.ok(mainRouteNodeFile, 'main route has no generated client route node');
const mainRouteNodeId = siteRelative(mainRouteNodeFile);
assert.ok(clientManifest[mainRouteNodeId]?.isEntry, 'main route client entry is missing');
// Follow only static imports from that initial route entry. Every locale chunk
// must remain outside this closure; it is fetched through a dynamic loader.
function resolveStaticImport(specifier) {
  const normalized = specifier.replace(/^\.\//, '');
  const withoutRollupPrefix = normalized.startsWith('_')
    ? normalized.slice(1)
    : normalized;
  const matches = clientEntries.filter(([, entry]) => {
    const file = entry.file.replaceAll('\\', '/');
    const basename = file.slice(file.lastIndexOf('/') + 1);
    return file === normalized || basename === normalized || basename === withoutRollupPrefix;
  });
  assert.equal(matches.length, 1, `cannot resolve Vite static import ${specifier}`);
  return matches[0][0];
}

function collectStaticReachable(entryId) {
  const staticReachable = new Set();
  const pending = [entryId];
  while (pending.length > 0) {
    const moduleId = pending.pop();
    if (staticReachable.has(moduleId)) continue;
    staticReachable.add(moduleId);
    const entry = clientEntryById.get(moduleId);
    assert.ok(entry, `Vite manifest graph is missing ${moduleId}`);
    for (const imported of entry.imports ?? []) {
      pending.push(resolveStaticImport(imported));
    }
  }
  return staticReachable;
}
for (const root of [mainRouteNodeId,chunkLabRouteNodeId]) {
  for (const moduleId of collectStaticReachable(root)) {
    assert.equal(localeChunkFiles.has(clientManifest[moduleId].file), false, 'an initial route imported a locale catalog eagerly');
  }
}

const clientJavaScriptFiles = filesUnder(clientOutputRoot).filter(file => file.endsWith('.js'));
assert.ok(clientJavaScriptFiles.length <= effectiveLocales.size * 2 + 24,
  `client emitted ${clientJavaScriptFiles.length} JavaScript files for ${effectiveLocales.size} locales`);
const clientText = clientJavaScriptFiles.filter(file => !localeChunkFiles.has(siteRelative(file).replace('.svelte-kit/output/client/','')))
  .map(file => readFileSync(file,'utf8')).join('\n');
assert.doesNotMatch(clientText, /createLinguiniProvider|localeLoaders|__linguini_loaders|does not export messages/);
assert.equal(countMatches(clientText, /Invalid Linguini page payload/g), 1, 'the maintained page loader must be shared');
assert.match(clientText, /import\(/);

const messageIds = new Map(Object.keys(generatedManifest.messages).sort().map((name,index) => [name,index]));
const mainExpectedMessages = [...new Set([
  ...generatedManifest.applications['src/routes/+page.svelte'].references.map(reference => reference.message),
  'main.codegen.intro'
])];
for (const [route,expected] of [['/chunk-lab',chunkLabExpectedMessages],['/',mainExpectedMessages]]) {
  for (const [locale,file] of Object.entries(payloads[route])) {
    const source = readFileSync(join(clientOutputRoot,file),'utf8');
    assert.doesNotMatch(source, /(?:^|[;\n}])\s*import\b|main\.[a-z_]+/,
      `${route}/${locale} must be self-contained and omit canonical message paths`);
    const {messages} = await import(pathToFileURL(join(clientOutputRoot,file)).href);
    assert.deepEqual(Object.keys(messages).sort(), expected.map(name => String(messageIds.get(name))).sort(),
      `${route}/${locale} is not its exact surviving message closure`);
    if (route === '/chunk-lab') {
      const native = (await import(pathToFileURL(join(generatedRoot,`locales/${locale}/main.js`)).href)).main;
      for (const name of expected) {
        const value = name.split('.').slice(1).reduce((value,key) => value[key], native);
        assert.equal(messages[messageIds.get(name)], value);
      }
      assert.doesNotMatch(source, /\bfunction\b|=>/, 'constant-only lab payload must omit formatter/plural/helper bodies');
      assert.equal(Object.hasOwn(messages,messageIds.get('main.codegen.intro')), false, 'unused shared export leaked another page’s message');
    }
  }
}
console.log(`[production-graph] composed-pages=${Object.keys(payloads).length} locales=${effectiveLocales.size} lab-messages=${chunkLabExpectedMessages.length} payloads=${localeChunkFiles.size}`);
for (const route of ['/', '/chunk-lab']) {
  const source = readFileSync(join(clientOutputRoot,payloads[route].fr));
  console.log(`[production-graph] ${route}/fr payload bytes=${source.length} gzip=${gzipSync(source).length}`);
}

const serverManifest = readJson(
  join(serverOutputRoot, '.vite/manifest.json'),
  'Vite SSR manifest'
);
assert.deepEqual(Object.keys(serverManifest).filter(id => id.startsWith('virtual:linguini/locale/')), [],
  'SSR must not emit client-style dynamic locale entries');
const serverPage = serverManifest['src/routes/+page.svelte'];
assert.ok(serverPage?.isEntry, 'SSR page entry is missing');
const serverPagePath = join(serverOutputRoot, serverPage.file);
assert.equal(statSync(serverPagePath).isFile(), true, 'SSR page output is missing');
const serverPageText = readFileSync(serverPagePath, 'utf8');
assert.doesNotMatch(serverPageText, /registerLocaleLoader|__linguini_loaders/);

assert.match(english, /<title>Linguini \| typed localization language<\/title>/);
assert.match(english, /Internationalization for the rest of us/);
assert.doesNotMatch(output, /l\.main\.hero\.title\.toUpperCase/);

// The transformed message graph imports only the exact virtual message modules.
// These symbols are unique to the eager generated index/provider barrel.
assert.doesNotMatch(output, /createLinguiniProvider|localeLoaders/);
