import { browser } from '$app/environment';
import { createControls } from '@antepodeum/linguini-sveltekit/client';
import { startRuntimeLinkLocalization } from '@antepodeum/linguini-web/runtime-links';
import * as localeState from '$lib/generated/linguini/svelte-locale.svelte.js';
import { web } from './linguini-web.js';

const effects = browser ? startRuntimeLinkLocalization(web, localeState.getCurrentLocale) : undefined;
export const linguini = createControls(web, localeState, {
  refresh: () => effects?.refresh(),
  destroy: () => effects?.destroy(),
});
export const setLocale = linguini.setLocale;
export const localizeHref = linguini.localizeHref;
if (import.meta.hot) import.meta.hot.dispose(() => linguini.destroy());
