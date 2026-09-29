import { resolveLocalStorageLocale } from "./web/local-storage.js";

const locales = /** @type {const} */ (["en", "de", "fr"]);
/** @param {unknown} value */
const matchLocale = (value) => locales.find((locale) => locale === value);
const options = { localStorage: { key: "locale" } };
const storage = /** @type {Storage} */ (/** @type {unknown} */ ({ getItem: () => "de" }));
if (resolveLocalStorageLocale({ localStorage: storage }, options, matchLocale) !== "de") {
  throw new Error("local-storage locale did not resolve");
}

if (false) {
  // @ts-expect-error input must be a record
  resolveLocalStorageLocale("de", options, matchLocale);
  // @ts-expect-error storage key must be a string
  resolveLocalStorageLocale({}, { localStorage: { key: 4 } }, matchLocale);
  // @ts-expect-error matcher must return a string locale
  resolveLocalStorageLocale({}, options, () => 4);
}
