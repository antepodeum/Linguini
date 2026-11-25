interface OpeningTag {
  start: number;
  end: number;
  name: string;
  bodyEnd: number;
}

/** Locate tag boundaries while leaving script/style/preprocessor contents untouched. */
export function* openingTags(code: string): Generator<OpeningTag> {
  let cursor = 0;
  while (cursor < code.length) {
    const start = code.indexOf("<", cursor);
    if (start === -1) return;
    if (code.startsWith("<!--", start)) {
      const close = code.indexOf("-->", start + 4);
      cursor = close === -1 ? code.length : close + 3;
      continue;
    }
    const match = /^<\s*([A-Za-z][\w:-]*)\b/.exec(code.slice(start));
    if (!match) {
      cursor = start + 1;
      continue;
    }
    const name = match[1].toLowerCase();
    const end = openingTagEnd(code, start + match[0].length);
    if (end === -1) return;
    let bodyEnd = -1;
    cursor = end + 1;
    if (name === "script" || name === "style") {
      const closing = name === "script" ? /<\/script\s*>/gi : /<\/style\s*>/gi;
      closing.lastIndex = cursor;
      const close = closing.exec(code);
      bodyEnd = close?.index ?? -1;
      cursor = close ? close.index + close[0].length : code.length;
    }
    yield { start, end, name, bodyEnd };
  }
}

function openingTagEnd(code: string, start: number): number {
  let quote: string | undefined;
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

export function instanceScript(code: string): { insertion: number } | undefined {
  for (const { start, end, name } of openingTags(code)) {
    if (name === "script" && !isSvelteModuleScript(code.slice(start, end + 1))) {
      return { insertion: end + 1 };
    }
  }
  return undefined;
}

function isSvelteModuleScript(openingTag: string): boolean {
  const attributes = /([^\s=<>/]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/g;
  for (const match of openingTag.matchAll(attributes)) {
    const name = match[1].toLowerCase();
    const value = match[2] ?? match[3] ?? match[4];
    if (name === "module" || name === "context" && value === "module") return true;
  }
  return false;
}
