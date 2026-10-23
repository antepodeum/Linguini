import path from "node:path";
import { linguini as viteLinguini } from "./index.js";
import { toVitePath } from "./paths.js";

/** Svelte syntax and component lifecycle, composed with the framework-neutral Vite core. */
export const svelteAdapter = Object.freeze({
  entry: "svelte.js",
  messageExports: Object.freeze(["l", "messages"]),
  localeHelper: "svelte_locale",
  effectsHelper: "svelte_effects",
  extensions: Object.freeze([".svelte"]),
  applicationScope,
  findStaticLinks,
  applyStaticLinks,
  injectLocaleLoaderLifecycle,
});

export function linguini(options = {}) {
  return viteLinguini({ ...options, adapter: svelteAdapter });
}
export default linguini;

function applyStaticLinks(magic, code, sourceName, generatedRoot, operations) {
  let aliasIndex = 0;
  let alias;
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
  const helper = toVitePath(path.resolve(generatedRoot, "web/link-transform.js"));
  const importLine = `import { localizeTransformedHref as ${alias} } from ${JSON.stringify(helper)};\n`;
  const instance = findSvelteInstanceScript(code);
  if (instance) {
    magic.prependLeft(instance.insertion, importLine);
  } else {
    magic.append(`\n<script>\n${importLine}</script>\n`);
  }
}

function findStaticLinks(code, sourceName) {
  if (path.extname(sourceName).toLowerCase() !== ".svelte") return [];
  const operations = [];
  let cursor = 0;
  while (cursor < code.length) {
    const start = code.indexOf("<", cursor);
    if (start === -1) break;
    if (code.startsWith("<!--", start)) {
      const end = code.indexOf("-->", start + 4);
      cursor = end === -1 ? code.length : end + 3;
      continue;
    }
    const nameMatch = /^<\s*([A-Za-z][\w:-]*)\b/.exec(code.slice(start));
    if (!nameMatch) {
      cursor = start + 1;
      continue;
    }
    const tagName = nameMatch[1].toLowerCase();
    const end = findSvelteOpeningTagEnd(code, start + nameMatch[0].length);
    if (end === -1) break;
    if (tagName === "script" || tagName === "style") {
      const close = code.toLowerCase().indexOf(`</${tagName}`, end + 1);
      cursor = close === -1 ? code.length : close + tagName.length + 3;
      continue;
    }
    if (tagName === "a") {
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
    cursor = end + 1;
  }
  return operations;
}

function findSvelteOpeningTagEnd(code, start) {
  let quote;
  let braces = 0;
  for (let cursor = start; cursor < code.length; cursor += 1) {
    const character = code[cursor];
    if (quote) {
      if (character === "\\") cursor += 1;
      else if (character === quote) quote = undefined;
      continue;
    }
    if (character === '"' || character === "'") {
      quote = character;
    } else if (character === "{") {
      braces += 1;
    } else if (character === "}" && braces > 0) {
      braces -= 1;
    } else if (character === ">" && braces === 0) {
      return cursor;
    }
  }
  return -1;
}

function shouldSkipStaticAnchor(opening) {
  if (/\sdownload(?:\s|=|\/?>)/i.test(opening)) return true;
  if (/\sdata-linguini-(?:ignore|no-localize)(?:\s|=|\/?>)/i.test(opening)) return true;
  const rel = /\srel\s*=\s*(["'])(.*?)\1/is.exec(opening)?.[2] ?? "";
  return rel.split(/\s+/).some((token) => token.toLowerCase() === "external");
}

function applicationScope(code, insertion, sourceName) {
  if (path.extname(sourceName).toLowerCase() !== ".svelte") {
    return undefined;
  }
  const openPattern = /<script\b[^>]*>/gi;
  let match;
  while ((match = openPattern.exec(code))) {
    const close = code.indexOf("</script>", openPattern.lastIndex);
    if (close === -1) {
      break;
    }
    if (insertion >= openPattern.lastIndex && insertion <= close) {
      return {
        key: `${match.index}:${close}`,
        insertionByte: Buffer.byteLength(code.slice(0, openPattern.lastIndex), "utf8")
      };
    }
    openPattern.lastIndex = close + "</script>".length;
  }
  throw new Error(`Linguini import binding is outside a Svelte script in ${sourceName}`);
}

function injectLocaleLoaderLifecycle(magic, code, application, { prelude, release, allocateAlias }) {
  if (path.extname(application.relative).toLowerCase() !== ".svelte") return false;
  const onDestroy = allocateAlias("__linguini_on_destroy_");
  const lifecycle =
    `import { onDestroy as ${onDestroy} } from "svelte";\n` +
    prelude + `${onDestroy}(${release});\n`;
  const instance = findSvelteInstanceScript(code);
  if (instance) {
    magic.prependLeft(instance.insertion, lifecycle);
  } else {
    magic.append(`\n<script>\n${lifecycle}</script>\n`);
  }
  return true;
}

function findSvelteInstanceScript(code) {
  const pattern = /<script\b[^>]*>/gi;
  let match;
  while ((match = pattern.exec(code))) {
    if (!isSvelteModuleScript(match[0])) {
      return { insertion: pattern.lastIndex };
    }
    const close = code.indexOf("</script>", pattern.lastIndex);
    if (close === -1) break;
    pattern.lastIndex = close + "</script>".length;
  }
  return undefined;
}

function isSvelteModuleScript(openingTag) {
  return (
    /\scontext\s*=\s*(["'])module\1/i.test(openingTag) ||
    /\smodule(?:\s|>|=)/i.test(openingTag)
  );
}
