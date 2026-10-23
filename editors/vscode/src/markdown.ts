export interface LinguiniFence {
  language: 'linguini-schema' | 'linguini-locale';
  source: string;
  startLine: number;
  indentation: number[];
}

/** Locate Markdown fences only; Linguini syntax is always classified by Rust. */
export function linguiniFences(source: string): LinguiniFence[] {
  const lines = source.split(/\r\n|\r|\n/);
  const result: LinguiniFence[] = [];
  let open: { marker: string; width: number; indent: number; start: number; language?: LinguiniFence['language'] } | undefined;
  const finish = (end: number) => {
    if (!open?.language) return;
    const indentation = lines.slice(open.start, end).map(line => Math.min(open!.indent, /^ */.exec(line)![0].length));
    result.push({
      language: open.language,
      startLine: open.start,
      source: lines.slice(open.start, end).map((line, index) => line.slice(indentation[index])).join('\n'),
      indentation
    });
  };
  for (let index = 0; index < lines.length; index++) {
    if (open) {
      const close = /^ {0,3}(`{3,}|~{3,})[ \t]*$/.exec(lines[index]);
      if (close && close[1][0] === open.marker && close[1].length >= open.width) {
        finish(index);
        open = undefined;
      }
      continue;
    }
    const fence = /^( {0,3})(`{3,}|~{3,})(.*)$/.exec(lines[index]);
    if (!fence || (fence[2][0] === '`' && fence[3].includes('`'))) continue;
    const alias = fence[3].trim().split(/\s+/)[0].toLowerCase();
    const language = ['lgs', 'linguini-schema'].includes(alias) ? 'linguini-schema'
      : ['lgl', 'linguini-locale'].includes(alias) ? 'linguini-locale' : undefined;
    open = { marker: fence[2][0], width: fence[2].length, indent: fence[1].length, start: index + 1, language };
  }
  finish(lines.length);
  return result;
}
