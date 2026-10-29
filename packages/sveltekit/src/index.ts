import type { Reroute } from '@sveltejs/kit';
import type { LinguiniWebLocale } from '@antepodeum/linguini-web';

export function createReroute<Locale extends string>(web: LinguiniWebLocale<Locale>): Reroute {
  return ({url}) => {
    if (web.shouldExclude(url)) return undefined;
    const path = web.delocalizePathname(url.pathname);
    return path === url.pathname ? undefined : path;
  };
}

export type { PagePayload, SerializedLinguiniContext } from './types.js';
