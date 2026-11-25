import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {runPnpm} from '../../scripts/pnpm-command.mjs';

test('package tests launch script and standalone package managers without losing arguments or failures', context => {
  const root = mkdtempSync(path.join(tmpdir(),'linguini-package-manager-'));
  const previous = process.env.npm_execpath;
  context.after(() => {
    if (previous === undefined) delete process.env.npm_execpath;
    else process.env.npm_execpath = previous;
    rmSync(root,{recursive:true,force:true});
  });
  const file = path.join(root,'package manager.cjs');
  writeFileSync(file,'console.log(JSON.stringify(process.argv.slice(2))); console.error("expected failure"); process.exitCode = 7;');
  const args = ['a b','"quoted"','`literal`','$(literal)'];
  for (const standalone of [false,true]) {
    process.env.npm_execpath = standalone ? process.execPath : file;
    const result = runPnpm(standalone ? [file,...args] : args,root);
    assert.equal(result.error,undefined);
    assert.equal(result.status,7);
    assert.deepEqual(JSON.parse(result.stdout),args);
    assert.equal(result.stderr.trim(),'expected failure');
  }
  process.env.npm_execpath = path.join(root,'missing-manager.exe');
  assert.throws(() => runPnpm([],root),{code:'ENOENT'});
});
