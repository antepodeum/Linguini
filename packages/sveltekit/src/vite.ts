import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import MagicString from 'magic-string';
import viteLinguini from '@antepodeum/linguini-vite';
import { svelteAdapter } from '@antepodeum/linguini-vite/svelte';
import type { LinguiniViteOptions } from '@antepodeum/linguini-vite';
import type { Plugin, ResolvedConfig } from 'vite';
import { clientPages } from './build/routes.js';

const PAGE_MANIFEST = 'virtual:linguini/page-manifest';

export type LinguiniSvelteKitOptions = Omit<LinguiniViteOptions,'adapter'>;

export default function linguini(options: LinguiniSvelteKitOptions = {}): Plugin[] {
  const args = [...(options.args ?? ['build'])];
  if (args.includes('build')) {
    if (!args.includes('--svelte')) args.push('--svelte');
    if (!args.includes('--svelte-host-module')) args.push('--svelte-host-module','@antepodeum/linguini-sveltekit/client');
  }
  const core = viteLinguini({...options,args,adapter:{
    ...svelteAdapter,
    pageMessages: {module:'@antepodeum/linguini-sveltekit/client',export:'getPageMessage'},
  }});
  let config: ResolvedConfig;
  return [core, {
    name: 'linguini-sveltekit-pages',
    configResolved(value) { config = value; },
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
        const pages = clientPages(this);
        const outputOptions = config.build.rollupOptions.output;
        const output = Array.isArray(outputOptions) ? outputOptions[0] : outputOptions;
        const pattern = output?.assetFileNames;
        if (typeof pattern !== 'string' || path.posix.dirname(pattern).includes('[')) {
          throw new Error('Unsupported SvelteKit asset output pattern for page payloads');
        }
        const payloads = await core.api.compilePages(this,pages,{assetsDir:path.posix.dirname(pattern)});
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
