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
  generatedModulePatterns?: string[];
  build?: (context: LinguiniBuildContext) => void | Promise<void>;
}

export declare function linguini(options?: LinguiniViteOptions): Plugin;
export default linguini;

export declare function discoverLinguiniFiles(
  root: string,
  configFile?: string
): Promise<string[]>;

export declare function isLinguiniSource(
  file: string,
  root?: string,
  configFile?: string
): boolean;

export type { ViteDevServer };
