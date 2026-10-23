import type { Plugin, ViteDevServer } from "vite";
import type MagicString from "magic-string";

export interface LinguiniStaticLink {
  readonly start: number;
  readonly end: number;
  readonly href: string;
}

/** Build-time syntax and lifecycle hooks; no framework is imported by the Vite core. */
export interface LinguiniFrameworkAdapter {
  readonly entry: string;
  readonly messageExports: readonly string[];
  readonly localeHelper: string;
  readonly effectsHelper?: string;
  readonly extensions?: readonly string[];
  applicationScope?(code: string, insertion: number, sourceName: string):
    { key: string; insertionByte: number } | undefined;
  findStaticLinks?(code: string, sourceName: string): readonly LinguiniStaticLink[];
  applyStaticLinks?(magic: MagicString, code: string, sourceName: string,
    generatedRoot: string, operations: readonly LinguiniStaticLink[]): void;
  /** Return true when the adapter owns cleanup; false uses module/HMR cleanup. */
  injectLocaleLoaderLifecycle?(magic: MagicString, code: string,
    application: { readonly relative: string },
    context: { readonly prelude: string; readonly release: string;
      allocateAlias(prefix: string): string }): boolean;
}

export interface LinguiniBuildContext {
  root: string;
  reason: "build-start" | "hot-update" | string;
}

export interface LinguiniViteOptions {
  root?: string;
  configFile?: string;
  command?: string;
  args?: string[];
  buildOnStart?: boolean;
  debounceMs?: number;
  generatedModulePatterns?: string[];
  adapter?: LinguiniFrameworkAdapter;
  build?: (context: LinguiniBuildContext) => void | Promise<void>;
}

export declare function linguini(options?: LinguiniViteOptions): Plugin;
export default linguini;

export declare function discoverLinguiniFiles(
  root: string,
  configFile?: string
): Promise<string[]>;

export type LinguiniProjectLayout = import("@antepodeum/linguini-project/ecmascript").EcmascriptProjectLayout;

export declare function readProjectLayout(
  root: string,
  configFile?: string
): Promise<LinguiniProjectLayout>;

export declare function isLinguiniSource(
  file: string,
  root?: string,
  configFile?: string,
  layout?: LinguiniProjectLayout
): boolean;

export type { ViteDevServer };
