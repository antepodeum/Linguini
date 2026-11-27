import path from "node:path";
import { build, version as viteVersion } from "vite";
import {withPreparedMessages} from './deferred-plan.js';

const singleChunk = Number(viteVersion.split(".")[0]) >= 8
  ? { codeSplitting: false }
  : { inlineDynamicImports: true };

const ENTRY = "\0linguini-page-analysis";
const GRAPH = "\0linguini-page-module/";
const BOUNDARY = "\0linguini-page-boundary/";
const MESSAGE = "linguini-page-message:";
const DEFERRED = "linguini-page-deferred:";
const VIRTUAL_MESSAGE = "\0virtual:linguini/message/";

/** Re-shake each page independently, including shared modules with multiple exported functions. */
export async function pageMessageUsage(context, entries, config, followDeferred = false) {
  const {messages,deferred} = await analyzePage(context,entries,config,followDeferred);
  return {messages,deferred};
}

/** Preserve each lazy entry's consumed exports when planning its own message closure. */
export async function pageMessagePlan(context,entries,config) {
  const initial = await analyzePage(context,entries,config,false);
  if (!initial.deferred.some(id => graphHasMessages(context,id))) {
    return {messages:initial.messages,deferred:[]};
  }
  const complete = await analyzePage(context,entries,config,true);
  const deferred = [];
  const demands = new Map();
  for (const boundary of complete.boundaries) {
    if (!graphHasMessages(context,boundary.id)) continue;
    const messages = new Set();
    const children = new Set();
    const variants = [];
    for (const variant of boundary.variants) {
      const usage = await analyzePage(context,[{id:boundary.id,exports:variant.map(name => [name])}],config,false);
      variants.push(usage.messages);
      for (const name of usage.messages) messages.add(name);
      for (const id of usage.deferred) children.add(id);
    }
    demands.set(boundary.id,variants);
    deferred.push({id:boundary.id,exports:boundary.exports,messages:[...messages].sort(),deferred:[...children].sort()});
  }
  const planned = new Set(deferred.map(boundary => boundary.id));
  const unmapped = initial.deferred.filter(id => graphHasMessages(context,id) && !planned.has(id));
  if (unmapped.length) {
    throw new Error(`Linguini cannot determine deferred boundaries for ${unmapped.join(', ')}`);
  }
  const prepared = withPreparedMessages(initial,deferred);
  for (const boundary of prepared) {
    // Export variants may read different messages already guaranteed by every
    // parent path. Only their remaining payload demand must agree.
    const available = new Set(boundary.available);
    let expected;
    for (const messages of demands.get(boundary.id)) {
      const additional = JSON.stringify(messages.filter(name => !available.has(name)));
      if (expected !== undefined && expected !== additional) {
        throw new Error(`Linguini deferred imports of ${boundary.id} require different message sets; call-site preparation is not supported yet`);
      }
      expected = additional;
    }
  }
  return {messages:initial.messages,deferred:prepared};
}

async function analyzePage(context, entries, config, followDeferred) {
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
        if (source === ENTRY || source.startsWith(GRAPH) || source.startsWith(BOUNDARY)) return source;
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
        if (followDeferred) return BOUNDARY + resolved.id;
        return {id:DEFERRED + Buffer.from(resolved.id).toString('base64url'),external:true};
      },
      load(id) {
        if (id === ENTRY) return root;
        if (id.startsWith(BOUNDARY)) {
          const original = id.slice(BOUNDARY.length);
          const source = JSON.stringify(GRAPH + original);
          // Analysis-only async facades preserve lazy entry identities, including pure
          // re-exports and modules also imported eagerly elsewhere in the page.
          const defaults = context.getModuleInfo(original)?.exports?.includes('default')
            ? `export {default} from ${source};` : '';
          return `await 0; export * from ${source}; ${defaults}`;
        }
        if (!id.startsWith(GRAPH)) return;
        const original = id.slice(GRAPH.length);
        const model = config.analysisModule?.(original);
        if (model !== undefined) {
          if (typeof model !== 'string') throw new TypeError(`Linguini analysis module ${original} must contain JavaScript source`);
          return model;
        }
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
      lib: { entry: ENTRY, formats: ["es"], fileName: "linguini-page-analysis" },
      minify: false,
      // The async facade is an analysis boundary, not code delivered to this target.
      target: followDeferred ? 'esnext' : config.build.target,
      modulePreload: false,
      rollupOptions: {
        input: ENTRY,
        preserveEntrySignatures: "strict",
        // Inlining dynamic imports can retain their entire namespace on some hosts.
        // Keep those boundaries for demand analysis; compiled payloads remain one chunk.
        output: { format: "es", ...(followDeferred ? {} : singleChunk) }
      }
    }
  });
  const used = new Set();
  const deferred = new Set();
  const chunks = new Map((Array.isArray(result) ? result : [result]).flatMap(bundle => bundle.output)
    .filter(chunk => chunk.type === 'chunk').map(chunk => [chunk.fileName,chunk]));
  const boundaries = new Map();
  for (const chunk of chunks.values()) {
    // Some hosts retain removed dependencies in metadata; the emitted program is authoritative.
    for (const imported of emittedImports(context,chunk.code)) {
      const source = imported.source?.value;
      if (typeof source !== 'string') continue;
      if (!imported.dynamic && source.startsWith(MESSAGE)) {
        used.add(Buffer.from(source.slice(MESSAGE.length),"hex").toString("utf8"));
      } else if (imported.dynamic && source.startsWith(DEFERRED)) {
        deferred.add(Buffer.from(source.slice(DEFERRED.length),'base64url').toString('utf8'));
      } else if (imported.dynamic && followDeferred) {
        const target = chunks.get(path.posix.normalize(path.posix.join(path.posix.dirname(chunk.fileName),source)));
        if (!target) continue;
        if (!target.facadeModuleId?.startsWith(BOUNDARY)) {
          throw new Error('Linguini deferred analysis requires a module facade for each lazy entry');
        }
        const id = target.facadeModuleId.slice(BOUNDARY.length);
        const previous = boundaries.get(id);
        const requested = [...new Set(imported.exports ?? target.exports)].sort();
        const exports = [...new Set([...(previous?.exports.flat() ?? []),...requested])].sort();
        const known = context.getModuleInfo(id)?.exports;
        if (!known || exports.some(name => !known.includes(name))) {
          throw new Error(`Linguini deferred analysis cannot resolve consumed exports of ${id}`);
        }
        const variants = previous?.variants ?? [];
        if (!variants.some(variant => JSON.stringify(variant) === JSON.stringify(requested))) variants.push(requested);
        boundaries.set(id,{id,exports:exports.map(name => [name]),variants});
      }
    }
  }
  return {messages:[...used].sort(),deferred:[...deferred].sort(),
    boundaries:[...boundaries.values()].sort((left,right) => left.id < right.id ? -1 : left.id > right.id ? 1 : 0)};
}

export function emittedImports(context,code) {
  const pending = [{node:context.parse(code),ancestors:[]}];
  const imports = [];
  while (pending.length) {
    const {node,ancestors} = pending.pop();
    if (!node || typeof node !== 'object' || typeof node.type !== 'string') continue;
    if (node.type === 'ImportExpression') imports.push({source:node.source,dynamic:true,exports:consumedExports(node,ancestors)});
    else if (['ImportDeclaration','ExportAllDeclaration','ExportNamedDeclaration'].includes(node.type) && node.source) {
      imports.push({source:node.source,dynamic:false});
    }
    for (const child of Object.values(node)) {
      if (Array.isArray(child)) pending.push(...child.map(value => ({node:value,ancestors:[node,...ancestors].slice(0,3)})));
      else if (child && typeof child === 'object') pending.push({node:child,ancestors:[node,...ancestors].slice(0,3)});
    }
  }
  return imports;
}

function consumedExports(expression,ancestors) {
  let [parent,grandparent] = ancestors;
  if (parent?.type === 'AwaitExpression') {
    expression = parent;
    parent = grandparent;
  }
  if (parent?.type === 'VariableDeclarator' && parent.init === expression && parent.id.type === 'ObjectPattern') {
    const names = [];
    for (const property of parent.id.properties) {
      if (property.type !== 'Property' || property.computed) return undefined;
      const name = property.key.type === 'Identifier' ? property.key.name : property.key.value;
      if (typeof name !== 'string') return undefined;
      names.push(name);
    }
    return names;
  }
  if (parent?.type === 'MemberExpression' && parent.object === expression) {
    const name = parent.computed ? parent.property.value : parent.property.name;
    if (typeof name === 'string' && name !== 'then') return [name];
  }
  if (parent?.type === 'ExpressionStatement') return [];
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

