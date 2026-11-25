import MagicString from 'magic-string';
import type {Plugin,ResolvedConfig,Rollup} from 'vite';

const BODY = '\0linguini-kit-node-body/';
const CAPABILITY = '\0linguini-kit-node-capability/';
export const SHARED_RUNTIME = '\0linguini-kit-deferred-runtime.js';

export function isNodeCapability(id:string): boolean {
  return id.startsWith(CAPABILITY);
}

/** Keep optional preparation code in the pages that actually expose lazy actions. */
export function clientNodes(capability:(context:Rollup.PluginContext,owner:string)=>Promise<string>,runtime:()=>string): Plugin {
  const bodies = new Map<string,{id:string;code:string}>();
  const owners = new Map<string,string>();
  let config: ResolvedConfig;
  return {
    name:'linguini-sveltekit-client-nodes',enforce:'pre',apply:'build',
    configResolved(value) {config=value;},
    buildStart() {bodies.clear();owners.clear();},
    resolveId(source,importer) {
      if (source === SHARED_RUNTIME || bodies.has(source) || owners.has(source)) return source;
      const body = importer && bodies.get(importer);
      if (body) return this.resolve(source,body.id,{skipSelf:true});
    },
    transform(code,id) {
      if (config.build.ssr || !/\/generated\/client-optimized\/nodes\/\d+\.js$/.test(id.replaceAll('\\','/'))) return;
      const key = Buffer.from(id).toString('base64url') + '.js';
      const body = BODY + key;
      const owner = CAPABILITY + key;
      bodies.set(body,{id,code});
      owners.set(owner,id);
      const names = nodeExports(this.parse(code));
      return {code:`export {${names.join(',')}} from ${JSON.stringify(body)}; export * from ${JSON.stringify(owner)};`,map:null};
    },
    load(id) {
      if (id === SHARED_RUNTIME) return runtime();
      const body = bodies.get(id);
      if (body) return {code:body.code,map:new MagicString(body.code).generateMap({source:body.id,includeContent:true,hires:true})};
      const owner = owners.get(id);
      if (owner) return capability(this,owner);
    },
  };
}

function nodeExports(ast: unknown): string[] {
  const program = ast as {body:Record<string,unknown>[]};
  const names: string[] = [];
  const add = (value:unknown) => {
    const identifier = value as {name?:unknown} | undefined;
    if (typeof identifier?.name !== 'string' || !/^[A-Za-z_$][\w$]*$/.test(identifier.name)) {
      throw new Error('Unsupported SvelteKit client node export');
    }
    names.push(identifier.name);
  };
  for (const statement of program.body) {
    if (statement.type === 'ExportAllDeclaration') add(statement.exported);
    else if (statement.type === 'ExportDefaultDeclaration') names.push('default');
    else if (statement.type === 'ExportNamedDeclaration') {
      for (const specifier of statement.specifiers as {exported:unknown}[]) add(specifier.exported);
      const declaration = statement.declaration as {type:string;id?:unknown;declarations?:{id:unknown}[]} | undefined;
      if (declaration?.type === 'VariableDeclaration') {
        for (const item of declaration.declarations ?? []) add(item.id);
      } else if (declaration) add(declaration.id);
    }
  }
  return [...new Set(names)];
}
