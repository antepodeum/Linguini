import type { Plugin, ViteDevServer } from "vite";

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
  build?: (context: LinguiniBuildContext) => void | Promise<void>;
}

export declare function linguini(options?: LinguiniViteOptions): Plugin;
export default linguini;

export declare function discoverLinguiniFiles(
  root: string,
  configFile?: string
): Promise<string[]>;

export interface LinguiniProjectLayout {
  readonly projectRoot: string;
  readonly configPath: string;
  readonly configExists: boolean;
  readonly schemaRoot: string;
  readonly localeRoot: string;
  readonly generatedRoot: string;
  readonly bundlerSourceRoots: readonly string[];
  readonly bundlerExcludeRoots: readonly string[];
}

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
