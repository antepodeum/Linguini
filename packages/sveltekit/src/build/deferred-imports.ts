import path from 'node:path';
import MagicString from 'magic-string';
import type {LinguiniPageBundles,LinguiniPage} from '@antepodeum/linguini-vite';
import type {Plugin,Rollup} from 'vite';
import {CLIENT_ENTRY} from './client-entry.js';

export const PREPARE_EXPORT = '__linguiniPrepare';
export const LOCALE_EXPORT = '__linguiniLocale';

/** Lower after shaking, preserving the host's namespace consumption and native live bindings. */
export function deferredImports(bundles: () => LinguiniPageBundles | undefined,host:(target:string)=>string = () => CLIENT_ENTRY,pages:()=>readonly LinguiniPage[] = () => []): Plugin {
  const observed = new Set<string>();
  return {
    name:'linguini-sveltekit-deferred-imports',
    apply:'build',
    buildStart() {observed.clear();},
    renderChunk(code,chunk,_options,meta) {
      const plan = bundles();
      if (!plan || !Object.keys(plan.deferred).length) return;
      for (const target of Object.keys(plan.deferred)) {
        const importers = this.getModuleInfo(target)?.dynamicImporters ?? [];
        if (Object.hasOwn(chunk.modules,target) && importers.some(id => Object.hasOwn(chunk.modules,id))) {
          throw new Error(`Linguini cannot yet prepare deferred messages through an inlined import of ${target}`);
        }
      }
      const allocate = allocator(code);
      const prepare = allocate('__linguiniPrepareBoundary');
      const locale = allocate('__linguiniBoundaryLocale');
      const magic = new MagicString(code);
      const imports = new Set<string>();
      const declarations: string[] = [];
      const directories = new Map<string,string>();
      const descriptors = new Map<string,string>();
      const roots = pages();
      const scopes = pageScopes(roots,chunk.fileName,meta.chunks,plan.deferredImports);
      const pending: unknown[] = [this.parse(code)];
      let changed = false;
      while (pending.length) {
        const value = pending.pop();
        if (!value || typeof value !== 'object') continue;
        const expression = value as Record<string,unknown>;
        if (expression.type === 'ImportExpression') {
          const source = expression.source as Record<string,unknown>;
          const file = typeof source?.value === 'string'
            ? path.posix.normalize(path.posix.join(path.posix.dirname(chunk.fileName),source.value)) : undefined;
          const dependency = file && meta.chunks[file];
          const target = dependency && dependency.facadeModuleId;
          if (target && plan.deferred[target] && typeof expression.start === 'number' && typeof expression.end === 'number') {
            const entry = Object.values(meta.chunks).find(item => item.facadeModuleId === host(target));
            if (!entry) throw new Error(`Missing SvelteKit preparation owner for ${target}`);
            if (roots.length && !scopes.size) throw new Error(`Linguini cannot identify the pages using deferred caller ${chunk.fileName}`);
            if ([...scopes].some(route => !Object.hasOwn(plan.deferred[target],route))) {
              throw new Error(`Linguini cannot yet prepare deferred messages from a caller shared with an unrelated page: ${target}`);
            }
            const routes = Object.entries(plan.deferred[target]).filter(([route]) => !roots.length || scopes.has(route)).map(([route,locales]) => {
              const file = Object.values(locales)[0]?.file;
              if (!file) throw new Error(`Missing deferred message payload for ${route}`);
              const directory = relative(chunk.fileName,path.posix.dirname(file)) + '/';
              let binding = directories.get(directory);
              if (!binding) {
                binding = allocate('__linguiniDirectory' + directories.size);
                directories.set(directory,binding);
                declarations.push(`const ${binding}=new URL(${JSON.stringify(directory)},import.meta.url).href;`);
              }
              return `${JSON.stringify(route)}:${binding}`;
            });
            const source = routes.join(',');
            let descriptor = descriptors.get(source);
            if (!descriptor) {
              descriptor = allocate('__linguiniBoundary' + descriptors.size);
              descriptors.set(source,descriptor);
              declarations.push(`const ${descriptor}={${source}};`);
            }
            const original = code.slice(expression.start,expression.end);
            magic.overwrite(expression.start,expression.end,`${prepare}(${descriptor},${locale}).then(() => ${original})`);
            imports.add(entry.fileName);
            observed.add(target);
            changed = true;
          }
        }
        for (const child of Object.values(expression)) {
          if (Array.isArray(child)) pending.push(...child);
          else if (child && typeof child === 'object') pending.push(child);
        }
      }
      if (!changed) return;
      if (imports.size !== 1) throw new Error('Linguini deferred callers require one preparation owner');
      const [file] = imports;
      magic.prepend(`import {${PREPARE_EXPORT} as ${prepare},${LOCALE_EXPORT} as ${locale}} from ${JSON.stringify(relative(chunk.fileName,file))};\n${declarations.join('\n')}\n`);
      // Rollup supports mutable import metadata; Rolldown's render metadata is read-only.
      const metadata = chunk as typeof chunk & {importedBindings?:Record<string,string[]>};
      if (metadata.importedBindings && Object.getOwnPropertyDescriptor(chunk,'imports')?.writable
        && Object.getOwnPropertyDescriptor(chunk,'importedBindings')?.writable) {
        chunk.imports = [...new Set([...chunk.imports,file])];
        metadata.importedBindings = {...metadata.importedBindings,[file]:[...new Set([
          ...(metadata.importedBindings[file] ?? []),PREPARE_EXPORT,LOCALE_EXPORT,
        ])]};
      }
      return {code:magic.toString(),map:magic.generateMap({hires:true})};
    },
    generateBundle() {
      for (const target of Object.keys(bundles()?.deferred ?? {})) {
        if (!observed.has(target)) throw new Error(`Linguini cannot lower deferred message preparation for ${target}`);
      }
    },
  };
}

function allocator(code:string): (prefix:string)=>string {
  const used = new Set<string>();
  return prefix => {
    let name = prefix;
    for (let index = 0; code.includes(name) || used.has(name); index++) name = prefix + index;
    used.add(name);
    return name;
  };
}

function relative(importer: string,target: string): string {
  const value = path.posix.relative(path.posix.dirname(importer),target);
  if (!value) return '.';
  return value.startsWith('.') ? value : './' + value;
}

function pageScopes(pages:readonly LinguiniPage[],file:string,chunks:Record<string,Rollup.RenderedChunk>,planned?:Record<string,readonly string[]>): Set<string> {
  const scopes = new Set<string>();
  for (const page of pages) {
    const roots = new Set(page.entries.map(entry => typeof entry === 'string' ? entry : entry.id));
    const pending = Object.values(chunks).filter(chunk => chunk.facadeModuleId && roots.has(chunk.facadeModuleId)).map(chunk => chunk.fileName);
    const visited = new Set<string>();
    const allowed = new Set(planned?.[page.id] ?? []);
    while (pending.length) {
      const current = pending.pop()!;
      if (visited.has(current)) continue;
      visited.add(current);
      if (current === file) {scopes.add(page.id);break;}
      for (const imported of chunks[current]?.imports ?? []) {
        const dependency = chunks[imported] ? imported : path.posix.normalize(path.posix.join(path.posix.dirname(current),imported));
        if (chunks[dependency]) pending.push(dependency);
      }
      for (const imported of chunks[current]?.dynamicImports ?? []) {
        const dependency = chunks[imported] ? imported : path.posix.normalize(path.posix.join(path.posix.dirname(current),imported));
        const target = chunks[dependency]?.facadeModuleId;
        if (target && allowed.has(target)) pending.push(dependency);
      }
    }
  }
  return scopes;
}
