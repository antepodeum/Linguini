export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};

const pendingLocales = new Map<Locale, Promise<Linguini>>();

export async function prepareLinguini(language: LinguiniLanguageInput): Promise<Linguini> {
  const locale = normalizeLocale(language) ?? baseLocale;
  const available = localeModules[locale];
  if (available) return available;
  const pending = pendingLocales.get(locale);
  if (pending) return pending;
  const task = localeLoaders[locale]().then((loaded) => {
    localeModules[locale] = loaded;
    return loaded;
  }).finally(() => {
    pendingLocales.delete(locale);
  });
  pendingLocales.set(locale, task);
  return task;
}

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  const locale = normalizeLocale(language) ?? baseLocale;
  const messages = localeModules[locale];
  if (messages) return messages;
  throw new Error(`Linguini: locale ${JSON.stringify(locale)} is not prepared; call prepareLinguini(locale) first`);
}

export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {
  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);
  return new Proxy({} as Linguini, {
    get(_target, property) {
      return createLinguini(resolve())[property as keyof Linguini];
    },
  });
}

export function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini {
  if (typeof options.language === "function") {
    return createLinguiniProvider({ resolveLanguage: options.language });
  }
  return createLinguini(options.language);
}

export const lgl: Linguini = createLinguini(baseLocale);
