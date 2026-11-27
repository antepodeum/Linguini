import {existsSync,mkdtempSync,readFileSync,readdirSync,realpathSync,renameSync,rmSync,writeFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {runPnpm} from './pnpm-command.mjs';

const PACKAGES_ROOT=fileURLToPath(new URL('../',import.meta.url));

function hasRepositoryDependencies(projectRoot,packagesRoot) {
  const manifest=JSON.parse(readFileSync(path.join(projectRoot,'package.json'),'utf8'));
  const declared=new Set([
    ...Object.keys(manifest.dependencies??{}),
    ...Object.keys(manifest.devDependencies??{}),
    ...Object.keys(manifest.optionalDependencies??{}),
  ]);
  for (const entry of readdirSync(packagesRoot,{withFileTypes:true})) {
    if (!entry.isDirectory()) continue;
    const root=path.join(packagesRoot,entry.name);
    const file=path.join(root,'package.json');
    if (!existsSync(file)) continue;
    const {name}=JSON.parse(readFileSync(file,'utf8'));
    if (declared.has(name)) return true;
  }
  return false;
}

/** Install a repository project without changing its declared or locked dependency graph. */
export function installProjectDependencies(projectRoot,{packagesRoot=PACKAGES_ROOT,execute=runPnpm}={}) {
  projectRoot=realpathSync(projectRoot);
  packagesRoot=realpathSync(packagesRoot);
  const files=['package.json','pnpm-lock.yaml'];
  if (existsSync(path.join(projectRoot,'pnpm-workspace.yaml'))) files.push('pnpm-workspace.yaml');
  const metadata=files.map(name=>{
    const file=path.join(projectRoot,name);
    return {file,contents:readFileSync(file)};
  });
  function run(args) {
    const result=execute(args,projectRoot);
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    if (result.status!==0) throw new Error(`pnpm ${args[0]} failed for ${projectRoot} (exit ${result.status})`);
  }
  const modules=path.join(projectRoot,'node_modules');
  let backupRoot,previousModules,installed=false;
  try {
    // pnpm 11.1 reuses stale directory inventories, even with --force. A fresh
    // modules tree imports current local files using the frozen graph, without
    // re-resolving registry dependencies or requiring their metadata offline.
    // https://github.com/pnpm/pnpm/issues/11795
    if (hasRepositoryDependencies(projectRoot,packagesRoot)&&existsSync(modules)) {
      backupRoot=mkdtempSync(path.join(projectRoot,'.linguini-install-'));
      const backup=path.join(backupRoot,'node_modules');
      renameSync(modules,backup);
      previousModules=backup;
    }
    run(['install','--frozen-lockfile']);
    installed=true;
  } finally {
    const changed=metadata.filter(({file,contents})=>!existsSync(file)||!readFileSync(file).equals(contents));
    for (const {file,contents} of changed) writeFileSync(file,contents);
    if (previousModules&&(!installed||changed.length)) {
      rmSync(modules,{recursive:true,force:true});
      renameSync(previousModules,modules);
    }
    if (backupRoot) rmSync(backupRoot,{recursive:true,force:true});
    if (changed.length) throw new Error(`Repository installation would change locked metadata: ${changed.map(({file})=>file).join(', ')}. Update the dependency declarations and lockfile explicitly first.`);
  }
}

if (process.argv[1]&&existsSync(process.argv[1])&&realpathSync(fileURLToPath(import.meta.url))===realpathSync(process.argv[1])) {
  try {
    if (process.argv.length>3) throw new Error('Usage: node install-project-dependencies.mjs [project-directory]');
    installProjectDependencies(path.resolve(process.argv[2]??'.'));
  } catch (error) {
    console.error(error.message);
    process.exitCode=1;
  }
}
