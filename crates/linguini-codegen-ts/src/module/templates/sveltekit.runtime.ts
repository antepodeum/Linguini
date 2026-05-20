import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import { createWebI18n } from "./web";
import * as runtime from "./index";

const options = {{OPTIONS}};

export const handle: Handle = createHandle(runtime, options);
export const reroute: Reroute = createReroute(runtime, options);
export const load: ServerLoad = createLoad();

function createHandle(runtime: typeof import("./index"), options: Record<string, unknown> = {}) {
  const web = createWebI18n(runtime, options.web as Record<string, unknown> | undefined ?? options);
  const redirectStatus = Number(options.redirectStatus ?? 307);
  const persistCookie = options.persistCookie !== false;

  return async function linguiniHandle({ event, resolve }: Parameters<Handle>[0]) {
    if (web.shouldExclude(event.url)) {
      return resolve(event);
    }

    const context = await web.resolveRequest(event.request, {
      url: event.url,
      currentUrl: event.url,
      origin: event.url.origin,
      headers: event.request.headers,
      cookie: event.request.headers.get("cookie") ?? undefined,
    });

    const locals = event.locals as Record<string, unknown>;
    locals.linguini = context;
    locals.locale = context.locale;
    locals.direction = context.direction;
    locals.l = context.l;

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
      transformPageChunk: ({ html, done }: { html: string; done: boolean }) => {
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

function createReroute(runtime: typeof import("./index"), options: Record<string, unknown> = {}) {
  const web = createWebI18n(runtime, options.web as Record<string, unknown> | undefined ?? options);
  return function linguiniReroute({ url }: { url: URL }) {
    if (web.shouldExclude(url)) return undefined;
    const delocalized = web.delocalizePathname(url.pathname);
    return delocalized === url.pathname ? undefined : delocalized;
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
