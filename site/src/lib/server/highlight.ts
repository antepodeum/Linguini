import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { examples } from './examples.mjs';
import { examplesHash } from './highlight-render.mjs';

type CodeBlocks = Record<keyof typeof examples, string>;

export async function highlightedExamples(): Promise<CodeBlocks> {
  const artifact = JSON.parse(await readFile(resolve('.svelte-kit/linguini/highlights.json'), 'utf8'));
  if (artifact.version !== 1 || artifact.sourceHash !== examplesHash(examples)) {
    throw new Error('Site highlights are stale. Run pnpm highlight:build.');
  }
  return artifact.codeBlocks;
}
