import { highlightedExamples } from '$lib/server/highlight';

export async function load() {
  return { codeBlocks: await highlightedExamples() };
}
