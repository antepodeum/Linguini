import type { Handle, RequestEvent } from '@sveltejs/kit';
import type { LinguiniLocaleRequestContext, LinguiniWebLocale } from '@antepodeum/linguini-web';
import { createLocaleRequestContext } from '@antepodeum/linguini-web';
import { createLocaleSwitchHandler } from '@antepodeum/linguini-web/server';
import type { LocaleSwitchRouteOptions } from '@antepodeum/linguini-web/server';
import type { SerializedLinguiniContext } from './types.js';

export interface HandleOptions {
  persistCookie?: boolean;
  switchRoute?: LocaleSwitchRouteOptions;
}

export function createHandle<Locale extends string>(web: LinguiniWebLocale<Locale>, options: HandleOptions = {}): Handle {
  if (web.options.locale.sources.length === 1 && web.options.locale.sources[0] === 'local-storage') {
    throw new TypeError('SvelteKit SSR requires a server-readable locale source');
  }
  const persistCookie = options.persistCookie !== false && web.options.locale.switch.writesCookie;
  const switchLocale = options.switchRoute ? createLocaleSwitchHandler(web, options.switchRoute) : undefined;
  return async ({event, resolve}) => {
    if (web.shouldExclude(event.url)) return resolve(event);
    const switched = switchLocale?.(event);
    if (switched) return switched;
    const input = {
      url: event.url,
      currentUrl: event.url,
      origin: event.url.origin,
      headers: event.request.headers,
      cookie: event.request.headers.get('cookie') ?? undefined,
    };
    const locale = web.resolveLocaleSync(input);
    const context = createLocaleRequestContext(web, locale, input);
    (event.locals as {linguini?: LinguiniLocaleRequestContext<Locale>}).linguini = context;
    const location = web.getCanonicalRedirect(event.url, locale);
    // SvelteKit owns trailingSlash. Redirecting a slash-only difference here
    // would alternate with SvelteKit's own canonical redirect indefinitely.
    if (location && new URL(location, event.url).pathname.replace(/\/+$/, '') !== event.url.pathname.replace(/\/+$/, '')) {
      const response = new Response(null, {status:307, headers:{location}});
      if (persistCookie) web.setLocaleCookie(response, locale, {origin:event.url.origin});
      return response;
    }
    const response = await resolve(event, {transformPageChunk:createHtmlTransform(context)});
    if (!persistCookie) return response;
    // Fetch responses may have immutable headers; preserve their streaming body.
    const persisted = new Response(response.body, {status:response.status, statusText:response.statusText, headers:response.headers});
    web.setLocaleCookie(persisted, locale, {origin:event.url.origin});
    return persisted;
  };
}

export function createLoad<Locale extends string = string>() {
  return ({locals}: Pick<RequestEvent, 'locals'>): {linguini:SerializedLinguiniContext<Locale> | undefined} => {
    const context = (locals as {linguini?:LinguiniLocaleRequestContext<Locale>}).linguini;
    return {linguini: context ? {
      locale: context.locale,
      baseLocale: context.baseLocale,
      locales: context.locales,
      direction: context.direction,
      lang: context.lang,
      htmlAttrs: context.htmlAttrs,
    } : undefined};
  };
}

function createHtmlTransform<Locale extends string>(context: LinguiniLocaleRequestContext<Locale>) {
  const replacements = [
    ['%linguini.lang%', context.lang],
    ['%linguini.dir%', context.direction],
    ['%linguini.locale%', context.locale],
  ] as const;
  const maxPrefix = Math.max(...replacements.map(([token]) => token.length)) - 1;
  let pending = '';
  return ({html, done}: {html:string; done:boolean}): string => {
    let output = pending + html;
    pending = '';
    for (const [token, value] of replacements) output = output.replaceAll(token, value);
    if (done) return output;
    // Only a possible split placeholder is retained between streamed chunks.
    for (let length = Math.min(maxPrefix, output.length); length > 0; length--) {
      const suffix = output.slice(-length);
      if (replacements.some(([token]) => token.startsWith(suffix))) {
        pending = suffix;
        return output.slice(0, -length);
      }
    }
    return output;
  };
}
