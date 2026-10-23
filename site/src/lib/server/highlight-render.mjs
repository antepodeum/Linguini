import { createHash } from 'node:crypto';

/** @param {unknown} examples */
export function examplesHash(examples) {
  return createHash('sha256').update(JSON.stringify(examples)).digest('hex');
}

/** @param {string} value */
function escapeHtml(value) {
  return value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;').replaceAll("'", '&#39;');
}

// Presentation only. Language classification belongs to the Rust pipeline.
const colors = /** @type {Record<string, string>} */ ({
  keyword: '#569CD6', variable: '#9CDCFE', enum: '#4EC9B0', enumMember: '#4FC1FF',
  string: '#CE9178', comment: '#6A9955', operator: '#D4D4D4', function: '#DCDCAA',
  type: '#4EC9B0', parameter: '#9CDCFE'
});

/**
 * @param {string} source
 * @param {{version: number, tokens: Array<{line: number, start: number, length: number, type: string}>}} highlight
 */
export function renderHighlight(source, highlight) {
  if (highlight.version !== 1) throw new Error('Unsupported native highlight version');
  const lines = source.split(/\r\n|\r|\n/);
  const byLine = new Map();
  for (const token of highlight.tokens) {
    const line = lines[token.line];
    if (!line || token.start < 0 || token.length <= 0 || token.start + token.length > line.length || !colors[token.type]) {
      throw new Error('Invalid native highlight span');
    }
    const tokens = byLine.get(token.line) ?? [];
    tokens.push(token);
    byLine.set(token.line, tokens);
  }
  const html = lines.map((line, index) => {
    let cursor = 0;
    let rendered = '';
    for (const token of byLine.get(index) ?? []) {
      if (token.start < cursor) throw new Error('Overlapping native highlight spans');
      rendered += escapeHtml(line.slice(cursor, token.start));
      rendered += `<span style="color:${colors[token.type]}">${escapeHtml(line.slice(token.start, token.start + token.length))}</span>`;
      cursor = token.start + token.length;
    }
    return rendered + escapeHtml(line.slice(cursor));
  }).join('\n');
  return `<pre class="shiki dark-plus" style="background-color:#1E1E1E;color:#D4D4D4"><code>${html}</code></pre>`;
}
