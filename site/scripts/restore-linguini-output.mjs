import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { restoreOutput } from './linguini-output.mjs';

const siteRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const manifest = restoreOutput(
	join(siteRoot, 'generated/linguini-output.br'),
	join(siteRoot, 'generated/linguini-output.json'),
	join(siteRoot, 'src/lib/generated/linguini'),
	join(siteRoot, 'src/lib')
);
console.log(`Restored ${manifest.files} generated Linguini files (${manifest.sha256}).`);
