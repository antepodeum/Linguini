import { createLoad, getPagePayload } from '@antepodeum/linguini-sveltekit/server';
import type { Locale } from '$lib/generated/linguini/locale.js';

export const load = createLoad<Locale>({getPagePayload});
