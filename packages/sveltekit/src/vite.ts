import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import MagicString from 'magic-string';
import viteLinguini from '@antepodeum/linguini-vite';
import { svelteAdapter } from '@antepodeum/linguini-vite/svelte';
import type { LinguiniViteOptions,LinguiniPageBundles,LinguiniPage } from '@antepodeum/linguini-vite';
import type { Plugin, ResolvedConfig } from 'vite';
import { clientPages } from './build/routes.js';
import {clientEntry} from './build/client-entry.js';
import {deferredImports,PREPARE_EXPORT,LOCALE_EXPORT} from './build/deferred-imports.js';
import {clientNodes,isNodeCapability,SHARED_RUNTIME} from './build/client-nodes.js';

const PAGE_MANIFEST = 'virtual:linguini/page-manifest';

export type LinguiniSvelteKitOptions = Omit<LinguiniViteOptions,'adapter'>;

export default function linguini(options: LinguiniSvelteKitOptions = {}): Plugin[] {
  let compilation: LinguiniPageBundles | undefined;
  let pageRoots: readonly LinguiniPage[] = [];
  const owners = new Set<string>();
  const hosts = new Map<string,string>();
  let demand: Promise<Set<string>>;
  let resolveDemand: (messages:Set<string>) => void;
  let rejectDemand: (error:unknown) => void;
  function resetDemand() {
    compilation = undefined;
    pageRoots = [];
    owners.clear();hosts.clear();
    demand = new Promise((resolve,reject) => {resolveDemand=resolve;rejectDemand=reject;});
    void demand.catch(() => {});
  }
  resetDemand();
  const args = [...(options.args ?? ['build'])];
  if (args.includes('build')) {
    if (!args.includes('--svelte')) args.push('--svelte');
    if (!args.includes('--svelte-host-module')) args.push('--svelte-host-module','@antepodeum/linguini-sveltekit/client');
  }
  const core = viteLinguini({...options,args,adapter:{
    ...svelteAdapter,
    pageMessages: {
      module:'@antepodeum/linguini-sveltekit/client',export:'getPageMessage',
      async resolve(canonical) {
        return (await demand).has(canonical)
          ? {module:'@antepodeum/linguini-sveltekit/internal/deferred',export:'getDeferredPageMessage'}
          : {module:'@antepodeum/linguini-sveltekit/client',export:'getPageMessage'};
      },
      analysisModule(id) {if (isNodeCapability(id)) return 'export {};';},
    },
  }});
  let config: ResolvedConfig;
  const entry = clientEntry(async context => {
    try {
      const outputOptions = config.build.rollupOptions.output;
      const output = Array.isArray(outputOptions) ? outputOptions[0] : outputOptions;
      const pattern = output?.assetFileNames;
      if (typeof pattern !== 'string' || path.posix.dirname(pattern).includes('[')) {
        throw new Error('Unsupported SvelteKit asset output pattern for page payloads');
      }
      const pages = clientPages(context);
      pageRoots = pages;
      compilation = await core.api.compilePageBundles(context,pages,{assetsDir:path.posix.dirname(pattern)});
      const messages = new Set<string>();
      const nodes = new Map(pages.map(page => {
        const leaf = page.entries.at(-1);
        const id = typeof leaf === 'string' ? leaf : leaf?.id;
        if (!id) throw new Error(`Missing SvelteKit page node for ${page.id}`);
        return [page.id,id];
      }));
      for (const routes of Object.values(compilation.deferred)) {
        for (const [route,locales] of Object.entries(routes)) {
          const owner = nodes.get(route);
          if (!owner) throw new Error(`Missing SvelteKit preparation owner for ${route}`);
          owners.add(owner);
          for (const message of Object.values(locales)[0]?.messages ?? []) messages.add(message);
        }
      }
      const host = owners.size > 1 ? SHARED_RUNTIME : [...owners][0];
      if (host) for (const target of Object.keys(compilation.deferred)) hosts.set(target,host);
      if (owners.size > 1) context.emitFile({type:'chunk',id:SHARED_RUNTIME,name:'linguini-deferred',preserveSignature:'strict'});
      resolveDemand(messages);
      return 'export {};';
    } catch (error) {
      rejectDemand(error);
      throw error;
    }
  });
  function runtimeSource() {
    if (!compilation?.localeModule) throw new Error('Deferred messages require a locale runtime');
    return [
        'import {prepareDeferredPageMessages,prepareDeferredCurrentPageMessages,prepareDeferredNavigation} from "@antepodeum/linguini-sveltekit/internal/deferred";',
        'import {registerPagePreparation} from "@antepodeum/linguini-sveltekit/load";',
        `import {getCurrentLocale,registerLocaleLoader} from ${JSON.stringify(compilation.localeModule)};`,
        'registerLocaleLoader(prepareDeferredCurrentPageMessages);',
        'registerPagePreparation(prepareDeferredNavigation);',
        `export {prepareDeferredPageMessages as ${PREPARE_EXPORT},getCurrentLocale as ${LOCALE_EXPORT}};`,
    ].join('\n');
  }
  const nodes = clientNodes(async (_context,owner) => {
    await demand;
    if (!owners.has(owner)) return 'export {};';
    return owners.size > 1 ? `import ${JSON.stringify(SHARED_RUNTIME)};` : runtimeSource();
  },runtimeSource);
  return [core,entry,nodes,deferredImports(() => compilation,target => hosts.get(target) ?? '',() => pageRoots), {
    name: 'linguini-sveltekit-pages',
    configResolved(value) { config = value; },
    buildStart() {resetDemand();},
    resolveId(id) {
      if (id !== PAGE_MANIFEST) return;
      if (config.command === 'serve') return '\0' + PAGE_MANIFEST;
      if (!config.build.ssr) throw new Error('Linguini page manifest is server-only');
      return {id:PAGE_MANIFEST,external:true};
    },
    load(id) {
      if (id === '\0' + PAGE_MANIFEST) return 'export const payloads = {};';
    },
    renderChunk(code,chunk) {
      if (!config.build.ssr || !code.includes(PAGE_MANIFEST)) return;
      const magic = new MagicString(code);
      const pending: unknown[] = [this.parse(code)];
      const relative = path.posix.relative(path.posix.dirname(chunk.fileName),'linguini-page-manifest.js');
      const target = relative.startsWith('.') ? relative : './' + relative;
      while (pending.length) {
        const value = pending.pop();
        if (!value || typeof value !== 'object') continue;
        const expression = value as Record<string,unknown>;
        if (expression.type === 'ImportExpression') {
          const source = expression.source as Record<string,unknown>;
          if (source.value === PAGE_MANIFEST && typeof source.start === 'number' && typeof source.end === 'number') {
            magic.overwrite(source.start,source.end,JSON.stringify(target));
          }
        }
        for (const child of Object.values(expression)) {
          if (Array.isArray(child)) pending.push(...child);
          else if (child && typeof child === 'object') pending.push(child);
        }
      }
      return {code:magic.toString(),map:magic.generateMap({hires:true})};
    },
    generateBundle: {
      order: 'post',
      async handler() {
        if (config.build.ssr) return;
        if (!compilation) throw new Error('Linguini requires a prepared SvelteKit client graph');
        const payloads = compilation.pages;
        const manifest: Record<string,Record<string,string>> = Object.create(null);
        for (const [page,locales] of Object.entries(payloads)) {
          const entries = manifest[page] = Object.create(null);
          for (const [locale,payload] of Object.entries(locales)) {
            entries[locale] = payload.file;
          }
        }
        const serverOutput = path.resolve(config.root,config.build.outDir,'../server');
        await mkdir(serverOutput,{recursive:true});
        await writeFile(path.join(serverOutput,'linguini-page-manifest.js'),`export const payloads = ${JSON.stringify(manifest)};\n`);
      },
    },
  }];
}
