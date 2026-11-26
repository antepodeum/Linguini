import {fileURLToPath} from 'node:url';
import {installProjectDependencies} from './install-project-dependencies.mjs';
import {runPnpm} from './pnpm-command.mjs';

// Repository setup only. Published packages already contain dist/ output.
// Build dependencies before pnpm snapshots them into the next local package.
for (const name of ['web', 'svelte', 'sveltekit']) {
  const cwd = fileURLToPath(new URL(`../${name}/`, import.meta.url));
  installProjectDependencies(cwd);
  const result=runPnpm(['build'],cwd);
  process.stdout.write(result.stdout??'');
  process.stderr.write(result.stderr??'');
  if (result.status!==0) process.exit(result.status??1);
}
