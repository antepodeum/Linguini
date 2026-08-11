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

function countMatches(source, pattern) {
  return [...source.matchAll(pattern)].length;
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
    if (source.includes('/_runtime"')) runtimeImportCount += 1;
    if (source.includes('/semantic/')) semanticImportCount += 1;
    if (message.arity === 0) {
      parameterlessLeafCount += 1;
      assert.match(source, /export const message = /, `${moduleId} is not a value export`);
      assert.doesNotMatch(
        source,
        /export const message = \(\): string =>/,
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
const virtualLocaleModuleIds = new Set(
  [...effectiveLocales].map(
    (locale) => `virtual:linguini/locale/${Buffer.from(locale, 'utf8').toString('hex')}`
  )
);

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
for (const moduleId of virtualLocaleModuleIds) {
  const entry = clientManifest[moduleId];
  assert.ok(entry, `virtual locale entry ${moduleId} is absent from the Vite client manifest`);
  assert.equal(entry.isDynamicEntry, true, `virtual locale entry ${moduleId} is not dynamic`);
  assert.equal(typeof entry.file, 'string');
  assert.equal(
    statSync(join(clientOutputRoot, entry.file)).isFile(),
    true,
    `virtual locale entry ${moduleId} has no physical output chunk`
  );
  localeChunkFiles.add(entry.file);
}
assert.equal(
  localeChunkFiles.size,
  effectiveLocales.size,
  'effective locales do not map one-to-one to emitted locale chunks'
);

// SvelteKit's route node must reference exactly one shared entry per locale,
// never one dynamic entry per message and locale.
const initialEntries = clientEntries.filter(([, entry]) =>
  (entry.dynamicImports ?? []).some((moduleId) => virtualLocaleModuleIds.has(moduleId))
);
assert.equal(initialEntries.length, 1, 'expected one initial entry for locale dynamic imports');
const [initialEntryId, initialEntry] = initialEntries[0];
assert.equal(initialEntry.isEntry, true);
assert.deepEqual(new Set(initialEntry.dynamicImports), virtualLocaleModuleIds);

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
for (const moduleId of virtualLocaleModuleIds) {
  assert.equal(
    staticReachable.has(moduleId),
    false,
    `virtual locale entry ${moduleId} is reachable through static imports`
  );
}

const clientJavaScriptFiles = filesUnder(clientOutputRoot).filter((file) => file.endsWith('.js'));
assert.ok(
  clientJavaScriptFiles.length <= effectiveLocales.size + 24,
  `client emitted ${clientJavaScriptFiles.length} JavaScript files for ${effectiveLocales.size} locales`
);
const clientText = clientJavaScriptFiles
  .map((file) => readFileSync(file, 'utf8'))
  .join('\n');
const registryJavaScriptFiles = clientJavaScriptFiles.filter((file) =>
  readFileSync(file, 'utf8').includes('does not export messages')
);
assert.equal(
  registryJavaScriptFiles.length,
  1,
  'client graph must contain exactly one shared locale registry module'
);
assert.equal(
  countMatches(clientText, /does not export messages/g),
  1,
  'shared locale registry must validate locale payloads exactly once'
);
// The client must not pull the eager generated index/provider into the initial
// graph. The SSR route is checked separately below.
assert.doesNotMatch(clientText, /createLinguiniProvider|localeLoaders/);
assert.doesNotMatch(clientText, /not prepared/);
assert.doesNotMatch(clientText, /does not export message\b/);
assert.doesNotMatch(clientText, /__linguini_loaders/);
assert.match(clientText, /import\(/);

const initialRouteText = readFileSync(join(clientOutputRoot, initialEntry.file), 'utf8');
const dynamicImportTargets = [
  ...initialRouteText.matchAll(/import\((["'`])([^"'`]+)\1\)/g)
].map((match) => match[2].split('/').pop());
assert.equal(
  dynamicImportTargets.length,
  effectiveLocales.size,
  `initial route must contain exactly one literal dynamic import per locale (${effectiveLocales.size})`
);
for (const moduleId of virtualLocaleModuleIds) {
  const chunkFile = join(clientOutputRoot, clientManifest[moduleId].file);
  const chunkText = readFileSync(chunkFile, 'utf8');
  assert.match(chunkText, /\bas messages\b/, `locale chunk ${moduleId} has no messages export`);
  assert.equal(
    dynamicImportTargets.filter((target) => target === clientManifest[moduleId].file.split('/').pop()).length,
    1,
    `initial route must import locale chunk ${moduleId} exactly once`
  );
}

const serverManifest = readJson(
  join(serverOutputRoot, '.vite/manifest.json'),
  'Vite SSR manifest'
);
for (const moduleId of virtualLocaleModuleIds) {
  assert.equal(
    serverManifest[moduleId],
    undefined,
    `SSR emitted virtual locale module ${moduleId} as a client-style dynamic entry`
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
