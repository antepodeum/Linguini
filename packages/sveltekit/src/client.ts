import { page } from '$app/state';
import { goto } from '$app/navigation';

export function getRequestLocale(): unknown {
  // Route modules can load before SvelteKit publishes page.data during hydration.
  // The hook has already selected this locale and written it into the SSR HTML.
  return page.data.linguini?.locale ?? (browser ? document.documentElement.lang : undefined);
}

export type NavigationState = App.PageState;
export function navigate(href: string, options: Parameters<typeof goto>[1]): Promise<void> {
  return goto(href, options);
}

import { browser } from '$app/environment';
import type { LinguiniWebLocale } from '@antepodeum/linguini-web';
import { createLocaleControl } from '@antepodeum/linguini-web/client';
import type { LocaleState, LocaleControlOptions } from '@antepodeum/linguini-web/client';

export function createControls<Locale extends string>(web: LinguiniWebLocale<Locale>, state: LocaleState<Locale>, options: Omit<LocaleControlOptions<Locale>, 'navigate'|'browser'> = {}) {
  return createLocaleControl(web,state,{
    ...options,
    browser,
    navigate: (href, input) => navigate(href,{
      replaceState: input.replaceState ?? false,
      invalidateAll: input.invalidateAll ?? true,
      keepFocus: input.keepFocus ?? true,
      noScroll: input.noScroll ?? true,
      state: input.state,
    }),
  });
}

import {createLocaleState as createSvelteLocaleState} from '@antepodeum/linguini-svelte';
import type {LocaleRuntime,LocaleStateOptions} from '@antepodeum/linguini-svelte';
import {prepareCurrentPageMessages} from './page.svelte.js';
export {getPageMessage} from './page.svelte.js';
export {createClientLoad} from './load.js';
export {createLinguiniRune} from '@antepodeum/linguini-svelte';
export function createLocaleState<Locale extends string>(runtime: LocaleRuntime<Locale>, options: LocaleStateOptions = {}) {
  const state = createSvelteLocaleState(runtime, {
    ...options,
    getRequestIdentity: options.getRequestIdentity ?? (() => page.data.linguini),
  });
  if (browser) state.registerLocaleLoader(prepareCurrentPageMessages);
  return state;
}
export type {LocaleState,LinguiniRune} from '@antepodeum/linguini-svelte';
