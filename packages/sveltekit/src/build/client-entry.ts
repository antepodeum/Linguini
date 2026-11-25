import path from 'node:path';
import {normalizePath} from 'vite';
import type {Plugin,ResolvedConfig,Rollup} from 'vite';
import {isMessageModule} from './routes.js';
import {isNodeCapability} from './client-nodes.js';

export const CLIENT_ENTRY = '\0linguini-kit-client-entry.js';
const CAPABILITIES = '\0linguini-kit-client-capabilities.js';

/** Resolve optional runtime exports after loading the real graph, before tree shaking. */
export function clientEntry(prepare: (context: Rollup.PluginContext) => Promise<string>): Plugin {
  let config: ResolvedConfig;
  let application: string | undefined;
  return {
    name:'linguini-sveltekit-client-entry',
    enforce:'pre',
    apply:'build',
    configResolved(value) {config=value;},
    options(options) {
      if (config.build.ssr || !options.input || typeof options.input !== 'object' || Array.isArray(options.input)) return;
      const input = {...options.input};
      const entries = Object.entries(input).filter(([,id]) => normalizePath(id).endsWith('/generated/client-optimized/app.js'));
      if (entries.length !== 1) throw new Error('Linguini requires one SvelteKit client application entry');
      const [name,id] = entries[0];
      application = path.resolve(config.root,id);
      input[name] = CLIENT_ENTRY;
      return {...options,input};
    },
    resolveId(id) {
      if (id === CLIENT_ENTRY || id === CAPABILITIES) return id;
    },
    async load(id) {
      if (!application) return;
      if (id === CLIENT_ENTRY) {
        const source = '/@fs/' + normalizePath(application);
        return `export * from ${JSON.stringify(source)}; export * from ${JSON.stringify(CAPABILITIES)};`;
      }
      if (id !== CAPABILITIES) return;
      await loadGraph(this,application);
      return prepare(this);
    },
    generateBundle: {
      order:'post',
      handler(_options,bundle) {
        const entry = Object.values(bundle).find(item => item.type === 'chunk' && item.facadeModuleId === CLIENT_ENTRY);
        if (!entry || !application) return;
        const manifestPath = typeof config.build.manifest === 'string' ? config.build.manifest : '.vite/manifest.json';
        const asset = bundle[manifestPath];
        if (asset?.type !== 'asset') throw new Error('Linguini requires the SvelteKit client manifest');
        const source = typeof asset.source === 'string' ? asset.source : new TextDecoder().decode(asset.source);
        const manifest = JSON.parse(source) as Record<string,{file:string;src?:string;isEntry?:boolean;imports?:string[];dynamicImports?:string[]}>;
        const matches = Object.entries(manifest).filter(([,value]) => value.file === entry.fileName && value.isEntry);
        if (matches.length !== 1) throw new Error('Linguini cannot identify its SvelteKit application facade');
        const [previous,value] = matches[0];
        const original = normalizePath(path.relative(config.root,application));
        if (original !== previous && Object.hasOwn(manifest,original)) throw new Error('Conflicting SvelteKit application manifest entry');
        delete manifest[previous];
        manifest[original] = {...value,src:original};
        for (const item of Object.values(manifest)) {
          for (const key of ['imports','dynamicImports'] as const) {
            if (item[key]) item[key] = item[key].map(id => id === previous ? original : id);
          }
        }
        asset.source = JSON.stringify(manifest,null,2);
      },
    },
  };
}

async function loadGraph(context: Rollup.PluginContext,entry: string): Promise<void> {
  const visited = new Set([CLIENT_ENTRY,CAPABILITIES]);
  let frontier = [entry];
  while (frontier.length) {
    const batch = [...new Set(frontier.filter(id => !visited.has(id) && !isMessageModule(id) && !isNodeCapability(id)))];
    frontier = [];
    for (const id of batch) visited.add(id);
    await Promise.all(batch.map(id => context.load({id,resolveDependencies:true})));
    for (const id of batch) {
      const info = context.getModuleInfo(id);
      if (!info) throw new Error(`Missing SvelteKit client module ${id}`);
      for (const dependency of [...info.importedIds,...info.dynamicallyImportedIds]) {
        if (visited.has(dependency)) continue;
        if (isMessageModule(dependency) || isNodeCapability(dependency)) continue;
        const module = context.getModuleInfo(dependency);
        if (module && 'isExternal' in module && module.isExternal) continue;
        if (typeof module?.code !== 'string') {
          const resolved = await context.resolve(dependency,id,{skipSelf:false});
          if (!resolved) throw new Error(`Cannot resolve SvelteKit client dependency ${dependency}`);
          if (resolved.external) continue;
        }
        frontier.push(dependency);
      }
    }
  }
}
