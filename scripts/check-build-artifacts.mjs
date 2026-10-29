import {spawnSync} from 'node:child_process';
import {existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

export function isBuildArtifact(path) {
  return /^(?:site\/(?:generated|build|\.svelte-kit|src\/lib\/generated)|packages\/(?:web|svelte|sveltekit)\/dist|packages\/cli-[^/]+\/bin|editors\/vscode\/dist|target|release)\//.test(path);
}

export function main(root = fileURLToPath(new URL('../',import.meta.url))) {
  const result = spawnSync('git',['ls-files','-z'],{cwd:root,encoding:'utf8'});
  if (result.error || result.status !== 0) throw result.error ?? new Error(result.stderr);
  const artifacts = result.stdout.split('\0').filter(path => isBuildArtifact(path) && existsSync(resolve(root,path)));
  if (artifacts.length) throw new Error(`Build output must not be tracked in Git:\n${artifacts.join('\n')}`);
  console.log('Verified application build output is absent from tracked files.');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
