import { fileURLToPath } from 'node:url';
import { runCompiler } from './compiler.mjs';

runCompiler(['build','--svelte','--svelte-host-module','@antepodeum/linguini-sveltekit/client'],{
  cwd:fileURLToPath(new URL('../',import.meta.url)),stdio:'inherit'
});
