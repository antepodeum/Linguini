import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHighlighterCore } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import svelteGrammar from 'shiki/langs/svelte.mjs';
import darkPlusTheme from 'shiki/themes/dark-plus.mjs';
import { examples } from '../src/lib/server/examples.mjs';
import { examplesHash, renderHighlight } from '../src/lib/server/highlight-render.mjs';

const siteRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = join(siteRoot, '.svelte-kit/linguini');
mkdirSync(output, { recursive: true });
const native = JSON.parse(readFileSync(join(output, 'native-highlights.json'), 'utf8'));
if (native.version !== 1 || native.sourceHash !== examplesHash(examples)) {
  throw new Error('Native highlight artifacts are stale. Run pnpm highlight:build.');
}
const shiki = await createHighlighterCore({
  engine: createJavaScriptRegexEngine(), themes: [darkPlusTheme], langs: [svelteGrammar]
});
const codeBlocks = {};
try {
  for (const [name, example] of Object.entries(examples)) {
    if (example.language === 'svelte') {
      codeBlocks[name] = shiki.codeToHtml(example.source, { lang: 'svelte', theme: 'dark-plus' });
      continue;
    }
    codeBlocks[name] = renderHighlight(example.source, native.highlights[name]);
  }
  writeFileSync(join(output, 'highlights.json'), JSON.stringify({ version: 1, sourceHash: examplesHash(examples), codeBlocks }));
} finally {
  shiki.dispose();
}
console.log('Rendered site examples from shared Rust highlight artifacts.');
