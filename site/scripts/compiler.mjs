import { spawnSync } from 'node:child_process';

// Source-checkout builds use the workspace compiler. A supplied binary can replace Cargo.
export const compilerCommand = process.env.LINGUINI_COMMAND ?? 'cargo';
/** @param {string[]} args */
export function compilerArgs(args) {
  return process.env.LINGUINI_COMMAND ? args : [
    'run', '--offline', '--quiet', '--locked', '-p', 'linguini-cli', '--', ...args
  ];
}
/** @param {string[]} args @param {import('node:child_process').SpawnSyncOptions} options */
export function runCompiler(args, options) {
  const result = spawnSync(compilerCommand,compilerArgs(args),{...options,encoding:'utf8'});
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr || `Linguini exited with status ${result.status}`);
  return result;
}
