import locale_ru from "./locales/ru.js";
import { baseLocale, normalizeLocale } from "./locale.js";
export {
  locales,
  baseLocale,
  localeDirections,
  isLocale,
  normalizeLocale,
  getTextDirection,
} from "./locale.js";

/** @typedef {import("./locale.js").Locale} Locale */
/** @typedef {typeof locale_ru} LinguiniMessages */
/** @typedef {LinguiniMessages} Linguini */
/**
 * @typedef {object} LinguiniProviderOptions
 * @property {(() => Locale)=} getLocale
 * @property {(() => Locale)=} resolveLanguage
 */

/** @type {Partial<Record<Locale, LinguiniMessages>>} */
export const localeModules = {
  ru: locale_ru,
};

/** @type {Record<Locale, () => Promise<LinguiniMessages>>} */
export const localeLoaders = {
  ru: () => Promise.resolve(locale_ru),
};

/** @type {Map<Locale, Promise<Linguini>>} */
const pendingLocales = new Map();

/**
 * @param {Locale} language
 * @returns {Promise<Linguini>}
 */
export async function prepareLinguini(language) {
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

/**
 * @param {Locale} language
 * @returns {Linguini}
 */
export function createLinguini(language) {
  const locale = normalizeLocale(language) ?? baseLocale;
  const messages = localeModules[locale];
  if (messages) return messages;
  throw new Error(`Linguini: locale ${JSON.stringify(locale)} is not prepared; call prepareLinguini(locale) first`);
}

/**
 * @param {LinguiniProviderOptions} [options]
 * @returns {Linguini}
 */
export function createLinguiniProvider(options = {}) {
  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);
  return new Proxy(/** @type {Linguini} */ ({}), {
    get(_target, property) {
      return createLinguini(resolve())[/** @type {keyof Linguini} */ (property)];
    },
  });
}

/**
 * @param {{language: Locale | (() => Locale)}} options
 * @returns {Linguini}
 */
export function configureLinguini(options) {
  if (typeof options.language === "function") {
    return createLinguiniProvider({ resolveLanguage: options.language });
  }
  return createLinguini(options.language);
}

/** @type {Linguini} */
export const lgl = createLinguini(baseLocale);
