import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { readModuleConfiguration } from '../../packages/project/src/modules.js';

const root = fileURLToPath(new URL('../', import.meta.url));
const {plan} = await readModuleConfiguration(root);
const result = spawnSync('cargo', ['run', '--locked', '-p', 'linguini-cli', '--', 'build', '--module-plan-stdin'], {
  cwd: root, input: JSON.stringify(plan), stdio: ['pipe', 'inherit', 'inherit']
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
