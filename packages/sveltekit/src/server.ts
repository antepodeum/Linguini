import type { Handle, RequestEvent } from '@sveltejs/kit';
import type { LinguiniLocaleRequestContext, LinguiniWebLocale } from '@antepodeum/linguini-web';
import { createLocaleRequestContext } from '@antepodeum/linguini-web';
import { createLocaleSwitchHandler } from '@antepodeum/linguini-web/server';
import type { LocaleSwitchRouteOptions } from '@antepodeum/linguini-web/server';
import {createPagePreload} from './page-preload.js';
import type { PagePayload, SerializedLinguiniContext } from './types.js';

export interface HandleOptions {
  persistCookie?: boolean;
  getPagePayload?: LoadOptions['getPagePayload'];
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
    const payload = !event.isDataRequest && event.route?.id && options.getPagePayload
      ? await selectPagePayload(event.locals,options.getPagePayload,event.route.id,locale)
      : undefined;
    const transform = createHtmlTransform(context);
    const preload = payload ? createPagePreload(payload) : undefined;
    const response = await resolve(event, {transformPageChunk:chunk => {
      const html = transform(chunk);
      return preload ? preload({html,done:chunk.done}) : html;
    }});
    if (!persistCookie) return response;
    // Fetch responses may have immutable headers; preserve their streaming body.
    const persisted = new Response(response.body, {status:response.status, statusText:response.statusText, headers:response.headers});
    web.setLocaleCookie(persisted, locale, {origin:event.url.origin});
    return persisted;
  };
}

export interface LoadOptions {
  getPagePayload?: (input: {routeId: string; locale: string}) => Promise<PagePayload | undefined>;
}

let pageManifest: Promise<typeof import('virtual:linguini/page-manifest')> | undefined;
let pathContext: Promise<typeof import('$app/paths')> | undefined;

export async function getPagePayload({routeId,locale}: {routeId: string; locale: string}): Promise<PagePayload | undefined> {
  const {payloads} = await (pageManifest ??= import('virtual:linguini/page-manifest'));
  const file = payloads[routeId]?.[locale];
  if (!file) return;
  const paths = await (pathContext ??= import('$app/paths'));
  return {url:(paths.assets || paths.base) + '/' + file};
}

type LoadEvent = Pick<RequestEvent, 'locals'> & Partial<Pick<RequestEvent, 'url'|'route'>>;
type LoadResult<Locale extends string> = {linguini: SerializedLinguiniContext<Locale> | undefined};

export function createLoad<Locale extends string = string>(): (event: LoadEvent) => LoadResult<Locale>;
export function createLoad<Locale extends string = string>(options: LoadOptions): (event: LoadEvent) => LoadResult<Locale> | Promise<LoadResult<Locale>>;
export function createLoad<Locale extends string = string>(options: LoadOptions = {}) {
  return ({locals,url,route}: Pick<RequestEvent, 'locals'> & Partial<Pick<RequestEvent, 'url'|'route'>>) => {
    // SvelteKit otherwise reuses shared layout metadata when only the localized URL changes.
    void url?.pathname;
    const context = (locals as {linguini?:LinguiniLocaleRequestContext<Locale>}).linguini;
    const result = {linguini: context ? {
      locale: context.locale,
      baseLocale: context.baseLocale,
      locales: context.locales,
      direction: context.direction,
      lang: context.lang,
      htmlAttrs: context.htmlAttrs,
    } : undefined};
    if (context && route?.id && options.getPagePayload) {
      return selectPagePayload(locals,options.getPagePayload,route.id,context.locale).then(payload => ({
        linguini: {...result.linguini!,payload},
      }));
    }
    return result;
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

const pagePayloadSelection = Symbol('linguini.page-payload');
interface PayloadSelection {
  resolver: NonNullable<LoadOptions['getPagePayload']>;
  routeId: string;
  locale: string;
  task: Promise<PagePayload | undefined>;
}
function selectPagePayload(locals: RequestEvent['locals'], resolver: PayloadSelection['resolver'], routeId: string, locale: string) {
  const storage = locals as {[pagePayloadSelection]?: PayloadSelection};
  const previous = storage[pagePayloadSelection];
  if (previous?.resolver === resolver && previous.routeId === routeId && previous.locale === locale) return previous.task;
  const task = resolver({routeId,locale});
  storage[pagePayloadSelection] = {resolver,routeId,locale,task};
  return task;
}
