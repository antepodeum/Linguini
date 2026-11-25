import path from "node:path";
import type MagicString from "magic-string";
import type { LinguiniFrameworkAdapter, LinguiniViteOptions, LinguiniVitePlugin, LinguiniStaticLink } from "@antepodeum/linguini-vite";
import { linguini as viteLinguini } from "@antepodeum/linguini-vite";
import { openingTags, instanceScript } from "./vite/source.js";

/** Svelte syntax and component lifecycle, composed with the framework-neutral Vite core. */
export const svelteAdapter: Readonly<LinguiniFrameworkAdapter> = Object.freeze({
  entry: "svelte.js",
  entrySideEffects: false,
  messageExports: Object.freeze(["l", "messages"]),
  localeHelper: "svelte_locale",
  effectsHelper: "svelte_effects",
  extensions: Object.freeze([".svelte"]),
  applicationScope,
  findStaticLinks,
  applyStaticLinks,
  injectLocaleLoaderLifecycle,
});

export function linguini(options: Omit<LinguiniViteOptions, "adapter"> = {}): LinguiniVitePlugin {
  const args = [...(options.args ?? ["build"])];
  if (args.includes("build") && !args.includes("--svelte")) args.push("--svelte");
  return viteLinguini({ ...options, args, adapter: svelteAdapter });
}
export default linguini;

function applyStaticLinks(
  magic: MagicString, code: string, sourceName: string, generatedRoot: string,
  operations: readonly LinguiniStaticLink[], linkModule?: string,
): void {
  let aliasIndex = 0;
  let alias: string;
  do {
    alias = `__linguini_localize_href_${aliasIndex++}`;
  } while (code.includes(alias));
  for (const operation of operations) {
    magic.overwrite(
      operation.start,
      operation.end,
      `href={${alias}(${JSON.stringify(operation.href)})}`
    );
  }
  if (!linkModule) throw new Error("Static link transforms require a project linkModule");
  const helper = linkModule;
  const importLine = `import { localizeTransformedHref as ${alias} } from ${JSON.stringify(helper)};\n`;
  const instance = instanceScript(code);
  if (instance) {
    magic.prependLeft(instance.insertion, importLine);
  } else {
    magic.append(`\n<script>\n${importLine}</script>\n`);
  }
}

function findStaticLinks(code: string, sourceName: string): LinguiniStaticLink[] {
  if (path.extname(sourceName).toLowerCase() !== ".svelte") return [];
  const operations: LinguiniStaticLink[] = [];
  for (const { start, end, name } of openingTags(code)) {
    if (name === "a") {
      const opening = code.slice(start, end + 1);
      const href = /\bhref\s*=\s*(["'])(.*?)\1/is.exec(opening);
      if (href && !/[{}]/.test(href[2]) && !shouldSkipStaticAnchor(opening)) {
        const attributeOffset = href.index;
        operations.push({
          start: start + attributeOffset,
          end: start + attributeOffset + href[0].length,
          href: href[2]
        });
      }
    }
  }
  return operations;
}

function shouldSkipStaticAnchor(opening: string): boolean {
  if (/\sdownload(?:\s|=|\/?>)/i.test(opening)) return true;
  if (/\sdata-linguini-(?:ignore|no-localize)(?:\s|=|\/?>)/i.test(opening)) return true;
  const rel = /\srel\s*=\s*(["'])(.*?)\1/is.exec(opening)?.[2] ?? "";
  return rel.split(/\s+/).some((token) => token.toLowerCase() === "external");
}

function applicationScope(
  code: string, insertion: number, sourceName: string,
): { key: string; insertionByte: number } | undefined {
  if (path.extname(sourceName).toLowerCase() !== ".svelte") {
    return undefined;
  }
  for (const { start, end, name, bodyEnd } of openingTags(code)) {
    if (name === "script" && insertion > end && insertion <= bodyEnd) {
      return {
        key: `${start}:${bodyEnd}`,
        insertionByte: Buffer.byteLength(code.slice(0, end + 1), "utf8")
      };
    }
  }
  throw new Error(`Linguini import binding is outside a Svelte script in ${sourceName}`);
}

function injectLocaleLoaderLifecycle(
  magic: MagicString, code: string, application: { readonly relative: string },
  { prelude, release, allocateAlias }: Parameters<NonNullable<LinguiniFrameworkAdapter["injectLocaleLoaderLifecycle"]>>[3],
): boolean {
  if (path.extname(application.relative).toLowerCase() !== ".svelte") return false;
  const onDestroy = allocateAlias("__linguini_on_destroy_");
  const lifecycle =
    `import { onDestroy as ${onDestroy} } from "svelte";\n` +
    prelude + `${onDestroy}(${release});\n`;
  const instance = instanceScript(code);
  if (instance) {
    magic.prependLeft(instance.insertion, lifecycle);
  } else {
    magic.append(`\n<script>\n${lifecycle}</script>\n`);
  }
  return true;
}
