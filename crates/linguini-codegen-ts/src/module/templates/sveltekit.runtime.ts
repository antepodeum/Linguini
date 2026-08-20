import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import { createWebI18n } from "./web";
import * as runtime from "./index";
{{SERVER_COOKIE_IMPORT}}

const options = {{OPTIONS}};

export const linguiniHandle: Handle = createHandle(runtime, options);
export const linguiniReroute: Reroute = createReroute(runtime, options);
export const linguiniLoad: ServerLoad = createLoad();
export const handle: Handle = linguiniHandle;
export const reroute: Reroute = linguiniReroute;
export const load: ServerLoad = linguiniLoad;

function createHandle(runtime: typeof import("./index"), options: Record<string, unknown> = {}) {
  const web = createWebI18n(runtime, options.web as Record<string, unknown> | undefined ?? options);
{{PERSIST_COOKIE_DECLARATION}}

  return async function linguiniHandle({ event, resolve }: Parameters<Handle>[0]) {
    if (web.shouldExclude(event.url)) {
      return resolve(event);
    }

    const context = await web.resolveRequest(event.request, {
      url: event.url,
      currentUrl: event.url,
      origin: event.url.origin,
      headers: event.request.headers,
{{COOKIE_INPUT}}
    });

    const locals = event.locals as Record<string, unknown>;
    locals.linguini = context;

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
