import { baseLocale, normalizeLocale, type Locale } from "./locale";

export type LinguiniLocaleLoader = (locale: Locale) => void | Promise<void>;

const localeLoaders = new Set<LinguiniLocaleLoader>();

let activeLocale = $state<Locale>(baseLocale);

export function registerLocaleLoader(loader: LinguiniLocaleLoader): () => void {
  localeLoaders.add(loader);
  let disposed = false;
  return () => {
    if (disposed) return;
    disposed = true;
    localeLoaders.delete(loader);
  };
}

export async function prepareLocale(locale: unknown): Promise<Locale> {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  const loaders = [...localeLoaders];
  await Promise.all(loaders.map((loader) => loader(resolved)));
  return resolved;
}

export function getCurrentLocale(): Locale {
  return activeLocale;
}

export function setCurrentLocale(locale: unknown): Locale {
  activeLocale = normalizeLocale(locale) ?? baseLocale;
  return activeLocale;
}
