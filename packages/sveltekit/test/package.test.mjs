import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';

function pnpm(args,cwd) {
  const result=process.env.npm_execpath
    ?spawnSync(process.execPath,[process.env.npm_execpath,...args],{cwd,encoding:'utf8'})
    :spawnSync('pnpm',args,{cwd,encoding:'utf8'});
  assert.equal(result.status,0,result.stderr+result.stdout);
  return result.stdout;
}

test('published runtime packages preserve strict TypeScript, declarations/maps, and metadata-only hooks', context=>{
  const root=mkdtempSync(path.join(tmpdir(),'linguini-runtime-packages-'));
  context.after(()=>rmSync(root,{recursive:true,force:true}));
  const packageRoot=fileURLToPath(new URL('../',import.meta.url));
  const webRoot=fileURLToPath(new URL('../../web/',import.meta.url));
  const store=pnpm(['store','path'],packageRoot).trim();
  const web=path.join(root,'web.tgz'),kit=path.join(root,'kit.tgz'),svelte=path.join(root,'svelte.tgz');
  pnpm(['pack','--out',svelte],fileURLToPath(new URL('../../svelte/',import.meta.url)));
  pnpm(['pack','--out',web],webRoot);
  pnpm(['pack','--out',kit],packageRoot);
  const consumer=path.join(root,'consumer');mkdirSync(consumer);
  writeFileSync(path.join(consumer,'package.json'),JSON.stringify({type:'module',private:true,dependencies:{
    '@antepodeum/linguini-sveltekit':`file:${kit.replaceAll('\\','/')}`,
    '@antepodeum/linguini-web':`file:${web.replaceAll('\\','/')}`,
    '@sveltejs/kit':'2.60.1',svelte:'5.55.8',typescript:'6.0.3','@types/node':'24.12.4',
  }}));
  writeFileSync(path.join(consumer,'pnpm-workspace.yaml'),`overrides:\n  '@antepodeum/linguini-web': ${JSON.stringify(`file:${web.replaceAll('\\','/')}`)}\n  '@antepodeum/linguini-svelte': ${JSON.stringify(`file:${svelte.replaceAll('\\','/')}`)}\n`);
  pnpm(['install','--offline','--ignore-scripts','--store-dir',store],consumer);
  writeFileSync(path.join(consumer,'tsconfig.json'),JSON.stringify({compilerOptions:{strict:true,allowJs:false,skipLibCheck:false,module:'NodeNext',moduleResolution:'NodeNext',target:'ES2022',outDir:'output',types:['node','@sveltejs/kit']},include:['consumer.ts']}));
  writeFileSync(path.join(consumer,'consumer.ts'),`
import {createWebLocaleI18n,pathLocale,cookieLocale} from '@antepodeum/linguini-web';
import {createReroute} from '@antepodeum/linguini-sveltekit';
import {createHandle,createLoad} from '@antepodeum/linguini-sveltekit/server';
import type {Handle} from '@sveltejs/kit';
const metadata={locales:['en','fr'] as const,baseLocale:'en' as const};
const web=createWebLocaleI18n(metadata,{locale:{sources:[pathLocale(),cookieLocale({name:'LANG'})]}});
const handle:Handle=createHandle(web,{switchRoute:{path:'/locale/{locale}'}});
const reroute=createReroute(web);
const load=createLoad<'en'|'fr'>();
const locale:'en'|'fr'|undefined=load({locals:{}}).linguini?.locale;
// @ts-expect-error Locale types stay finite in installed declarations.
void (()=>web.localizeHref('/','ar'));
// @ts-expect-error Runtime resolver capabilities replace generator strings.
void (()=>createWebLocaleI18n(metadata,{locale:{sources:['path']}}));
// @ts-expect-error Cookie policy remains closed.
void (()=>cookieLocale({sameSite:'anything'}));
console.log(JSON.stringify([web.localizeHref('/en/shop','fr'),typeof handle,typeof reroute,locale]));
`);
  pnpm(['exec','tsc','--project','tsconfig.json'],consumer);
  const result=spawnSync(process.execPath,[path.join(consumer,'output/consumer.js')],{cwd:consumer,encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);
  assert.deepEqual(JSON.parse(result.stdout),['/fr/shop','function','function',null]);
  for(const name of ['web','sveltekit']) {
    const installed=path.join(consumer,'node_modules/@antepodeum/linguini-'+name);
    const map=JSON.parse(readFileSync(path.join(installed,'dist/index.d.ts.map'),'utf8'));
    for(const source of map.sources) readFileSync(path.resolve(installed,'dist',source),'utf8');
  }
});
