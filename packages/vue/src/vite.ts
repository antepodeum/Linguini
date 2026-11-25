import path from "node:path";
import type { LinguiniFrameworkAdapter, LinguiniViteOptions, LinguiniVitePlugin } from "@antepodeum/linguini-vite";
import { parse } from "vue/compiler-sfc";
import { linguini as viteLinguini } from "@antepodeum/linguini-vite";

/** Vue owns SFC syntax; message planning and linking remain in the Vite core. */
export const vueAdapter: Readonly<LinguiniFrameworkAdapter> = Object.freeze({
  entry: "index.js",
  messageExports: Object.freeze(["lgl"]),
  extensions: Object.freeze([".vue"]),
  applicationScope(code: string, insertion: number, sourceName: string) {
    if (path.extname(sourceName).toLowerCase() !== ".vue") return undefined;
    const { descriptor, errors } = parse(code, { filename: sourceName, sourceMap: false });
    if (errors.length > 0) {
      throw new Error(`Cannot locate Linguini imports in ${sourceName}: ${errors.map(String).join("; ")}`);
    }
    for (const [key, block] of [["script", descriptor.script], ["setup", descriptor.scriptSetup]] as const) {
      if (block && insertion >= block.loc.start.offset && insertion < block.loc.end.offset) {
        return { key, insertionByte: Buffer.byteLength(code.slice(0, block.loc.start.offset)) };
      }
    }
    throw new Error(`Linguini import in ${sourceName} must be inside an inline script block`);
  },
});

export function linguini(options: Omit<LinguiniViteOptions, "adapter"> = {}): LinguiniVitePlugin {
  return viteLinguini({ ...options, adapter: vueAdapter });
}
export default linguini;
