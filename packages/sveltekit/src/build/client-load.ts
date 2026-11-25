import {fileURLToPath} from 'node:url';
import {normalizePath} from 'vite';
import type {Plugin,ResolvedConfig} from 'vite';

export const CLIENT_LOAD = '\0linguini-kit-client-load.js';
// Keep the real loader's promise and an opaque result while demand is being planned.
export const CLIENT_LOAD_MODEL = 'export function createClientLoad() {return async event => globalThis.__linguiniAnalysisLoad(event);}';

/** Select optional navigation work after page demand is known, before tree shaking. */
export function clientLoad(deferred:()=>Promise<boolean>): Plugin {
  let config: ResolvedConfig;
  return {
    name:'linguini-sveltekit-client-load',enforce:'pre',apply:'build',
    configResolved(value) {config=value;},
    resolveId(source) {
      if (source === CLIENT_LOAD) return source;
      if (!config.build.ssr && source === '@antepodeum/linguini-sveltekit/load') return CLIENT_LOAD;
    },
    async load(id) {
      if (id !== CLIENT_LOAD) return;
      const source = await deferred() ? '../deferred-load.js' : '../load.js';
      const file = normalizePath(fileURLToPath(new URL(source,import.meta.url)));
      return `export {createClientLoad} from ${JSON.stringify(file)};`;
    },
  };
}
