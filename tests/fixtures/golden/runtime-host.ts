//! Execute generated modules against explicit host boundaries without filesystem rewrites.

const transpiler = new Bun.Transpiler({ loader: "ts", target: "browser" });
const hostKey = Symbol.for("linguini.generated.runtime.host");

export type RuntimeHost = { modules: Record<string, unknown>; meta?: { hot?: { dispose(callback: () => void): void } } };

export async function withRuntimeHost<T>(source: string, target: "javascript" | "typescript", host: RuntimeHost, run: (module: any) => T | Promise<T>): Promise<T> {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, hostKey);
  Reflect.set(globalThis, hostKey, host);
  // Bun folds import.meta.hot away; inject the explicit HMR host before stripping test-only TS.
  source = source.replaceAll("import.meta", "(host.meta ?? {})");
  let javascript = target === "typescript" ? transpiler.transformSync(source) : source;
  javascript = javascript
    .replace(/import\s+\*\s+as\s+(\w+)\s+from\s+(["'])([^"']+)\2\s*;/g, (_match, name, _quote, path) => `const ${name} = host.modules[${JSON.stringify(path)}];`)
    .replace(/import\s+\{([^}]+)\}\s+from\s+(["'])([^"']+)\2\s*;/g, (_match, names, _quote, path) => `const { ${names.replace(/\s+as\s+/g, ": ")} } = host.modules[${JSON.stringify(path)}];`);
  try {
    const wrapped = `const host = globalThis[Symbol.for(${JSON.stringify(Symbol.keyFor(hostKey))})];\n${javascript}`;
    const url = URL.createObjectURL(new Blob([wrapped], { type: "text/javascript" }));
    try {
      return await run(await import(url));
    } finally {
      URL.revokeObjectURL(url);
    }
  } finally {
    if (descriptor) Object.defineProperty(globalThis, hostKey, descriptor);
    else Reflect.deleteProperty(globalThis, hostKey);
  }
}

export async function withGlobals<T>(values: Record<string, unknown>, run: () => T | Promise<T>): Promise<T> {
  const descriptors = Object.keys(values).map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)] as const);
  for (const [key, value] of Object.entries(values)) {
    if (value === undefined) Reflect.deleteProperty(globalThis, key);
    else Object.defineProperty(globalThis, key, { configurable: true, value });
  }
  try { return await run(); }
  finally {
    for (const [key, descriptor] of descriptors) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  }
}
