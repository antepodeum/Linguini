use super::names::{escape_string, property_key, safe_identifier};
use super::{TypeScriptLocaleModule, TypeScriptWebOptions};
use linguini_cldr::built_in_text_direction;

pub fn generate_project_index(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("export const locales = [");
    output.push_str(&locale_literals(locales).join(", "));
    output.push_str("] as const;\n");
    output.push_str("export const baseLocale = ");
    output.push_str(&base_locale_literal(locales, base_locale));
    output.push_str(";\n\n");
    output.push_str("export const localeDirections = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: \"{}\",\n",
            property_key(&locale.locale),
            locale_direction(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export const localeModules = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: {},\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export const localeLoaders = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: () => Promise.resolve({}),\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Locale = (typeof locales)[number];\n");
    output.push_str("export type TextDirection = \"ltr\" | \"rtl\";\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_types(&mut output);
    push_locale_fallback_runtime(&mut output);
    output
        .push_str("export function createLinguini(language: LinguiniLanguageInput): Linguini {\n");
    output.push_str("  const locale = normalizeLocale(language) ?? baseLocale;\n");
    output.push_str("  return localeModules[locale];\n");
    output.push_str("}\n\n");
    output.push_str("export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {\n");
    output.push_str(
        "  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);\n",
    );
    output.push_str("  return new Proxy({} as Linguini, {\n");
    output.push_str("    get(_target, property) {\n");
    output.push_str("      return createLinguini(resolve())[property as keyof Linguini];\n");
    output.push_str("    },\n");
    output.push_str("  });\n");
    output.push_str("}\n\n");
    output.push_str("export function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini {\n");
    output.push_str("  if (typeof options.language === \"function\") {\n");
    output.push_str("    return createLinguiniProvider({ resolveLanguage: options.language });\n");
    output.push_str("  }\n");
    output.push_str("  return createLinguini(options.language);\n");
    output.push_str("}\n\n");
    output.push_str("export const lgl: Linguini = createLinguini(baseLocale);\n");
    push_runtime_functions(&mut output);
    output
}

pub fn generate_project_index_declaration(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("export declare const locales: readonly [");
    output.push_str(&locale_literals(locales).join(", "));
    output.push_str("];\n");
    output.push_str("export declare const baseLocale: ");
    output.push_str(&base_locale_literal(locales, base_locale));
    output.push_str(";\n\n");
    output.push_str("export declare const localeDirections: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: \"{}\";\n",
            property_key(&locale.locale),
            locale_direction(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("export declare const localeModules: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: typeof {};\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("export declare const localeLoaders: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: () => Promise<typeof {}>;\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Locale = (typeof locales)[number];\n");
    output.push_str("export type TextDirection = \"ltr\" | \"rtl\";\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_declarations(&mut output);
    output.push_str(
        "export declare function createLinguini(language: LinguiniLanguageInput): Linguini;\n\n",
    );
    output.push_str(
        "export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;\n\n",
    );
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini;\n\n");
    output.push_str("export declare const lgl: Linguini;\n");
    output.push('\n');
    push_runtime_function_declarations(&mut output);
    output
}

pub fn generate_project_svelte_module(options: &TypeScriptWebOptions) -> String {
    let mut output = String::new();
    output.push_str("import { browser } from \"$app/environment\";\n");
    output.push_str("import { goto } from \"$app/navigation\";\n");
    output.push_str("import { page } from \"$app/state\";\n");
    output.push_str("import { createWebI18n } from \"./web\";\n");
    output.push_str("import * as runtime from \"./index\";\n\n");
    output.push_str("let activeAutoLinkCleanup: (() => void) | undefined;\n\n");
    output.push_str("export const linguini = createLinguiniRune(runtime, ");
    output.push_str(&web_options_literal(options));
    output.push_str(");\n");
    output.push_str("export const l = linguini.l;\n");
    output.push_str("export const messages = linguini.messages;\n");
    output.push_str("export const setLocale = linguini.setLocale;\n");
    output.push_str("export const localizeHref = linguini.localizeHref;\n");
    output.push_str("export const localizeUrl = linguini.localizeUrl;\n");
    output.push_str("export const delocalizeUrl = linguini.delocalizeUrl;\n");
    output.push_str("export const alternateLinks = linguini.alternateLinks;\n");
    output.push_str(SVELTE_RUNTIME);
    output
}

pub fn generate_project_svelte_declaration() -> String {
    let mut output = String::new();
    output.push_str("import type { AlternateLink } from \"./web\";\n");
    output.push_str("import type { Locale, Linguini, TextDirection } from \"./index\";\n\n");
    output.push_str(SVELTE_DECLARATIONS);
    output.push_str("export declare const linguini: LinguiniRune<Locale, Linguini>;\n");
    output.push_str("export declare const l: Linguini;\n");
    output.push_str("export declare const messages: Linguini;\n");
    output.push_str(
        "export declare const setLocale: LinguiniRune<Locale, Linguini>[\"setLocale\"];\n",
    );
    output.push_str(
        "export declare const localizeHref: LinguiniRune<Locale, Linguini>[\"localizeHref\"];\n",
    );
    output.push_str(
        "export declare const localizeUrl: LinguiniRune<Locale, Linguini>[\"localizeUrl\"];\n",
    );
    output.push_str(
        "export declare const delocalizeUrl: LinguiniRune<Locale, Linguini>[\"delocalizeUrl\"];\n",
    );
    output.push_str("export declare const alternateLinks: LinguiniRune<Locale, Linguini>[\"alternateLinks\"];\n");
    output
}

pub fn generate_project_sveltekit_module(options: &TypeScriptWebOptions) -> String {
    let mut output = String::new();
    output.push_str("import type { Handle, Reroute, ServerLoad } from \"@sveltejs/kit\";\n");
    output.push_str("import { createWebI18n } from \"./web\";\n");
    output.push_str("import * as runtime from \"./index\";\n\n");
    output.push_str("const options = ");
    output.push_str(&web_options_literal(options));
    output.push_str(";\n\n");
    output.push_str("export const handle: Handle = createHandle(runtime, options);\n");
    output.push_str("export const reroute: Reroute = createReroute(runtime, options);\n");
    output.push_str("export const load: ServerLoad = createLoad();\n");
    output.push_str(SVELTEKIT_RUNTIME);
    output
}

pub fn generate_project_sveltekit_declaration() -> String {
    let mut output = String::new();
    output.push_str("import type { Handle, Reroute, ServerLoad } from \"@sveltejs/kit\";\n");
    output.push_str("import type { LinguiniRequestContext } from \"./web\";\n");
    output.push_str("import type { Locale, Linguini, TextDirection } from \"./index\";\n\n");
    output.push_str("export declare const handle: Handle;\n");
    output.push_str("export declare const reroute: Reroute;\n");
    output.push_str("export declare const load: ServerLoad;\n\n");
    output.push_str("export interface SerializedLinguiniContext<Locale extends string = string> {\n");
    output.push_str("  locale: Locale;\n");
    output.push_str("  baseLocale: Locale;\n");
    output.push_str("  locales: readonly Locale[];\n");
    output.push_str("  direction: TextDirection;\n");
    output.push_str("  lang: Locale;\n");
    output.push_str("  htmlAttrs: { lang: Locale; dir: TextDirection };\n");
    output.push_str("}\n\n");
    output.push_str("declare global {\n");
    output.push_str("  namespace App {\n");
    output.push_str("    interface Locals {\n");
    output.push_str("      linguini: LinguiniRequestContext<Locale, Linguini>;\n");
    output.push_str("      locale: Locale;\n");
    output.push_str("      direction: TextDirection;\n");
    output.push_str("      l: Linguini;\n");
    output.push_str("    }\n");
    output.push_str("    interface PageData {\n");
    output.push_str("      linguini?: SerializedLinguiniContext<Locale>;\n");
    output.push_str("    }\n");
    output.push_str("  }\n");
    output.push_str("}\n");
    output
}

pub fn generate_project_web_module() -> String {
    WEB_RUNTIME.to_owned()
}

pub fn generate_project_web_declaration() -> String {
    WEB_DECLARATIONS.to_owned()
}

const SVELTE_RUNTIME: &str = r#"

function createLinguiniRune(runtime: typeof import("./index"), options = {}) {
  const web = createWebI18n(runtime, options);
  let clientLocale = readInitialLocale(web);
  const messages = runtime.createLinguiniProvider({
    getLocale: () => getCurrentLocale(web, clientLocale),
  });

  if (browser && web.options.localizeLinks !== false) {
    activeAutoLinkCleanup?.();
    activeAutoLinkCleanup = startAutoLinkLocalization(web, () => getCurrentLocale(web, clientLocale));
  }

  async function setLocale(nextLocale: string, setOptions: Record<string, unknown> = {}) {
    const resolved = web.matchLocale(nextLocale) ?? web.baseLocale;
    clientLocale = resolved;

    if (browser) {
      writeLocalStorage(web, resolved);
      if (setOptions.cookie !== false) {
        document.cookie = web.serializeLocaleCookie(resolved, { httpOnly: false });
      }
      if (setOptions.navigate !== false) {
        const href = web.localizeHref(window.location.href, resolved);
        await goto(href, {
          replaceState: Boolean(setOptions.replaceState ?? false),
          invalidateAll: Boolean(setOptions.invalidateAll ?? true),
          keepFocus: setOptions.keepFocus as boolean | undefined,
          noScroll: setOptions.noScroll as boolean | undefined,
          state: setOptions.state as App.PageState | undefined,
        });
      }
      localizeDocumentLinks(web, resolved);
    }

    return resolved;
  }

  return {
    messages,
    l: messages,
    get locale() {
      return getCurrentLocale(web, clientLocale);
    },
    get lang() {
      return getCurrentLocale(web, clientLocale);
    },
    get direction() {
      return web.getTextDirection(getCurrentLocale(web, clientLocale));
    },
    get textDirection() {
      return web.getTextDirection(getCurrentLocale(web, clientLocale));
    },
    get htmlAttrs() {
      return web.htmlAttrs(getCurrentLocale(web, clientLocale));
    },
    setLocale,
    localizeHref: (href: string, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeHref(href, locale, input),
    localizeUrl: (url: string | URL, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeUrl(url, locale, input),
    shouldLocalizeHref: (href: string, input?: Record<string, unknown>) => web.shouldLocalizeHref(href, input),
    localizeHrefAttribute: (href: string, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeHrefAttribute(href, locale, input),
    localizeMarkupLinks: (html: string, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeMarkupLinks(html, locale, input),
    delocalizeUrl: (url: string | URL, input?: Record<string, unknown>) => web.delocalizeUrl(url, input),
    alternateLinks: (url: string | URL, input?: Record<string, unknown>) => web.alternateLinks(url, input),
  };
}

function getCurrentLocale(web: ReturnType<typeof createWebI18n>, clientLocale: string): any {
  const dataLocale = page.data?.linguini?.locale;
  return web.matchLocale(dataLocale) ?? web.matchLocale(clientLocale) ?? web.baseLocale;
}

function readInitialLocale(web: ReturnType<typeof createWebI18n>): any {
  if (!browser) return web.baseLocale;
  return web.resolveLocaleSync({
    url: window.location.href,
    cookie: document.cookie,
    localStorage,
    navigator,
  });
}

function writeLocalStorage(web: ReturnType<typeof createWebI18n>, locale: string) {
  try {
    localStorage.setItem(web.options.localStorageKey, locale);
  } catch {
    // Ignore storage failures in private browsing and locked-down contexts.
  }
}

function startAutoLinkLocalization(web: ReturnType<typeof createWebI18n>, getLocale: () => string) {
  if (typeof document === "undefined") return undefined;
  const localize = () => localizeDocumentLinks(web, getLocale());
  queueMicrotask(localize);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", localize, { once: true });
  }
  const observer = typeof MutationObserver === "undefined"
    ? undefined
    : new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          if (mutation.type === "attributes") {
            localizeAnchorElement(web, getLocale(), mutation.target as Element);
            continue;
          }
          for (const node of mutation.addedNodes) {
            localizeNodeLinks(web, getLocale(), node);
          }
        }
      });
  observer?.observe(document.documentElement, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["href"],
  });
  const onClick = (event: Event) => {
    const anchor = (event.target as Element | null)?.closest?.("a[href]");
    if (anchor) localizeAnchorElement(web, getLocale(), anchor);
  };
  document.addEventListener("click", onClick, true);
  return () => {
    observer?.disconnect();
    document.removeEventListener("click", onClick, true);
  };
}

function localizeDocumentLinks(web: ReturnType<typeof createWebI18n>, locale: string) {
  if (typeof document === "undefined") return;
  localizeNodeLinks(web, locale, document);
}

function localizeNodeLinks(web: ReturnType<typeof createWebI18n>, locale: string, node: Node) {
  if (node.nodeType !== 1 && node.nodeType !== 9 && node.nodeType !== 11) return;
  const element = node as Element;
  if (element.matches?.("a[href]")) localizeAnchorElement(web, locale, element);
  for (const anchor of element.querySelectorAll?.("a[href]") ?? []) {
    localizeAnchorElement(web, locale, anchor);
  }
}

function localizeAnchorElement(web: ReturnType<typeof createWebI18n>, locale: string, anchor: Element) {
  if (shouldSkipAnchor(anchor)) return;
  const href = anchor.getAttribute("href");
  if (!href) return;
  const localized = web.localizeHrefAttribute(href, locale, {
    currentUrl: window.location.href,
    origin: window.location.origin,
  });
  if (localized !== href) anchor.setAttribute("href", localized);
}

function shouldSkipAnchor(anchor: Element) {
  if (anchor.hasAttribute("download")) return true;
  if (anchor.hasAttribute("data-linguini-ignore")) return true;
  if (anchor.hasAttribute("data-linguini-no-localize")) return true;
  return (anchor.getAttribute("rel") ?? "").toLowerCase().split(/\s+/).includes("external");
}
"#;

const SVELTEKIT_RUNTIME: &str = r#"

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
"#;

const SVELTE_DECLARATIONS: &str = r#"export interface LinguiniSetLocaleOptions {
  navigate?: boolean;
  replaceState?: boolean;
  invalidateAll?: boolean;
  keepFocus?: boolean;
  noScroll?: boolean;
  cookie?: boolean;
  state?: App.PageState;
}

export interface LinguiniRune<Locale extends string, Linguini> {
  readonly messages: Linguini;
  readonly l: Linguini;
  readonly locale: Locale;
  readonly lang: Locale;
  readonly direction: TextDirection;
  readonly textDirection: TextDirection;
  readonly htmlAttrs: { lang: Locale; dir: TextDirection };
  setLocale(locale: Locale | string, options?: LinguiniSetLocaleOptions): Promise<Locale>;
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
}

"#;

const WEB_DECLARATIONS: &str = r#"export type TextDirection = "ltr" | "rtl";
export type LocaleStrategy = "url" | "cookie" | "localStorage" | "header" | "navigator" | "preferredLanguage" | "globalVariable" | "baseLocale" | `custom-${string}`;

export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> {
  locales: readonly Locale[];
  baseLocale: Locale;
  localeDirections?: Readonly<Record<Locale, TextDirection>>;
  createLinguini(locale: Locale): Linguini;
  normalizeLocale?(locale: unknown): Locale | undefined;
  getTextDirection?(locale: Locale): TextDirection;
}

export interface LinguiniWebOptions {
  strategy?: readonly LocaleStrategy[];
  cookieName?: string;
  localStorageKey?: string;
  prefixDefaultLocale?: boolean;
  basePath?: string;
  trailingSlash?: "ignore" | "always" | "never" | "directory";
  cookiePath?: string;
  cookieDomain?: string;
  cookieMaxAge?: number;
  cookieSameSite?: "lax" | "strict" | "none";
  cookieSecure?: boolean;
  cookieHttpOnly?: boolean;
  globalVariableName?: string;
  exclude?: readonly (string | RegExp | ((url: URL) => boolean))[];
  redirect?: boolean;
  origin?: string;
  localizeLinks?: boolean;
}

export interface AlternateLink {
  rel: "alternate";
  hreflang: string;
  href: string;
}

export interface LinguiniRequestContext<Locale extends string = string, Linguini = unknown> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  textDirection: TextDirection;
  lang: Locale;
  messages: Linguini;
  l: Linguini;
  htmlAttrs: { lang: Locale; dir: TextDirection };
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
}

export interface LinguiniWeb<Locale extends string = string, Linguini = unknown> extends LinguiniRuntime<Locale, Linguini> {
  options: Required<Pick<LinguiniWebOptions, "strategy" | "cookieName" | "localStorageKey" | "prefixDefaultLocale" | "basePath" | "trailingSlash" | "cookiePath" | "cookieMaxAge" | "cookieSameSite" | "cookieSecure" | "cookieHttpOnly" | "exclude" | "redirect" | "localizeLinks">> & LinguiniWebOptions;
  matchLocale(locale: unknown): Locale | undefined;
  resolveLocale(input?: Record<string, unknown>): Promise<Locale>;
  resolveLocaleSync(input?: Record<string, unknown>): Locale;
  resolveRequest(request: Request, input?: Record<string, unknown>): Promise<LinguiniRequestContext<Locale, Linguini>>;
  createRequestContext(locale: Locale, input?: Record<string, unknown>): LinguiniRequestContext<Locale, Linguini>;
  localizeUrl(url: string | URL, locale: Locale, input?: Record<string, unknown>): URL;
  localizeHref(href: string, locale: Locale, input?: Record<string, unknown>): string;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  delocalizePathname(pathname: string, input?: Record<string, unknown>): string;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
  htmlAttrs(locale: Locale): { lang: Locale; dir: TextDirection };
  getTextDirection(locale: Locale): TextDirection;
  getCanonicalRedirect(url: string | URL, locale: Locale, input?: Record<string, unknown>): string | undefined;
  shouldExclude(url: string | URL, input?: Record<string, unknown>): boolean;
  setLocaleCookie(target: unknown, locale: Locale, input?: Record<string, unknown>): void;
  serializeLocaleCookie(locale: Locale, input?: Record<string, unknown>): string;
}

export declare function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options?: LinguiniWebOptions): LinguiniWeb<Locale, Linguini>;
"#;

const WEB_RUNTIME: &str = r###"const DEFAULT_STRATEGY = ["url", "cookie", "localStorage", "preferredLanguage", "baseLocale"] as const;
const DEFAULT_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

export type TextDirection = "ltr" | "rtl";
export type LocaleStrategy = "url" | "cookie" | "localStorage" | "header" | "navigator" | "preferredLanguage" | "globalVariable" | "baseLocale" | `custom-${string}`;

export interface AlternateLink {
  rel: "alternate";
  hreflang: string;
  href: string;
}

export interface LinguiniWebOptions {
  strategy?: readonly LocaleStrategy[];
  cookieName?: string;
  localStorageKey?: string;
  prefixDefaultLocale?: boolean;
  basePath?: string;
  trailingSlash?: "ignore" | "always" | "never" | "directory";
  cookiePath?: string;
  cookieDomain?: string;
  cookieMaxAge?: number;
  cookieSameSite?: "lax" | "strict" | "none";
  cookieSecure?: boolean;
  cookieHttpOnly?: boolean;
  globalVariableName?: string;
  exclude?: readonly (string | RegExp | ((url: URL) => boolean))[];
  redirect?: boolean;
  origin?: string;
  localizeLinks?: boolean;
}

export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> {
  locales: readonly Locale[];
  baseLocale: Locale;
  localeDirections?: Readonly<Record<Locale, TextDirection>>;
  createLinguini(locale: Locale): Linguini;
  normalizeLocale?(locale: unknown): Locale | undefined;
  getTextDirection?(locale: Locale): TextDirection;
}

export interface LinguiniRequestContext<Locale extends string = string, Linguini = unknown> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  textDirection: TextDirection;
  lang: Locale;
  messages: Linguini;
  l: Linguini;
  htmlAttrs: { lang: Locale; dir: TextDirection };
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
}

export interface LinguiniWeb<Locale extends string = string, Linguini = unknown> extends LinguiniRuntime<Locale, Linguini> {
  options: Required<Pick<LinguiniWebOptions, "strategy" | "cookieName" | "localStorageKey" | "prefixDefaultLocale" | "basePath" | "trailingSlash" | "cookiePath" | "cookieMaxAge" | "cookieSameSite" | "cookieSecure" | "cookieHttpOnly" | "exclude" | "redirect" | "localizeLinks">> & LinguiniWebOptions;
  matchLocale(locale: unknown): Locale | undefined;
  resolveLocale(input?: Record<string, unknown>): Promise<Locale>;
  resolveLocaleSync(input?: Record<string, unknown>): Locale;
  resolveRequest(request: Request, input?: Record<string, unknown>): Promise<LinguiniRequestContext<Locale, Linguini>>;
  createRequestContext(locale: Locale, input?: Record<string, unknown>): LinguiniRequestContext<Locale, Linguini>;
  localizeUrl(url: string | URL, locale: Locale, input?: Record<string, unknown>): URL;
  localizeHref(href: string, locale: Locale, input?: Record<string, unknown>): string;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  delocalizePathname(pathname: string, input?: Record<string, unknown>): string;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
  htmlAttrs(locale: Locale): { lang: Locale; dir: TextDirection };
  getTextDirection(locale: Locale): TextDirection;
  getCanonicalRedirect(url: string | URL, locale: Locale, input?: Record<string, unknown>): string | undefined;
  shouldExclude(url: string | URL, input?: Record<string, unknown>): boolean;
  setLocaleCookie(target: unknown, locale: Locale, input?: Record<string, unknown>): void;
  serializeLocaleCookie(locale: Locale, input?: Record<string, unknown>): string;
}

export function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions = {}): LinguiniWeb<Locale, Linguini> {
  const normalized = normalizeOptions({ baseLocale: runtime.baseLocale, ...options });
  const matchLocale = (locale: unknown) => runtime.normalizeLocale?.(locale) ?? matchLocaleValue(runtime.locales, locale);
  const getTextDirection = (locale: Locale) => runtime.getTextDirection?.(locale) ?? runtime.localeDirections?.[locale] ?? "ltr";

  function createRequestContext(locale: Locale, contextInput: Record<string, unknown> = {}): LinguiniRequestContext<Locale, Linguini> {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    const messages = runtime.createLinguini(resolved);
    return {
      locale: resolved,
      baseLocale: runtime.baseLocale,
      locales: runtime.locales,
      direction: getTextDirection(resolved),
      textDirection: getTextDirection(resolved),
      lang: resolved,
      messages,
      l: messages,
      htmlAttrs: htmlAttrs(resolved),
      localizeHref: (href, nextLocale = resolved, input = contextInput) => localizeHref(href, nextLocale, input),
      localizeUrl: (url, nextLocale = resolved, input = contextInput) => localizeUrl(url, nextLocale, input),
      shouldLocalizeHref: (href, input = contextInput) => shouldLocalizeHref(href, input),
      localizeHrefAttribute: (href, nextLocale = resolved, input = contextInput) => localizeHrefAttribute(href, nextLocale, input),
      localizeMarkupLinks: (html, nextLocale = resolved, input = contextInput) => localizeMarkupLinks(html, nextLocale, input),
      delocalizeUrl: (url, input = contextInput) => delocalizeUrl(url, input),
      alternateLinks: (url, input = contextInput) => alternateLinks(url, input),
    };
  }

  function resolveLocaleSync(input: Record<string, unknown> = {}) {
    for (const strategy of normalized.strategy) {
      const locale = readStrategy(strategy, input, normalized);
      const resolved = matchLocale(locale);
      if (resolved) return resolved;
    }
    return runtime.baseLocale;
  }

  function localizeUrl(url: string | URL, locale: Locale, input: Record<string, unknown> = {}) {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    const copy = new URL(String(url), String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
    const path = stripBasePath(copy.pathname, normalized.basePath);
    const withoutLocale = stripLeadingLocale(path, runtime.locales, matchLocale);
    copy.pathname = applyTrailingSlash(joinPath(normalized.basePath, shouldPrefixLocale(normalized, resolved) ? joinPath("/", resolved, withoutLocale) : withoutLocale), normalized.trailingSlash);
    return copy;
  }

  function localizeHref(href: string, locale: Locale, input: Record<string, unknown> = {}) {
    const url = localizeUrl(href, locale, input);
    if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(String(href))) return url.toString();
    return `${url.pathname}${url.search}${url.hash}`;
  }

  function shouldLocalizeHref(href: string, input: Record<string, unknown> = {}) {
    if (normalized.localizeLinks === false) return false;
    const value = String(href ?? "").trim();
    if (!value || value.startsWith("#") || value.startsWith("//")) return false;
    const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
    if (scheme && !["http", "https"].includes(scheme[1].toLowerCase())) return false;
    let parsed: URL;
    try {
      parsed = new URL(value, String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
    } catch {
      return false;
    }
    if (!["http:", "https:"].includes(parsed.protocol)) return false;
    if (parsed.origin !== currentOrigin(normalized, input)) return false;
    return !shouldExclude(parsed, input);
  }

  function localizeHrefAttribute(href: string, locale: Locale, input: Record<string, unknown> = {}) {
    return shouldLocalizeHref(href, input) ? localizeHref(href, locale, input) : href;
  }

  function localizeMarkupLinks(html: string, locale: Locale, input: Record<string, unknown> = {}) {
    if (normalized.localizeLinks === false) return String(html);
    return String(html).replace(/<a\b[^>]*>/gi, (tag) => {
      if (/\sdownload(?:\s|=|>)/i.test(tag) || /\sdata-linguini-(?:ignore|no-localize)(?:\s|=|>)/i.test(tag)) return tag;
      return tag.replace(/\bhref\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/i, (attribute, rawValue) => {
        const quote = rawValue[0] === "\"" || rawValue[0] === "'" ? rawValue[0] : "";
        const href = quote ? rawValue.slice(1, -1) : rawValue;
        const localized = localizeHrefAttribute(href, locale, input);
        return localized === href ? attribute : `href=${quote}${localized}${quote}`;
      });
    });
  }

  function delocalizeUrl(url: string | URL, input: Record<string, unknown> = {}) {
    const copy = new URL(String(url), String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
    copy.pathname = joinPath(normalized.basePath, stripLeadingLocale(stripBasePath(copy.pathname, normalized.basePath), runtime.locales, matchLocale));
    return copy;
  }

  function delocalizePathname(pathname: string, input: Record<string, unknown> = {}) {
    return delocalizeUrl(pathname, input).pathname;
  }

  function alternateLinks(url: string | URL, input: Record<string, unknown> = {}) {
    const parsed = new URL(String(url), String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
    const links: AlternateLink[] = runtime.locales.map((locale) => ({
      rel: "alternate" as const,
      hreflang: locale,
      href: localizeUrl(parsed, locale, input).toString(),
    }));
    links.push({ rel: "alternate", hreflang: "x-default", href: localizeUrl(parsed, runtime.baseLocale, input).toString() });
    return links;
  }

  function htmlAttrs(locale: Locale) {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    return { lang: resolved, dir: getTextDirection(resolved) };
  }

  function getCanonicalRedirect(url: string | URL, locale: Locale, input: Record<string, unknown> = {}) {
    if (normalized.redirect === false || shouldExclude(url, input)) return undefined;
    const parsed = new URL(String(url), String(input.origin ?? normalized.origin ?? "http://localhost"));
    const canonical = localizeUrl(parsed, locale, input);
    return canonical.pathname === parsed.pathname ? undefined : `${canonical.pathname}${canonical.search}${canonical.hash}`;
  }

  function shouldExclude(url: string | URL, input: Record<string, unknown> = {}) {
    const parsed = new URL(String(url), String(input.origin ?? normalized.origin ?? "http://localhost"));
    return normalized.exclude.some((matcher) => matchesRoute(matcher, parsed));
  }

  function serializeLocaleCookie(locale: Locale, input: Record<string, unknown> = {}) {
    const parts = [`${normalized.cookieName}=${encodeURIComponent(locale)}`];
    parts.push(`Max-Age=${input.maxAge ?? normalized.cookieMaxAge}`);
    parts.push(`Path=${normalized.cookiePath}`);
    if (normalized.cookieDomain) parts.push(`Domain=${normalized.cookieDomain}`);
    parts.push(`SameSite=${normalized.cookieSameSite}`);
    if (input.secure ?? normalized.cookieSecure) parts.push("Secure");
    if (input.httpOnly ?? normalized.cookieHttpOnly) parts.push("HttpOnly");
    return parts.join("; ");
  }

  function setLocaleCookie(target: unknown, locale: Locale, input: Record<string, unknown> = {}) {
    const cookie = serializeLocaleCookie(locale, input);
    const sink = target as { headers?: Headers; cookies?: { set(name: string, value: string, options?: Record<string, unknown>): void }; setHeaders?: (headers: Record<string, string>) => void };
    if (sink.setHeaders) sink.setHeaders({ "set-cookie": cookie });
    else if (sink.headers?.append) sink.headers.append("set-cookie", cookie);
    else if (sink.cookies?.set) {
      sink.cookies.set(normalized.cookieName, locale, {
        path: normalized.cookiePath,
        domain: normalized.cookieDomain,
        maxAge: input.maxAge ?? normalized.cookieMaxAge,
        sameSite: normalized.cookieSameSite,
        secure: input.secure ?? normalized.cookieSecure,
        httpOnly: input.httpOnly ?? normalized.cookieHttpOnly,
      });
    }
  }

  return {
    ...runtime,
    options: normalized,
    matchLocale,
    resolveLocale: async (input = {}) => resolveLocaleSync(input),
    resolveLocaleSync,
    resolveRequest: async (request, input = {}) => {
      const requestInput = inputFromRequest(request, input);
      return createRequestContext(resolveLocaleSync(requestInput), requestInput);
    },
    createRequestContext,
    localizeUrl,
    localizeHref,
    shouldLocalizeHref,
    localizeHrefAttribute,
    localizeMarkupLinks,
    delocalizeUrl,
    delocalizePathname,
    alternateLinks,
    htmlAttrs,
    getCanonicalRedirect,
    shouldExclude,
    setLocaleCookie,
    serializeLocaleCookie,
    getTextDirection,
  };
}

function inputFromRequest(request: Request, input: Record<string, unknown> = {}) {
  const requestInput: Record<string, unknown> = { ...input };
  requestInput.url ??= request.url;
  requestInput.currentUrl ??= requestInput.url;
  requestInput.headers ??= request.headers;
  requestInput.cookie ??= readHeader(requestInput.headers, "cookie") ?? readHeader(request.headers, "cookie") ?? undefined;
  return requestInput;
}

function readHeader(headers: unknown, name: string) {
  if (!headers) return undefined;
  const getter = (headers as { get?: (header: string) => string | null | undefined }).get;
  if (typeof getter === "function") return getter.call(headers, name) ?? undefined;
  const record = headers as Record<string, unknown>;
  const value = record[name] ?? record[name.toLowerCase()];
  if (Array.isArray(value)) return value.join(", ");
  return typeof value === "string" ? value : undefined;
}

function normalizeOptions(options: LinguiniWebOptions & { baseLocale: string }) {
  return {
    strategy: options.strategy ?? DEFAULT_STRATEGY,
    cookieName: options.cookieName ?? "LINGUINI_LOCALE",
    localStorageKey: options.localStorageKey ?? "LINGUINI_LOCALE",
    prefixDefaultLocale: Boolean(options.prefixDefaultLocale ?? false),
    basePath: options.basePath ?? "",
    trailingSlash: options.trailingSlash ?? "ignore",
    cookiePath: options.cookiePath ?? "/",
    cookieDomain: options.cookieDomain,
    cookieMaxAge: options.cookieMaxAge ?? DEFAULT_COOKIE_MAX_AGE,
    cookieSameSite: options.cookieSameSite ?? "lax",
    cookieSecure: Boolean(options.cookieSecure ?? false),
    cookieHttpOnly: Boolean(options.cookieHttpOnly ?? false),
    globalVariableName: options.globalVariableName,
    exclude: options.exclude ?? [],
    redirect: options.redirect ?? true,
    origin: options.origin,
    localizeLinks: options.localizeLinks ?? true,
    baseLocale: options.baseLocale,
  };
}

function readStrategy(strategy: LocaleStrategy, input: Record<string, unknown>, options: ReturnType<typeof normalizeOptions>) {
  if (strategy === "baseLocale") return options.baseLocale;
  if (strategy === "url") return firstPathSegment(input.url as URL | string | undefined, options.basePath);
  if (strategy === "cookie") return readCookie(input.cookie as string | undefined, options.cookieName);
  if (strategy === "localStorage") {
    try {
      return (input.localStorage as Storage | undefined)?.getItem(options.localStorageKey) ?? undefined;
    } catch {
      return undefined;
    }
  }
  if (strategy === "header") return parseAcceptLanguage((input.headers as Headers | undefined)?.get("accept-language"))[0];
  if (strategy === "navigator") return (input.navigator as Navigator | undefined)?.languages?.[0] ?? (input.navigator as Navigator | undefined)?.language;
  return undefined;
}

function parseAcceptLanguage(header: string | null | undefined): string[] {
  if (!header) return [];
  return String(header)
    .split(",")
    .map((part) => part.trim().split(";")[0])
    .filter(Boolean);
}

function readCookie(source: string | undefined, name: string) {
  for (const cookie of String(source ?? "").split(";")) {
    const [rawName, ...rawValue] = cookie.trim().split("=");
    if (rawName === name) return decodeURIComponent(rawValue.join("="));
  }
  return undefined;
}

function matchLocaleValue<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {
  const candidates = Array.isArray(value) ? value : [value];
  for (const candidate of candidates) {
    if (typeof candidate !== "string") continue;
    let tag = candidate;
    while (tag) {
      const exact = locales.find((locale) => locale.toLowerCase() === tag.toLowerCase());
      if (exact) return exact;
      const dash = tag.lastIndexOf("-");
      tag = dash > 0 ? tag.slice(0, dash) : "";
    }
  }
  return undefined;
}

function firstPathSegment(url: URL | string | undefined, basePath = "") {
  if (!url) return undefined;
  const parsed = new URL(String(url), "http://localhost");
  return stripBasePath(parsed.pathname, basePath).split("/").filter(Boolean)[0];
}

function stripLeadingLocale<Locale extends string>(pathname: string, locales: readonly Locale[], matchLocale: (locale: unknown) => Locale | undefined) {
  const parts = pathname.split("/").filter(Boolean);
  return matchLocale(parts[0]) ? ensureSlash(parts.slice(1).join("/")) : ensureSlash(parts.join("/"));
}

function stripBasePath(pathname: string, basePath: string) {
  const normalizedPath = ensureSlash(pathname);
  const normalizedBase = basePath && basePath !== "/" ? ensureSlash(basePath).replace(/\/$/, "") : "";
  if (!normalizedBase) return normalizedPath;
  if (normalizedPath === normalizedBase) return "/";
  return normalizedPath.startsWith(`${normalizedBase}/`) ? normalizedPath.slice(normalizedBase.length) || "/" : normalizedPath;
}

function joinPath(...parts: Array<string | undefined>) {
  const joined = parts
    .filter((part) => part !== undefined && part !== "")
    .map((part) => String(part).replace(/^\/+|\/+$/g, ""))
    .filter(Boolean)
    .join("/");
  return `/${joined}`.replace(/\/+/g, "/");
}

function ensureSlash(pathname: string) {
  const value = String(pathname || "/");
  return value.startsWith("/") ? value : `/${value}`;
}

function applyTrailingSlash(pathname: string, mode: string) {
  if (pathname === "/") return pathname;
  if (mode === "always") return pathname.endsWith("/") ? pathname : `${pathname}/`;
  if (mode === "never") return pathname.replace(/\/+$/, "");
  return pathname;
}

function shouldPrefixLocale(options: ReturnType<typeof normalizeOptions>, locale: string) {
  return options.prefixDefaultLocale || locale !== options.baseLocale;
}

function matchesRoute(pattern: string | RegExp | ((url: URL) => boolean), url: URL) {
  if (typeof pattern === "function") return Boolean(pattern(url));
  if (pattern instanceof RegExp) return pattern.test(url.pathname);
  if (pattern.endsWith("/**")) return url.pathname.startsWith(pattern.slice(0, -3));
  return url.pathname === pattern;
}

function currentOrigin(options: ReturnType<typeof normalizeOptions>, input: Record<string, unknown> = {}) {
  return new URL(String(input.currentUrl ?? input.url ?? input.origin ?? options.origin ?? "http://localhost")).origin;
}
"###;

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}

fn locale_literals(locales: &[TypeScriptLocaleModule]) -> Vec<String> {
    locales
        .iter()
        .map(|locale| format!("\"{}\"", escape_string(&locale.locale)))
        .collect()
}

fn base_locale_literal(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    let selected = base_locale
        .filter(|base_locale| locales.iter().any(|locale| locale.locale == *base_locale))
        .or_else(|| locales.first().map(|locale| locale.locale.as_str()));

    selected
        .map(|locale| format!("\"{}\"", escape_string(locale)))
        .unwrap_or_else(|| "\"\" as LinguiniLanguageInput".to_owned())
}

fn push_runtime_types(output: &mut String) {
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n");
    output.push_str("  resolveLanguage?: () => LinguiniLanguageInput;\n");
    output.push_str("};\n\n");
}

fn push_locale_fallback_runtime(output: &mut String) {
    output.push_str(
        r#"function localeFallbackTags(locale: string): string[] {
  const tags: string[] = [];
  let tag = locale;
  while (tag) {
    tags.push(tag);
    const dash = tag.lastIndexOf("-");
    if (dash <= 0) break;
    tag = tag.slice(0, dash);
  }
  return tags;
}

"#,
    );
}

fn push_runtime_functions(output: &mut String) {
    output.push_str("\nexport function isLocale(locale: unknown): locale is Locale {\n");
    output.push_str("  return normalizeLocale(locale) !== undefined;\n");
    output.push_str("}\n\n");
    output.push_str("export function normalizeLocale(locale: unknown): Locale | undefined {\n");
    output.push_str("  if (typeof locale !== \"string\") return undefined;\n");
    output.push_str("  for (const tag of localeFallbackTags(locale)) {\n");
    output.push_str(
        "    const exact = locales.find((entry) => entry.toLowerCase() === tag.toLowerCase());\n",
    );
    output.push_str("    if (exact) return exact;\n");
    output.push_str("  }\n");
    output.push_str("  return undefined;\n");
    output.push_str("}\n\n");
    output.push_str("export function getTextDirection(locale: Locale): TextDirection {\n");
    output.push_str("  return localeDirections[normalizeLocale(locale) ?? baseLocale];\n");
    output.push_str("}\n");
}

fn push_runtime_declarations(output: &mut String) {
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n  resolveLanguage?: () => LinguiniLanguageInput;\n};\n");
    output.push('\n');
}

fn push_runtime_function_declarations(output: &mut String) {
    output.push_str("export declare function isLocale(locale: unknown): locale is Locale;\n");
    output.push_str(
        "export declare function normalizeLocale(locale: unknown): Locale | undefined;\n",
    );
    output.push_str("export declare function getTextDirection(locale: Locale): TextDirection;\n");
}

fn web_options_literal(options: &TypeScriptWebOptions) -> String {
    let strategy = js_string_array(&options.strategy);
    let exclude = js_string_array(&options.exclude);
    let mut fields = vec![
        format!("strategy: [{strategy}] as const"),
        format!("cookieName: \"{}\"", escape_string(&options.cookie_name)),
        format!("cookiePath: \"{}\"", escape_string(&options.cookie_path)),
        format!("cookieMaxAge: {}", options.cookie_max_age),
        format!(
            "cookieSameSite: \"{}\"",
            escape_string(&options.cookie_same_site)
        ),
        format!("cookieSecure: {}", js_bool(options.cookie_secure)),
        format!("cookieHttpOnly: {}", js_bool(options.cookie_http_only)),
        format!(
            "localStorageKey: \"{}\"",
            escape_string(&options.local_storage_key)
        ),
        format!(
            "prefixDefaultLocale: {}",
            js_bool(options.prefix_default_locale)
        ),
        format!("basePath: \"{}\"", escape_string(&options.base_path)),
        format!(
            "trailingSlash: \"{}\"",
            escape_string(&options.trailing_slash)
        ),
        format!("redirect: {}", js_bool(options.redirect)),
        format!("exclude: [{exclude}] as const"),
        format!("localizeLinks: {}", js_bool(options.localize_links)),
    ];

    if let Some(cookie_domain) = &options.cookie_domain {
        fields.push(format!(
            "cookieDomain: \"{}\"",
            escape_string(cookie_domain)
        ));
    }
    if let Some(global_variable_name) = &options.global_variable_name {
        fields.push(format!(
            "globalVariableName: \"{}\"",
            escape_string(global_variable_name)
        ));
    }
    if let Some(origin) = &options.origin {
        fields.push(format!("origin: \"{}\"", escape_string(origin)));
    }

    format!("{{ {} }} as const", fields.join(", "))
}

fn js_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|item| format!("\"{}\"", escape_string(item)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn js_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn locale_direction(locale: &str) -> &'static str {
    built_in_text_direction(locale).unwrap_or("ltr")
}
