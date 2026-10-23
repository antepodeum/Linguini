import type { Plugin } from "vite";
import type { LinguiniFrameworkAdapter, LinguiniViteOptions } from "./index.js";

export declare const svelteAdapter: Readonly<LinguiniFrameworkAdapter>;
export declare function linguini(options?: Omit<LinguiniViteOptions, "adapter">): Plugin;
export default linguini;
