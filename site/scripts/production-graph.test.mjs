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
assert.equal(generatedManifest.version, 4);
assert.equal(generatedManifest.locale_loading, 'dynamic');

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
for (const message of usedMessages) {
  assert.ok(generatedManifest.messages[message], `missing generated message ${message}`);
  for (const entry of Object.values(generatedManifest.messages[message].locales)) {
    localeModuleIds.add(`src/lib/generated/linguini/${entry.module}`);
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
