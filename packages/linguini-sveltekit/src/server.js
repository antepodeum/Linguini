import { createWebI18n } from "@antepod/linguini-web";

export function createHandle(runtime, options = {}) {
  const web = createWebI18n(runtime, options.web ?? options);
  const redirectStatus = options.redirectStatus ?? 307;
  const persistCookie = options.persistCookie ?? true;

  return async function linguiniHandle({ event, resolve }) {
    if (web.shouldExclude(event.url)) {
      return resolve(event);
    }

    const context = await web.resolveRequest(event.request, {
      url: event.url,
      cookies: event.cookies,
      headers: event.request.headers,
    });

    event.locals.linguini = context;
    event.locals.locale = context.locale;
    event.locals.direction = context.direction;
    event.locals.l = context.l;

    const redirectLocation = web.getCanonicalRedirect(event.url, context.locale);
    if (redirectLocation) {
      const response = new Response(null, {
        status: redirectStatus,
        headers: { location: redirectLocation },
      });
      if (persistCookie) web.setLocaleCookie(response, context.locale);
      return response;
    }

    let bufferedHtml = "";
    const response = await resolve(event, {
      transformPageChunk: ({ html, done }) => {
        const transformed = html
          .replaceAll("%linguini.lang%", context.lang)
          .replaceAll("%linguini.dir%", context.direction)
          .replaceAll("%linguini.locale%", context.locale);

        if (web.options.localizeLinks === false) return transformed;

        if (done === false) {
          bufferedHtml += transformed;
          return "";
        }

        const fullHtml = bufferedHtml + transformed;
        bufferedHtml = "";
        return web.localizeMarkupLinks(fullHtml, context.locale, {
          currentUrl: event.url,
          origin: event.url.origin,
        });
      },
    });

    if (persistCookie) web.setLocaleCookie(response, context.locale);
    return response;
  };
}

export function createReroute(runtime, options = {}) {
  const web = createWebI18n(runtime, options.web ?? options);
  return function linguiniReroute({ url }) {
    if (web.shouldExclude(url)) return undefined;
    const delocalized = web.delocalizePathname(url.pathname);
    return delocalized === url.pathname ? undefined : delocalized;
  };
}

export function createLoad() {
  return function linguiniLayoutLoad({ locals }) {
    return {
      linguini: serializeContext(locals.linguini),
    };
  };
}

export function serializeContext(context) {
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
