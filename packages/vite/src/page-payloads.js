import { createHash } from "node:crypto";
import path from "node:path";
import { build, version as viteVersion } from "vite";

import { toVitePath } from "./paths.js";

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

const ENTRY = "\0linguini-page-analysis";
const GRAPH = "\0linguini-page-module/";
const MESSAGE = "linguini-page-message:";
const DEFERRED = "linguini-page-deferred:";
const VIRTUAL_MESSAGE = "\0virtual:linguini/message/";

/** Re-shake each page independently, including shared modules with multiple exported functions. */
export async function pageMessageUsage(context, entries, config) {
  const root = entries.map((entry, index) => {
    if (typeof entry === 'string') return `export * as node${index} from ${JSON.stringify(entry)};`;
    const selected = entry.exports.map((parts, binding) =>
      `export const binding${index}_${binding} = node${index}${parts.map(part => `[${JSON.stringify(part)}]`).join('')};`).join('\n');
    return `import * as node${index} from ${JSON.stringify(entry.id)};\n${selected}`;
  }).join("\n");
  const result = await build({
    configFile: false,
    root: config.root,
    logLevel: "silent",
    plugins: [{
      name: "linguini-page-analysis",
      enforce: "pre",
      async resolveId(source, importer) {
        if (source === ENTRY || source.startsWith(GRAPH)) return source;
        const parent = importer?.startsWith(GRAPH) ? importer.slice(GRAPH.length) : undefined;
        const resolved = source.startsWith("\0") || context.getModuleInfo(source)
          ? { id: source }
          : await context.resolve(source, parent, { skipSelf: false });
        if (!resolved) throw new Error(`Linguini page analysis cannot resolve ${source} from ${parent}`);
        if (resolved.id.startsWith(VIRTUAL_MESSAGE)) {
          const canonical = resolved.id.slice(VIRTUAL_MESSAGE.length).split("/").at(-1);
          return { id: MESSAGE + canonical, external: true, moduleSideEffects: false };
        }
        if (resolved.external) return resolved;
        return GRAPH + resolved.id;
      },
      async resolveDynamicImport(source, importer) {
        // A lazy application boundary is not part of its parent's initial message closure.
        if (typeof source !== 'string') return false;
        const parent = importer?.startsWith(GRAPH) ? importer.slice(GRAPH.length) : undefined;
        const resolved = await context.resolve(source,parent,{skipSelf:false});
        if (!resolved || resolved.external) return {id:source,external:true};
        return {id:DEFERRED + Buffer.from(resolved.id).toString('base64url'),external:true};
      },
      load(id) {
        if (id === ENTRY) return root;
        if (!id.startsWith(GRAPH)) return;
        const original = id.slice(GRAPH.length);
        if (original.startsWith("\0virtual:linguini/locale-registry")) {
          return "export function acquireLocaleLoader() { return () => {}; }";
        }
        const info = context.getModuleInfo(original);
        if (!info || typeof info.code !== "string") {
          throw new Error(`Linguini page analysis needs transformed module ${original}`);
        }
        return {
          code: info.code,
          moduleSideEffects: info.moduleSideEffects,
          syntheticNamedExports: info.syntheticNamedExports,
        };
      }
    }],
    build: {
      write: false,
      lib: { entry: ENTRY, formats: ["es"] },
      minify: false,
      target: config.build.target,
      modulePreload: false,
      rollupOptions: {
        input: ENTRY,
        preserveEntrySignatures: "strict",
        output: { format: "es", ...singleChunk }
      }
    }
  });
  const used = new Set();
  const deferred = new Set();
  for (const chunk of (Array.isArray(result) ? result : [result]).flatMap(bundle => bundle.output)) {
    if (chunk.type !== "chunk") continue;
    // Some hosts retain removed dependencies in metadata; the emitted program is authoritative.
    for (const imported of emittedImports(context,chunk.code)) {
      const source = imported.source?.value;
      if (typeof source !== 'string') continue;
      if (!imported.dynamic && source.startsWith(MESSAGE)) {
        used.add(Buffer.from(source.slice(MESSAGE.length),"hex").toString("utf8"));
      } else if (imported.dynamic && source.startsWith(DEFERRED)) {
        deferred.add(Buffer.from(source.slice(DEFERRED.length),'base64url').toString('utf8'));
      }
    }
  }
  return {messages:[...used].sort(),deferred:[...deferred].sort()};
}

function emittedImports(context,code) {
  const pending = [context.parse(code)];
  const imports = [];
  while (pending.length) {
    const node = pending.pop();
    if (!node || typeof node !== 'object' || typeof node.type !== 'string') continue;
    if (node.type === 'ImportExpression') imports.push({source:node.source,dynamic:true});
    else if (['ImportDeclaration','ExportAllDeclaration','ExportNamedDeclaration'].includes(node.type) && node.source) {
      imports.push({source:node.source,dynamic:false});
    }
    for (const child of Object.values(node)) {
      if (Array.isArray(child)) pending.push(...child);
      else if (child && typeof child === 'object') pending.push(child);
    }
  }
  return imports;
}

function graphHasMessages(context,entry) {
  const pending = [entry];
  const visited = new Set();
  while (pending.length) {
    const id = pending.pop();
    if (visited.has(id)) continue;
    visited.add(id);
    if (id.startsWith(VIRTUAL_MESSAGE)) return true;
    const info = context.getModuleInfo(id);
    if (info) pending.push(...info.importedIds,...info.dynamicallyImportedIds);
  }
  return false;
}

/** Bundle selected compiled leaves once per page/locale, letting the host Vite shake their dependencies. */
export async function compilePagePayloads(context, pages, manifest, config) {
  const payloads = Object.create(null);
  if (!manifest) throw new Error("Linguini page payloads require a native bundler manifest");
  for (const page of pages) {
    const usage = await pageMessageUsage(context,page.entries,config);
    const names = usage.messages;
    const deferred = [...usage.deferred];
    const visited = new Set();
    while (deferred.length) {
      const id = deferred.pop();
      if (visited.has(id) || !graphHasMessages(context,id)) continue;
      visited.add(id);
      const child = await pageMessageUsage(context,[id],config);
      const missing = child.messages.filter(name => !names.includes(name));
      if (missing.length) {
        throw new Error(`Linguini cannot yet prepare deferred messages ${missing.join(', ')} in ${id} for page ${page.id}`);
      }
      deferred.push(...child.deferred);
    }
    if (names.length === 0) continue;
    const locales = Object.create(null);
    const compiled = new Map();
    payloads[page.id] = locales;
    for (const locale of manifest.effectiveLocales) {
      const imports = names.map((name, index) => {
        const message = manifest.messages.get(name);
        if (!message) throw new Error(`Linguini page ${page.id} uses unknown message ${name}`);
        const leaf = message.locales.get(locale) ?? message.locales.get(manifest.baseLocale);
        if (!leaf) throw new Error(`Linguini page ${page.id} has no implementation of ${name} for ${locale}`);
        return `import { message as m${index} } from ${JSON.stringify(toVitePath(leaf.module))};`;
      });
      const source = imports.join("\n") + "\nexport const messages = {" + names.map((name, index) => `${pageMessageIndex(manifest, name)}:m${index}`).join(",") + "};\n";
      const result = await build({
        configFile: false,
        root: config.root,
        logLevel: "silent",
        plugins: [{
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
        throw new Error(`Linguini page ${page.id}/${locale} is not a self-contained payload`);
      }
      let code = chunks[0].code.replace(/\n?\/\/# sourceMappingURL=[^\n]*\n?$/, "\n");
      let map;
      const name = encodeURIComponent(locale) + '.js';
      if (config.build.sourcemap) {
        const asset = result.output.find(output => output.type === 'asset' && output.fileName.endsWith('.js.map'));
        if (!asset) throw new Error(`Linguini page ${page.id}/${locale} has no source map`);
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
  }
  return payloads;
}
