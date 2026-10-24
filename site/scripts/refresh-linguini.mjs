import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const result = spawnSync('cargo', ['run', '--locked', '-p', 'linguini-cli', '--', 'build', '--svelte', '--svelte-host-module', '@antepodeum/linguini-sveltekit/client'], {
  cwd: root, stdio: 'inherit'
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
