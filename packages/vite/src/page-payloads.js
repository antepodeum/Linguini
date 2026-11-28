import { createHash } from "node:crypto";
import path from "node:path";
import { build, version as viteVersion } from "vite";

import { toVitePath } from "./paths.js";
import { pagePluralPlugin } from "./plural-payloads.js";

const singleChunk = Number(viteVersion.split(".")[0]) >= 8
  ? { codeSplitting: false }
  : { inlineDynamicImports: true };

const indices = new WeakMap();
export function pageMessageIndex(manifest, canonical) {
  let known = indices.get(manifest);
  if (!known) {
    known = new Map([...manifest.messages.keys()].sort().map((name, index) => [name, index]));
    indices.set(manifest, known);
  }
  return known.get(canonical);
}

export {pageMessageUsage, pageMessagePlan} from "./page-demand.js";
import {pageMessagePlan,emittedImports} from "./page-demand.js";
const ENTRY = "\0linguini-page-payload";

/** Compile initial delivery; integrations without deferred preparation keep failing closed. */
export async function compilePagePayloads(context,pages,manifest,config) {
  return (await compileBundles(context,pages,manifest,config,false)).pages;
}

/** Initial and lazy closures share one emitter and one content cache. */
export async function compilePageBundles(context,pages,manifest,config) {
  return compileBundles(context,pages,manifest,config,true);
}

async function compileBundles(context,pages,manifest,config,allowDeferred) {
  if (!manifest) throw new Error("Linguini page payloads require a native bundler manifest");
  const initial = Object.create(null);
  const deferred = Object.create(null);
  const deferredImports = Object.create(null);
  const cache = new Map();
  async function compile(names,label) {
    const key = JSON.stringify(names);
    if (!cache.has(key)) cache.set(key,await compileClosure(context,names,manifest,config,label));
    return cache.get(key);
  }
  for (const page of pages) {
    const usage = await pageMessagePlan(context,page.entries,config);
    deferredImports[page.id] = usage.deferred.map(boundary => boundary.id);
    const missing = [...new Set(usage.deferred.flatMap(boundary => boundary.messages))]
      .filter(name => !usage.messages.includes(name)).sort();
    if (missing.length && !allowDeferred) {
      throw new Error(`Linguini cannot yet prepare deferred messages ${missing.join(', ')} for page ${page.id}`);
    }
    if (usage.messages.length) initial[page.id] = await compile(usage.messages,page.id);
    for (const boundary of usage.deferred) {
      const names = boundary.messages.filter(name => !boundary.available.includes(name));
      if (!names.length) continue;
      const entries = deferred[boundary.id] ??= Object.create(null);
      entries[page.id] = await compile(names,`${page.id} deferred ${boundary.id}`);
    }
  }
  return {pages:initial,deferred,deferredImports,localeModule:manifest.localeHelper && toVitePath(manifest.localeHelper.file)};
}

async function compileClosure(context,names,manifest,config,label) {
  const locales = Object.create(null);
  const compiled = new Map();
  for (const locale of manifest.effectiveLocales) {
    const imports = names.map((name, index) => {
      const message = manifest.messages.get(name);
      if (!message) throw new Error(`Linguini page ${label} uses unknown message ${name}`);
      const leaf = message.locales.get(locale) ?? message.locales.get(manifest.baseLocale);
      if (!leaf) throw new Error(`Linguini page ${label} has no implementation of ${name} for ${locale}`);
      return `import { message as m${index} } from ${JSON.stringify(toVitePath(leaf.module))};`;
    });
    const source = imports.join("\n") + "\nexport const messages = {" + names.map((name, index) => `${pageMessageIndex(manifest, name)}:m${index}`).join(",") + "};\n";
    const result = await build({
      configFile: false,
      root: config.root,
      logLevel: "silent",
      plugins: [pagePluralPlugin(manifest,names,locale), {
        name: "linguini-page-payload",
        resolveId(id) { if (id === ENTRY) return id; },
        load(id) { if (id === ENTRY) return source; }
      }],
      build: {
        write: false,
        outDir: path.resolve(config.root, config.build.outDir ?? 'dist', config.build.assetsDir, 'linguini'),
        sourcemap: Boolean(config.build.sourcemap),
        minify: config.build.minify,
        target: config.build.target,
        modulePreload: false,
        rollupOptions: {
          input: ENTRY,
          preserveEntrySignatures: "strict",
          output: { format: "es", entryFileNames: "payload.js", ...singleChunk }
        }
      }
    });
    const chunks = result.output.filter(output => output.type === "chunk");
    if (chunks.length !== 1 || emittedImports(context,chunks[0].code).length) {
      throw new Error(`Linguini page ${label}/${locale} is not a self-contained payload`);
    }
    let code = chunks[0].code.replace(/\n?\/\/# sourceMappingURL=[^\n]*\n?$/, "\n");
    let map;
    const name = encodeURIComponent(locale) + '.js';
    if (config.build.sourcemap) {
      const asset = result.output.find(output => output.type === 'asset' && output.fileName.endsWith('.js.map'));
      if (!asset) throw new Error(`Linguini page ${label}/${locale} has no source map`);
      map = JSON.parse(typeof asset.source === 'string' ? asset.source : Buffer.from(asset.source).toString('utf8'));
      map.file = name;
      const absolute = value => /^(?:[a-z][a-z\d+.-]*:|\/)/i.test(value);
      if (!map.sourceRoot || !absolute(map.sourceRoot)) {
        map.sources = map.sources.map(source => source.includes('\0') || absolute(source) ? source : '../' + source);
      }
      if (config.build.sourcemap === 'inline') {
        code += '//# sourceMappingURL=data:application/json;base64,' + Buffer.from(JSON.stringify(map)).toString('base64') + '\n';
      } else if (config.build.sourcemap !== 'hidden') {
        code += '//# sourceMappingURL=' + name + '.map\n';
      }
    }
    compiled.set(locale, {code,map});
  }
  const hash = createHash("sha256").update(JSON.stringify([...compiled])).digest("hex").slice(0, 20);
  for (const [locale, {code,map}] of compiled) {
    const fileName = `${config.build.assetsDir}/linguini/${hash}/${encodeURIComponent(locale)}.js`;
    context.emitFile({ type: "asset", fileName, source: code });
    if (map && config.build.sourcemap !== 'inline') {
      context.emitFile({type:'asset',fileName:fileName + '.map',source:JSON.stringify(map)});
    }
    locales[locale] = { file: fileName, messages: names };
  }
  return locales;
}
