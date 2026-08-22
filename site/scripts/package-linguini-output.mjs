import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { packageOutput } from './linguini-output.mjs';

const siteRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const manifest = packageOutput(
	join(siteRoot, 'src/lib/generated/linguini'),
	join(siteRoot, 'generated/linguini-output.br'),
	join(siteRoot, 'generated/linguini-output.json')
);
console.log(`Packaged ${manifest.files} generated Linguini files (${manifest.sha256}).`);
