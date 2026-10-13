import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import { base } from "$app/paths";
import { createWebI18n } from "./web";
import * as runtime from "./index";
import { persistLocaleCookie } from "./web/server-cookie.js";

const options = { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: ["path", "cookie", "accept-language"] as const, switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } as const }, links: { mode: "runtime" }, routes: { exclude: [] as const }, cookie: { name: "LINGUINI_LOCALE", path: "auto", maxAge: 31536000, sameSite: "lax", secure: "auto", httpOnly: false } } as const;

export const linguiniHandle: Handle = createHandle(runtime, options);
export const linguiniReroute: Reroute = createReroute(runtime, options);
export const linguiniLoad: ServerLoad = createLoad();
export const handle: Handle = linguiniHandle;
export const reroute: Reroute = linguiniReroute;
export const load: ServerLoad = linguiniLoad;

function createHandle(runtime: typeof import("./index"), options: Record<string, unknown> = {}) {
  const web = createWebI18n(runtime, options.web as Record<string, unknown> | undefined ?? options, { base });
  const persistCookie = options.persistCookie !== false && web.options.locale.switch.writesCookie;

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


    event.locals.linguini = context;

    const redirectLocation = web.getCanonicalRedirect(event.url, context.locale);
    if (redirectLocation) {
      const response = new Response(null, {
        status: 307,
        headers: { location: redirectLocation },
      });
      if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });
      return response;
    }

    const response = await resolve(event, {
      transformPageChunk: ({ html }: { html: string }) =>
        html
          .replaceAll("%linguini.lang%", context.lang)
          .replaceAll("%linguini.dir%", context.direction)
          .replaceAll("%linguini.locale%", context.locale),
    });

    if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });
    return response;
  };
}

function createReroute(runtime: typeof import("./index"), options: Record<string, unknown> = {}) {
  const web = createWebI18n(runtime, options.web as Record<string, unknown> | undefined ?? options, { base });
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
//# sourceMappingURL=sveltekit.ts.map
