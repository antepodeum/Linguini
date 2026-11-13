import type { LinguiniWebLocale, LinkLocalizationAttributes } from './types.js';

export interface LocaleState<Locale extends string> {
  getCurrentLocale(): Locale;
  setCurrentLocale(value: unknown): Locale;
  prepareLocale(value: unknown): Promise<Locale>;
  clearCurrentLocaleOverride?(): void;
}
export interface SetLocaleOptions {
  cookie?: boolean;
  navigate?: boolean;
  replaceState?: boolean;
  invalidateAll?: boolean;
  keepFocus?: boolean;
  noScroll?: boolean;
  state?: Record<string, unknown>;
}
export interface LocaleControlOptions<Locale extends string> {
  browser?: boolean;
  /** Replacing the authoritative request cancels changes that are still being prepared. */
  getRequestIdentity?: () => unknown;
  navigate?: (href: string, options: SetLocaleOptions) => void | Promise<void>;
  persistLocale?: (locale: Locale) => void | Promise<void>;
  refresh?: () => void;
  destroy?: () => void;
}

export function createLocaleControl<Locale extends string>(web: LinguiniWebLocale<Locale>, state: LocaleState<Locale>, options: LocaleControlOptions<Locale> = {}) {
  const browser = options.browser ?? (typeof window !== 'undefined' && typeof document !== 'undefined');
  let latestChange = 0;
  let destroyed = false;
  function isCurrent(change: number, request: unknown): boolean {
    return !destroyed && change === latestChange
      && (!options.getRequestIdentity || options.getRequestIdentity() === request);
  }
  async function setLocale(value: unknown, input: SetLocaleOptions = {}): Promise<Locale> {
    if (browser && destroyed) return state.getCurrentLocale();
    const change = browser ? ++latestChange : 0;
    const request = browser ? options.getRequestIdentity?.() : undefined;
    const locale = await state.prepareLocale(value);
    if (!browser) return locale;
    if (!isCurrent(change,request)) return state.getCurrentLocale();
    if (input.cookie !== false && web.options.locale.switch.writesCookie && web.options.cookie.httpOnly && !options.persistLocale) {
      throw new TypeError('HttpOnly locale switching requires a server persistence callback');
    }
    if (input.cookie !== false && web.options.locale.switch.writesCookie && options.persistLocale) {
      await options.persistLocale(locale);
      if (!isCurrent(change,request)) return state.getCurrentLocale();
    }
    state.setCurrentLocale(locale);
    if (web.options.locale.switch.writesLocalStorage) {
      try { window.localStorage.setItem(web.options.localStorage.key, locale); } catch { /* Storage can be denied. */ }
    }
    if (input.cookie !== false && web.options.locale.switch.writesCookie && !options.persistLocale) {
      try { document.cookie = web.serializeLocaleCookie(locale, {httpOnly:false}); } catch { /* Cookies can be denied. */ }
    }
    if (input.navigate !== false && web.options.locale.switch.writesPath) {
      const href = web.localizeHref(window.location.href, locale);
      if (options.navigate) {
        await options.navigate(href, input);
        if (destroyed || change !== latestChange) return state.getCurrentLocale();
        state.clearCurrentLocaleOverride?.();
      } else if (input.replaceState) window.location.replace(href);
      else window.location.assign(href);
    }
    options.refresh?.();
    return locale;
  }
  return {
    get locale() { return state.getCurrentLocale(); },
    get lang() { return state.getCurrentLocale(); },
    get direction() { return web.getTextDirection(state.getCurrentLocale()); },
    get textDirection() { return web.getTextDirection(state.getCurrentLocale()); },
    get htmlAttrs() { return web.htmlAttrs(state.getCurrentLocale()); },
    setLocale,
    localizeHref: (href:string, locale = state.getCurrentLocale(), input?:Record<string,unknown>) => web.localizeHref(href,locale,input),
    localizeUrl: (url:string|URL, locale = state.getCurrentLocale(), input?:Record<string,unknown>) => web.localizeUrl(url,locale,input),
    shouldLocalizeHref: web.shouldLocalizeHref,
    shouldLocalizeLink: web.shouldLocalizeLink,
    localizeHrefAttribute: (href:string, locale = state.getCurrentLocale(), input?:Record<string,unknown>) => web.localizeHrefAttribute(href,locale,input),
    localizeLinkHref: (href:string, locale = state.getCurrentLocale(), attributes?:LinkLocalizationAttributes, input?:Record<string,unknown>) => web.localizeLinkHref(href,locale,attributes,input),
    delocalizeUrl: web.delocalizeUrl,
    alternateLinks: web.alternateLinks,
    destroy() {destroyed=true;latestChange++;options.destroy?.();},
  };
}
