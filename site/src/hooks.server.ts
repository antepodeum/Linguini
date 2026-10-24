import { createHandle } from '@antepodeum/linguini-sveltekit/server';
import { web } from '$lib/server/linguini.js';

export const handle = createHandle(web);
