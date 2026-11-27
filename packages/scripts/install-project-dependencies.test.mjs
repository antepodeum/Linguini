import assert from 'node:assert/strict';
import {existsSync,mkdirSync,mkdtempSync,readFileSync,readdirSync,rmSync,unlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import test from 'node:test';
import {installProjectDependencies} from './install-project-dependencies.mjs';
import {runPnpm} from './pnpm-command.mjs';

const packageManager=JSON.parse(readFileSync(new URL('../../package.json',import.meta.url),'utf8')).packageManager;

function fixture(context) {
  const root=mkdtempSync(path.join(tmpdir(),'linguini runtime install '));
  context.after(()=>rmSync(root,{recursive:true,force:true}));
  const packagesRoot=path.join(root,'packages');
  const runtime=path.join(packagesRoot,'runtime');
  const consumer=path.join(root,'consumer');
  mkdirSync(runtime,{recursive:true});
  mkdirSync(consumer);
  writeFileSync(path.join(runtime,'package.json'),JSON.stringify({name:'@linguini-fixture/runtime',version:'1.0.0',type:'module',files:['*.js']}));
  writeFileSync(path.join(runtime,'index.js'),'export const value="old";\n');
  writeFileSync(path.join(runtime,'removed.js'),'export const removed=true;\n');
  writeFileSync(path.join(consumer,'package.json'),JSON.stringify({name:'installer-consumer',private:true,type:'module',devDependencies:{'@linguini-fixture/runtime':'1.0.0'},packageManager}));
  writeFileSync(path.join(consumer,'pnpm-workspace.yaml'),"overrides:\n  '@linguini-fixture/runtime': file:../packages/runtime\n");
  return {root,packagesRoot,runtime,consumer};
}

function snapshot(root) {
  return new Map(['package.json','pnpm-lock.yaml','pnpm-workspace.yaml'].map(name=>[name,readFileSync(path.join(root,name))]));
}

function assertNoInstallBackup(consumer) {
  assert.equal(readdirSync(consumer).some(name=>name.startsWith('.linguini-install-')),false);
}

test('locked repository installs refresh added, replaced and deleted local files offline', async context=>{
  const {root,packagesRoot,runtime,consumer}=fixture(context);
  const execute=(args,cwd)=>runPnpm([...args,'--offline','--ignore-scripts','--store-dir',path.join(root,'store')],cwd);
  const initial=execute(['install'],consumer);
  assert.equal(initial.status,0,initial.stdout+initial.stderr);
  const before=snapshot(consumer);
  const installed=path.join(consumer,'node_modules/@linguini-fixture/runtime');
  // Simulate build tools replacing an existing output and adding/removing modules.
  unlinkSync(path.join(runtime,'index.js'));
  writeFileSync(path.join(runtime,'index.js'),'export {value} from "./new.js";\n');
  writeFileSync(path.join(runtime,'new.js'),'export const value="new";\n');
  unlinkSync(path.join(runtime,'removed.js'));
  const stale=execute(['install','--frozen-lockfile'],consumer);
  assert.equal(stale.status,0,stale.stdout+stale.stderr);
  const calls=[];
  installProjectDependencies(consumer,{packagesRoot,execute(args,cwd){
    calls.push(args);
    return execute(args,cwd);
  }});
  assert.deepEqual(calls,[
    ['install','--frozen-lockfile'],
  ]);
  assert.equal(existsSync(path.join(installed,'removed.js')),false);
  assert.equal((await import(pathToFileURL(path.join(installed,'index.js')).href)).value,'new');
  assert.deepEqual(snapshot(consumer),before);
  assertNoInstallBackup(consumer);
  // A subsequent cold install uses the same locked graph and current source files.
  rmSync(path.join(consumer,'node_modules'),{recursive:true,force:true});
  installProjectDependencies(consumer,{packagesRoot,execute});
  assert.ok(existsSync(path.join(installed,'new.js')));
  assert.equal(existsSync(path.join(installed,'removed.js')),false);
  assert.deepEqual(snapshot(consumer),before);
  assertNoInstallBackup(consumer);
});

test('metadata drift fails and restores declarations, lockfile and workspace policy', context=>{
  const {packagesRoot,consumer}=fixture(context);
  writeFileSync(path.join(consumer,'pnpm-lock.yaml'),'locked fixture\n');
  const before=snapshot(consumer);
  const modules=path.join(consumer,'node_modules');
  mkdirSync(modules);
  writeFileSync(path.join(modules,'previous.txt'),'previous installation');
  assert.throws(()=>installProjectDependencies(consumer,{packagesRoot,execute(args){
    assert.deepEqual(args,['install','--frozen-lockfile']);
    mkdirSync(modules);
    writeFileSync(path.join(modules,'new.txt'),'uncommitted installation');
    for (const name of ['package.json','pnpm-lock.yaml']) writeFileSync(path.join(consumer,name),'unexpected mutation\n');
    unlinkSync(path.join(consumer,'pnpm-workspace.yaml'));
    return {status:0};
  }}),/would change locked metadata/);
  assert.deepEqual(snapshot(consumer),before);
  assert.equal(readFileSync(path.join(modules,'previous.txt'),'utf8'),'previous installation');
  assert.equal(existsSync(path.join(modules,'new.txt')),false);
  assertNoInstallBackup(consumer);
});

test('locked local packages with registry dependencies install without registry metadata', context=>{
  const {root,packagesRoot,runtime,consumer}=fixture(context);
  const registry=path.join(root,'registry');
  mkdirSync(registry);
  writeFileSync(path.join(registry,'package.json'),JSON.stringify({name:'fixture-registry',version:'1.0.0',files:['index.js'],packageManager}));
  writeFileSync(path.join(registry,'index.js'),'exports.value="registry";\n');
  const packed=runPnpm(['pack','--pack-destination',root],registry);
  assert.equal(packed.status,0,packed.stdout+packed.stderr);
  const tarball='fixture-registry-1.0.0.tgz';
  assert.ok(existsSync(path.join(root,tarball)));
  const manifest=JSON.parse(readFileSync(path.join(runtime,'package.json'),'utf8'));
  manifest.dependencies={'fixture-registry':'1.0.0'};
  writeFileSync(path.join(runtime,'package.json'),JSON.stringify(manifest));
  // The lock already resolves the registry dependency to a tarball. Serving that locked
  // payload from a local file keeps this cold-cache regression entirely offline; no
  // registry packument has ever populated the isolated pnpm metadata cache.
  writeFileSync(path.join(consumer,'pnpm-lock.yaml'),`lockfileVersion: '9.0'
settings:
  autoInstallPeers: true
  excludeLinksFromLockfile: false
overrides:
  '@linguini-fixture/runtime': file:../packages/runtime
importers:
  .:
    devDependencies:
      '@linguini-fixture/runtime':
        specifier: file:../packages/runtime
        version: file:../packages/runtime
packages:
  '@linguini-fixture/runtime@file:../packages/runtime':
    resolution: {directory: ../packages/runtime, type: directory}
  fixture-registry@1.0.0:
    resolution: {tarball: file:../${tarball}}
snapshots:
  '@linguini-fixture/runtime@file:../packages/runtime':
    dependencies:
      fixture-registry: 1.0.0
  fixture-registry@1.0.0: {}
`);
  const before=snapshot(consumer);
  const cache=path.join(root,'empty-metadata');
  const install=()=>spawnSync(process.execPath,['--input-type=module','--eval',`
    import {installProjectDependencies} from ${JSON.stringify(new URL('./install-project-dependencies.mjs',import.meta.url).href)};
    installProjectDependencies(process.argv[1],{packagesRoot:process.argv[2]});
  `,consumer,packagesRoot],{encoding:'utf8',env:{...process.env,
    PNPM_CONFIG_CACHE_DIR:cache,PNPM_CONFIG_STORE_DIR:path.join(root,'store'),
    PNPM_CONFIG_OFFLINE:'true',PNPM_CONFIG_IGNORE_SCRIPTS:'true',
  }});
  for (const value of ['old','new']) {
    if (value==='new') {
      unlinkSync(path.join(runtime,'index.js'));
      writeFileSync(path.join(runtime,'index.js'),'export const value="new";\n');
    }
    const result=install();
    assert.equal(result.status,0,result.stdout+result.stderr);
    assert.equal(readFileSync(path.join(consumer,'node_modules/@linguini-fixture/runtime/index.js'),'utf8'),`export const value="${value}";\n`);
    assert.deepEqual(snapshot(consumer),before);
    assertNoInstallBackup(consumer);
    assert.equal(existsSync(path.join(cache,'v11/metadata/registry.npmjs.org/fixture-registry.jsonl')),false);
  }
});

test('a failed frozen install restores the previous installation', context=>{
  const {packagesRoot,consumer}=fixture(context);
  writeFileSync(path.join(consumer,'pnpm-lock.yaml'),'locked fixture\n');
  const before=snapshot(consumer);
  const modules=path.join(consumer,'node_modules');
  mkdirSync(modules);
  writeFileSync(path.join(modules,'previous.txt'),'previous installation');
  const calls=[];
  assert.throws(()=>installProjectDependencies(consumer,{packagesRoot,execute(args){
    calls.push(args);
    mkdirSync(modules);
    writeFileSync(path.join(modules,'partial.txt'),'partial installation');
    return {status:9};
  }}),/pnpm install failed.*exit 9/);
  assert.deepEqual(calls,[['install','--frozen-lockfile']]);
  assert.deepEqual(snapshot(consumer),before);
  assert.equal(readFileSync(path.join(modules,'previous.txt'),'utf8'),'previous installation');
  assert.equal(existsSync(path.join(modules,'partial.txt')),false);
  assertNoInstallBackup(consumer);
});

test('registry dependencies outside the repository catalog receive no update command', context=>{
  const {packagesRoot,consumer}=fixture(context);
  writeFileSync(path.join(consumer,'package.json'),JSON.stringify({private:true,devDependencies:{typescript:'6.0.3'}}));
  writeFileSync(path.join(consumer,'pnpm-lock.yaml'),'locked fixture\n');
  const before=snapshot(consumer);
  const modules=path.join(consumer,'node_modules');
  mkdirSync(modules);
  const marker=path.join(modules,'previous.txt');
  writeFileSync(marker,'registry installation');
  const calls=[];
  installProjectDependencies(consumer,{packagesRoot,execute(args){
    assert.equal(readFileSync(marker,'utf8'),'registry installation');
    calls.push(args);
    return {status:0};
  }});
  assert.deepEqual(calls,[['install','--frozen-lockfile']]);
  assert.deepEqual(snapshot(consumer),before);
  assertNoInstallBackup(consumer);
});
