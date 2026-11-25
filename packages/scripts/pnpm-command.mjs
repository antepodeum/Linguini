import {spawnSync} from 'node:child_process';

/** Preserve the selected pnpm, whether it is a script, a standalone binary, or a Windows shim. */
export function runPnpm(args,cwd) {
  const executable = process.env.npm_execpath ?? 'pnpm';
  const script = /\.[cm]?js$/i.test(executable);
  const result = spawnSync(script ? process.execPath : executable,script ? [executable,...args] : args,{
    cwd,encoding:'utf8',
    shell:!script && process.platform === 'win32' && (executable === 'pnpm' || /\.(?:cmd|bat)$/i.test(executable)),
  });
  if (result.error) throw result.error;
  return result;
}
