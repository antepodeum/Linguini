import { base } from "$app/paths";
import * as locale from "./locale.js";
import { createWebLocaleI18n } from "./web.js";

const options = { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: ["path"], switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } }, links: { mode: "runtime" }, routes: { exclude: [] } };

/** @type {import("@sveltejs/kit").Handle} */
export const linguiniHandle = createHandle(locale, options);
/** @type {import("@sveltejs/kit").Reroute} */
export const linguiniReroute = createReroute(locale, options);
/** @type {import("@sveltejs/kit").ServerLoad} */
export const linguiniLoad = createLoad();
/** @type {import("@sveltejs/kit").Handle} */
export const handle = linguiniHandle;
/** @type {import("@sveltejs/kit").Reroute} */
export const reroute = linguiniReroute;
/** @type {import("@sveltejs/kit").ServerLoad} */
export const load = linguiniLoad;

/** @param {typeof import("./locale.js")} runtime @param {Record<string, unknown>} [options] @returns {import("@sveltejs/kit").Handle} */
function createHandle(runtime, options = {}) {
  const web = createWebLocaleI18n(runtime, /** @type {Record<string, unknown> | undefined} */ (options.web) ?? options, { base });


  /** @param {Parameters<import("@sveltejs/kit").Handle>[0]} input */
  return async function linguiniHandle({ event, resolve }) {
    if (web.shouldExclude(event.url)) {
      return resolve(event);
    }


    const contextInput = {
      url: event.url,
      currentUrl: event.url,
      origin: event.url.origin,
      headers: event.request.headers,

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

      return response;
    }

    const response = await resolve(event, {
      transformPageChunk: ({ html }) =>
        html
          .replaceAll("%linguini.lang%", context.lang)
          .replaceAll("%linguini.dir%", context.direction)
          .replaceAll("%linguini.locale%", context.locale),
    });


    return response;
  };
}

/** @param {typeof import("./locale.js")} runtime @param {Record<string, unknown>} [options] @returns {import("@sveltejs/kit").Reroute} */
function createReroute(runtime, options = {}) {
  const web = createWebLocaleI18n(runtime, /** @type {Record<string, unknown> | undefined} */ (options.web) ?? options, { base });
  /** @param {{url: URL}} input */
  return function linguiniReroute({ url }) {
    if (web.shouldExclude(url)) return undefined;
    const delocalized = web.delocalizePathname(url.pathname);
    return delocalized === url.pathname ? undefined : delocalized;
  };
}

/** @param {import("./web.js").LinguiniWebLocale<import("./locale.js").Locale>} web @param {import("./locale.js").Locale} resolved @param {Record<string, unknown>} contextInput @returns {import("./web.js").LinguiniLocaleRequestContext<import("./locale.js").Locale>} */
function createRequestContext(web, resolved, contextInput) {
  const resolvedLocale = web.matchLocale(resolved) ?? web.baseLocale;
  return {
    locale: resolvedLocale,
    baseLocale: web.baseLocale,
    locales: web.locales,
    direction: web.getTextDirection(resolvedLocale),
    textDirection: web.getTextDirection(resolvedLocale),
    lang: resolvedLocale,
    htmlAttrs: web.htmlAttrs(resolvedLocale),
    localizeHref: (href, nextLocale = resolvedLocale, input = contextInput) => web.localizeHref(href, nextLocale, input),
    localizeUrl: (url, nextLocale = resolvedLocale, input = contextInput) => web.localizeUrl(url, nextLocale, input),
    shouldLocalizeHref: (href, input = contextInput) => web.shouldLocalizeHref(href, input),
    shouldLocalizeLink: (href, attributes = {}, input = contextInput) => web.shouldLocalizeLink(href, attributes, input),
    localizeHrefAttribute: (href, nextLocale = resolvedLocale, input = contextInput) => web.localizeHrefAttribute(href, nextLocale, input),
    delocalizeUrl: (url, input = contextInput) => web.delocalizeUrl(url, input),
    alternateLinks: (url, input = contextInput) => web.alternateLinks(url, input),
  };
}

function createLoad() {
  /** @param {{locals: App.Locals}} input */
  return function linguiniLayoutLoad({ locals }) {
    return {
      linguini: serializeContext(locals.linguini),
    };
  };
}

/** @param {import("./sveltekit-control.js").LinguiniSvelteKitLocaleContext | undefined} context @returns {import("./sveltekit-control.js").LinguiniSerializedContext | undefined} */
function serializeContext(context) {
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
//# sourceMappingURL=sveltekit-control.js.map
