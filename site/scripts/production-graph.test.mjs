import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

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
  const exportedBinding = /export (?:const|function|type) ([A-Za-z_$][\w$]*)/.exec(source)?.[1];
  assert.ok(exportedBinding, `${key} has no single exported semantic binding`);
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

const localeModuleIds = new Set();
let runtimeImportCount = 0;
let semanticImportCount = 0;
const helperBodyPattern =
  /function (?:formatNumber|formatCurrency|formatDate|formatGeneratedNumber|parseGeneratedDecimal|coerceDate|plural[A-Z][A-Za-z0-9_]*)\s*\(/;
for (const message of Object.values(generatedManifest.messages)) {
  for (const entry of Object.values(message.locales)) {
    const moduleId = `src/lib/generated/linguini/${entry.module}`;
    const source = readFileSync(join(siteRoot, moduleId), 'utf8');
    assert.doesNotMatch(source, helperBodyPattern, `${moduleId} contains an inline helper body`);
    for (const binding of semanticBindings) {
      assert.doesNotMatch(
        source,
        new RegExp(`^(?:const|function|type) ${escapeRegExp(binding)}\\b`, 'm'),
        `${moduleId} contains inline semantic binding ${binding}`
      );
    }
    if (source.includes('/_runtime"')) runtimeImportCount += 1;
    if (source.includes('/semantic/')) semanticImportCount += 1;
  }
}
assert.ok(runtimeImportCount > 0, 'no physical message imports a shared locale runtime');
assert.ok(semanticImportCount > 0, 'no physical message imports a shared semantic module');

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
for (const moduleId of localeModuleIds) {
  const entry = clientManifest[moduleId];
  assert.ok(entry, `locale module ${moduleId} is absent from the Vite client manifest`);
  assert.equal(entry.isDynamicEntry, true, `locale module ${moduleId} is not a dynamic entry`);
  assert.equal(typeof entry.file, 'string');
  assert.equal(
    statSync(join(clientOutputRoot, entry.file)).isFile(),
    true,
    `locale module ${moduleId} has no physical output chunk`
  );
}

// SvelteKit's route node is the initial entry that references every generated
// locale module. Keep this assertion based on module IDs, never hash-derived
// chunk names.
const initialEntries = clientEntries.filter(([, entry]) =>
  (entry.dynamicImports ?? []).some((moduleId) => localeModuleIds.has(moduleId))
);
assert.equal(initialEntries.length, 1, 'expected one initial entry for locale dynamic imports');
const [initialEntryId, initialEntry] = initialEntries[0];
assert.equal(initialEntry.isEntry, true);
for (const moduleId of localeModuleIds) {
  assert.ok(initialEntry.dynamicImports.includes(moduleId));
}

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

const staticReachable = new Set();
const pending = [initialEntryId];
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
for (const moduleId of localeModuleIds) {
  assert.equal(
    staticReachable.has(moduleId),
    false,
    `locale module ${moduleId} is reachable through static imports`
  );
}

const clientText = filesUnder(clientOutputRoot)
  .filter((file) => file.endsWith('.js'))
  .map((file) => readFileSync(file, 'utf8'))
  .join('\n');
// The client must not pull the eager generated index/provider into the initial
// graph. The SSR route is checked separately below.
assert.doesNotMatch(clientText, /createLinguiniProvider|localeLoaders/);
assert.match(clientText, /import\(/);

const serverManifest = readJson(
  join(serverOutputRoot, '.vite/manifest.json'),
  'Vite SSR manifest'
);
for (const moduleId of localeModuleIds) {
  assert.equal(
    serverManifest[moduleId],
    undefined,
    `SSR emitted locale module ${moduleId} as a client-style dynamic entry`
  );
}
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
