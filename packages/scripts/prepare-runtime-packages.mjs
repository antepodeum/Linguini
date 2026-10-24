import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

// Repository setup only. Published packages already contain dist/ output.
// Build dependencies before pnpm snapshots them into the next local package.
for (const name of ['web', 'svelte', 'sveltekit']) {
  const cwd = fileURLToPath(new URL(`../${name}/`, import.meta.url));
  for (const args of [['install', '--frozen-lockfile'], ['build']]) {
    const result = spawnSync('pnpm', args, {cwd, stdio: 'inherit', shell: process.platform === 'win32'});
    if (result.error) throw result.error;
    if (result.status !== 0) process.exit(result.status ?? 1);
  }
}
