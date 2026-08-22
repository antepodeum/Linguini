import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import { base } from "$app/paths";
import type { Locale } from "./locale";
import * as locale from "./locale";
import { createWebLocaleI18n } from "./web";
{{SERVER_COOKIE_IMPORT}}
{{SWITCH_ROUTE_IMPORT}}

const options = {{OPTIONS}};

export const linguiniHandle: Handle = createHandle(locale, options);
export const linguiniReroute: Reroute = createReroute(locale, options);
export const linguiniLoad: ServerLoad = createLoad();
export const handle: Handle = linguiniHandle;
export const reroute: Reroute = linguiniReroute;
export const load: ServerLoad = linguiniLoad;

function createHandle(runtime: typeof import("./locale"), options: Record<string, unknown> = {}) {
  const web = createWebLocaleI18n(runtime, options.web as Record<string, unknown> | undefined ?? options, { base });
{{PERSIST_COOKIE_DECLARATION}}

  return async function linguiniHandle({ event, resolve }: Parameters<Handle>[0]) {
    if (web.shouldExclude(event.url)) {
      return resolve(event);
    }
{{SWITCH_ROUTE_BRANCH}}

    const contextInput = {
      url: event.url,
      currentUrl: event.url,
      origin: event.url.origin,
      headers: event.request.headers,
{{COOKIE_INPUT}}
    };
    const resolvedLocale = await web.resolveLocale(contextInput);
    const context = createRequestContext(web, resolvedLocale, contextInput);

    event.locals.linguini = context;

    const redirectLocation = web.getCanonicalRedirect(event.url, context.locale);
    if (redirectLocation) {
      const response = new Response(null, {
        status: 307,
        headers: { location: redirectLocation },
      });
{{PERSIST_REDIRECT_COOKIE}}
      return response;
    }

    const response = await resolve(event, {
      transformPageChunk: ({ html }: { html: string }) =>
        html
          .replaceAll("%linguini.lang%", context.lang)
          .replaceAll("%linguini.dir%", context.direction)
          .replaceAll("%linguini.locale%", context.locale),
    });

{{PERSIST_RESPONSE_COOKIE}}
    return response;
  };
}

function createReroute(runtime: typeof import("./locale"), options: Record<string, unknown> = {}) {
  const web = createWebLocaleI18n(runtime, options.web as Record<string, unknown> | undefined ?? options, { base });
  return function linguiniReroute({ url }: { url: URL }) {
    if (web.shouldExclude(url)) return undefined;
    const delocalized = web.delocalizePathname(url.pathname);
    return delocalized === url.pathname ? undefined : delocalized;
  };
}

function createRequestContext(
  web: import("./web").LinguiniWebLocale<Locale>,
  resolved: Locale,
  contextInput: Record<string, unknown>,
) {
  const resolvedLocale = web.matchLocale(resolved) ?? web.baseLocale;
  return {
    locale: resolvedLocale,
    baseLocale: web.baseLocale,
    locales: web.locales,
    direction: web.getTextDirection(resolvedLocale),
    textDirection: web.getTextDirection(resolvedLocale),
    lang: resolvedLocale,
    htmlAttrs: web.htmlAttrs(resolvedLocale),
    localizeHref: (href: string, nextLocale = resolvedLocale, input = contextInput) => web.localizeHref(href, nextLocale, input),
    localizeUrl: (url: string | URL, nextLocale = resolvedLocale, input = contextInput) => web.localizeUrl(url, nextLocale, input),
    shouldLocalizeHref: (href: string, input = contextInput) => web.shouldLocalizeHref(href, input),
    shouldLocalizeLink: (href: string, attributes = {}, input = contextInput) => web.shouldLocalizeLink(href, attributes, input),
    localizeHrefAttribute: (href: string, nextLocale = resolvedLocale, input = contextInput) => web.localizeHrefAttribute(href, nextLocale, input),
    delocalizeUrl: (url: string | URL, input = contextInput) => web.delocalizeUrl(url, input),
    alternateLinks: (url: string | URL, input = contextInput) => web.alternateLinks(url, input),
  };
}

function createLoad() {
  return function linguiniLayoutLoad({ locals }: { locals: Record<string, any> }) {
    return {
      linguini: serializeContext(locals.linguini),
    };
  };
}

function serializeContext(context: any) {
  if (!context) return undefined;
  return {
    locale: context.locale,
    baseLocale: context.baseLocale,
    locales: context.locales,
    direction: context.direction,
    lang: context.lang,
    htmlAttrs: context.htmlAttrs,
  };
}
