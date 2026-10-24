import svelteLinguini from '@antepodeum/linguini-vite/svelte';
import type { LinguiniViteOptions } from '@antepodeum/linguini-vite';

export default function linguini(options: LinguiniViteOptions = {}): ReturnType<typeof svelteLinguini> {
  const args = [...(options.args ?? ['build'])];
  if (args.includes('build') && !args.includes('--svelte-host-module')) {
    args.push('--svelte-host-module','@antepodeum/linguini-sveltekit/client');
  }
  return svelteLinguini({...options,args});
}
