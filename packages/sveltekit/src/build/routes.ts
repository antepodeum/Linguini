import path from 'node:path';
import {readFileSync} from 'node:fs';
import { normalizePath } from 'vite';
import type { Rollup } from 'vite';
import type { LinguiniPage } from '@antepodeum/linguini-vite';

type Node = { type: string; [key: string]: unknown };
function node(value: unknown): Node {
  if (!value || typeof value !== 'object' || !('type' in value) || typeof value.type !== 'string') {
    throw new Error('Unsupported SvelteKit client route manifest syntax');
  }
  return value as Node;
}
function array(value: unknown): unknown[] {
  if (!Array.isArray(value)) throw new Error('Unsupported SvelteKit client route manifest array');
  return value;
}
function text(value: unknown): string {
  const expression = node(value);
  if (expression.type !== 'Literal' || typeof expression.value !== 'string') {
    throw new Error('Unsupported SvelteKit client route manifest string');
  }
  return expression.value;
}
function index(value: unknown): number {
  const expression = node(value);
  if (expression.type === 'Literal' && Number.isSafeInteger(expression.value) && Number(expression.value) >= 0) return Number(expression.value);
  if (expression.type === 'UnaryExpression' && expression.operator === '~') return index(expression.argument);
  throw new Error('Unsupported SvelteKit client route manifest node index');
}

/** Isolate Kit's generated manifest boundary; never reconstruct its layout/reset rules from filenames. */
export function clientPages(context: Rollup.PluginContext): LinguiniPage[] {
  const manifests = [...context.getModuleIds()].filter(id => id.replaceAll('\\', '/').endsWith('/generated/client-optimized/app.js'));
  if (manifests.length !== 1) throw new Error('Linguini requires one SvelteKit optimized client manifest');
  const id = manifests[0];
  const source = readFileSync(id,'utf8');
  const ast = node(context.parse(source));
  const declarations = new Map<string, Node>();
  for (const statement of array(ast.body)) {
    const exported = node(statement);
    if (exported.type !== 'ExportNamedDeclaration' || !exported.declaration) continue;
    const declaration = node(exported.declaration);
    if (declaration.type !== 'VariableDeclaration') continue;
    for (const item of array(declaration.declarations)) {
      const variable = node(item);
      const name = node(variable.id);
      if (name.type === 'Identifier' && typeof name.name === 'string') declarations.set(name.name, node(variable.init));
    }
  }
  const nodes = declarations.get('nodes');
  const dictionary = declarations.get('dictionary');
  if (nodes?.type !== 'ArrayExpression' || dictionary?.type !== 'ObjectExpression') {
    throw new Error('Unsupported SvelteKit client routing manifest; Linguini cannot determine exact page roots');
  }
  if (array(dictionary.properties).length === 0
    && [...context.getModuleIds()].some(id => id.startsWith('\0virtual:linguini/message/'))) {
    throw new Error('Linguini page payloads require SvelteKit client route descriptors; server-only route resolution is not supported yet');
  }
  const entries = array(nodes.elements).map(value => {
    const loader = node(value);
    const body = node(loader.body);
    if (loader.type !== 'ArrowFunctionExpression' || body.type !== 'ImportExpression') {
      throw new Error('Unsupported SvelteKit client node loader');
    }
    const source = text(body.source);
    return normalizePath(path.resolve(path.dirname(id), source.endsWith('.js') ? source : source + '.js'));
  });
  return array(dictionary.properties).map(value => {
    const property = node(value);
    const tuple = node(property.value);
    if (property.type !== 'Property' || property.computed || tuple.type !== 'ArrayExpression') {
      throw new Error('Unsupported SvelteKit client route descriptor');
    }
    const descriptor = array(tuple.elements);
    const layouts = descriptor[1] ? array(node(descriptor[1]).elements) : [];
    const selected = [0, ...layouts.filter(value => value !== null).map(index), index(descriptor[0])];
    return {
      id: text(property.key),
      entries: [...new Set(selected)].map(index => {
        if (!entries[index]) throw new Error(`SvelteKit client route references missing node ${index}`);
        const id = entries[index];
        const info = context.getModuleInfo(id);
        if (!info?.exports) throw new Error(`Missing SvelteKit client node exports ${id}`);
        const exports: string[][] = [];
        if (info.exports.includes('component')) exports.push(['component']);
        if (info.exports.includes('universal')) {
          const ast = node(context.parse(readFileSync(id,'utf8')));
          for (const value of array(ast.body)) {
            const imported = node(value);
            if (imported.type !== 'ImportDeclaration') continue;
            const universal = array(imported.specifiers).some(value => node(node(value).local).name === 'universal');
            if (!universal) continue;
            const source = normalizePath(path.resolve(path.dirname(id),text(imported.source)));
            const exported = context.getModuleInfo(source)?.exports;
            if (!exported) throw new Error(`Missing SvelteKit universal exports ${source}`);
            if (exported.includes('load')) exports.push(['universal','load']);
          }
        }
        return {id,exports};
      })
    };
  });
}
