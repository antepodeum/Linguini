import { createClientLoad } from '@antepodeum/linguini-sveltekit/load';
import type { Locale } from '$lib/generated/linguini/locale.js';

export const load = createClientLoad<Locale>();
export const prerender = true;
export const trailingSlash = 'always';
