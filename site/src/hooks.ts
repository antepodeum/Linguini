import { createReroute } from '@antepodeum/linguini-sveltekit';
import { web } from '$lib/linguini-web.js';

export const reroute = createReroute(web);
