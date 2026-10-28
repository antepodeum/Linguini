import type {LocaleRuntime,LocaleState,LocaleStateOptions} from './types.js';

export function createLocaleState<Locale extends string>(runtime: LocaleRuntime<Locale>, options: LocaleStateOptions = {}): LocaleState<Locale> {
  let clientLocale = $state(runtime.baseLocale);
  let hasClientOverride = $state(false);
  let overrideRequest = $state.raw<unknown>(undefined);
  const loaders = new Set<(locale:Locale) => void|Promise<void>>();
  const normalize = (value:unknown) => runtime.normalizeLocale(value) ?? runtime.baseLocale;
  function getCurrentLocale(): Locale {
    const requestLocale = options.getRequestLocale?.();
    // Server rendering always reads request context; a client override cannot leak between requests.
    if (typeof window === 'undefined') return normalize(requestLocale);
    const currentOverride = hasClientOverride
      && (!options.getRequestIdentity || options.getRequestIdentity() === overrideRequest);
    return currentOverride ? clientLocale : runtime.normalizeLocale(requestLocale) ?? clientLocale;
  }
  function setCurrentLocale(value:unknown): Locale {
    const locale = normalize(value);
    if (typeof window !== 'undefined') {
      overrideRequest = options.getRequestIdentity?.();
      clientLocale = locale;
      hasClientOverride = true;
    }
    return locale;
  }
  function initializeCurrentLocale(value:unknown): Locale {
    const locale = normalize(value);
    if (typeof window !== 'undefined') {
      clientLocale = locale;
      hasClientOverride = false;
      overrideRequest = undefined;
    }
    return locale;
  }
  function registerLocaleLoader(loader:(locale:Locale) => void|Promise<void>): () => void {
    loaders.add(loader);
    let disposed = false;
    return () => {
      if (disposed) return;
      disposed = true;
      loaders.delete(loader);
    };
  }
  async function prepareLocale(value:unknown): Promise<Locale> {
    const locale = normalize(value);
    await Promise.all([...loaders].map(loader => loader(locale)));
    return locale;
  }
  return {getCurrentLocale,setCurrentLocale,initializeCurrentLocale,registerLocaleLoader,prepareLocale,
    clearCurrentLocaleOverride() {hasClientOverride=false;overrideRequest=undefined;},
  };
}
