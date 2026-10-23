import type { ModuleDefinition } from "../modules.js";

export interface WebModuleOptions {
  readonly routing?: {
    readonly localePrefix?: "always" | "except-default" | "never";
    readonly canonical?: "redirect" | "preserve";
  };
  readonly locale?: { readonly sources?: readonly ("path" | "cookie" | "local-storage" | "accept-language")[] };
  readonly cookie?: {
    readonly name?: string;
    readonly path?: string;
    readonly domain?: string;
    readonly maxAge?: string;
    readonly sameSite?: "lax" | "strict" | "none";
    readonly secure?: boolean | "auto";
    readonly httpOnly?: boolean;
  };
  readonly localStorage?: { readonly key?: string };
  readonly links?: { readonly mode?: "transform" | "runtime" | "manual" };
  readonly routes?: { readonly exclude?: readonly string[] };
  readonly switchRoute?: { readonly path?: string; readonly returnQuery?: string; readonly status?: 301 | 302 | 303 | 307 | 308 };
}
export declare function web(options?: WebModuleOptions): ModuleDefinition<"ecmascript.web", WebModuleOptions>;
export declare function svelte(): ModuleDefinition<"svelte", Record<string, never>>;
export declare function svelteKit(): ModuleDefinition<"sveltekit", Record<string, never>>;
