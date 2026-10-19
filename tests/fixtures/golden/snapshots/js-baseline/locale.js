export const locales = /** @type {const} */ (["ru"]);
export const baseLocale = "ru";

export const localeDirections = /** @type {const} */ ({
  ru: "ltr",
});

/** @typedef {(typeof locales)[number]} Locale */
/** @typedef {"ltr" | "rtl"} TextDirection */

/** @type {Readonly<Record<string, Locale>>} */
const localeResolution = {
  "ru": "ru",
  "ru-cyrl": "ru",
  "rus": "ru",
};

/**
 * @param {unknown} locale
 * @returns {locale is Locale}
 */
export function isLocale(locale) {
  return normalizeLocale(locale) !== undefined;
}

/**
 * @param {unknown} locale
 * @returns {Locale | undefined}
 */
export function normalizeLocale(locale) {
  if (typeof locale !== "string") return undefined;
  return localeResolution[locale.toLowerCase()];
}

/**
 * @param {Locale} locale
 * @returns {TextDirection}
 */
export function getTextDirection(locale) {
  return localeDirections[normalizeLocale(locale) ?? baseLocale];
}
