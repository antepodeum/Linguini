export const locales = /** @type {const} */ (["en", "fr"]);
export const baseLocale = "en";

export const localeDirections = /** @type {const} */ ({
  en: "ltr",
  fr: "ltr",
});

/** @typedef {(typeof locales)[number]} Locale */
/** @typedef {"ltr" | "rtl"} TextDirection */

/** @type {Readonly<Record<string, Locale>>} */
const localeResolution = {
  "en": "en",
  "en-150": "en",
  "en-ag": "en",
  "en-ai": "en",
  "en-at": "en",
  "en-au": "en",
  "en-bb": "en",
  "en-be": "en",
  "en-bm": "en",
  "en-bs": "en",
  "en-bw": "en",
  "en-bz": "en",
  "en-cc": "en",
  "en-ch": "en",
  "en-ck": "en",
  "en-cm": "en",
  "en-cx": "en",
  "en-cy": "en",
  "en-cz": "en",
  "en-de": "en",
  "en-dg": "en",
  "en-dk": "en",
  "en-dm": "en",
  "en-ee": "en",
  "en-er": "en",
  "en-es": "en",
  "en-fi": "en",
  "en-fj": "en",
  "en-fk": "en",
  "en-fm": "en",
  "en-fr": "en",
  "en-gb": "en",
  "en-gb-oed": "en",
  "en-gd": "en",
  "en-ge": "en",
  "en-gg": "en",
  "en-gh": "en",
  "en-gi": "en",
  "en-gm": "en",
  "en-gs": "en",
  "en-gy": "en",
  "en-hk": "en",
  "en-hu": "en",
  "en-id": "en",
  "en-ie": "en",
  "en-il": "en",
  "en-im": "en",
  "en-in": "en",
  "en-io": "en",
  "en-it": "en",
  "en-je": "en",
  "en-jm": "en",
  "en-ke": "en",
  "en-ki": "en",
  "en-kn": "en",
  "en-ky": "en",
  "en-latn": "en",
  "en-lc": "en",
  "en-lr": "en",
  "en-ls": "en",
  "en-lt": "en",
  "en-lv": "en",
  "en-mg": "en",
  "en-mo": "en",
  "en-ms": "en",
  "en-mt": "en",
  "en-mu": "en",
  "en-mv": "en",
  "en-mw": "en",
  "en-my": "en",
  "en-na": "en",
  "en-nf": "en",
  "en-ng": "en",
  "en-nl": "en",
  "en-no": "en",
  "en-nr": "en",
  "en-nu": "en",
  "en-nz": "en",
  "en-pg": "en",
  "en-pk": "en",
  "en-pl": "en",
  "en-pn": "en",
  "en-pt": "en",
  "en-pw": "en",
  "en-ro": "en",
  "en-rw": "en",
  "en-sb": "en",
  "en-sc": "en",
  "en-sd": "en",
  "en-se": "en",
  "en-sg": "en",
  "en-sh": "en",
  "en-si": "en",
  "en-sk": "en",
  "en-sl": "en",
  "en-ss": "en",
  "en-sx": "en",
  "en-sz": "en",
  "en-tc": "en",
  "en-tk": "en",
  "en-to": "en",
  "en-tt": "en",
  "en-tv": "en",
  "en-tz": "en",
  "en-ua": "en",
  "en-ug": "en",
  "en-vc": "en",
  "en-vg": "en",
  "en-vu": "en",
  "en-ws": "en",
  "en-za": "en",
  "en-zm": "en",
  "en-zw": "en",
  "eng": "en",
  "fr": "fr",
  "fr-latn": "fr",
  "fra": "fr",
  "fre": "fr",
  "hat": "fr",
  "hi-latn": "en",
  "ht": "fr",
  "i-default": "en",
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
//# sourceMappingURL=locale.js.map
