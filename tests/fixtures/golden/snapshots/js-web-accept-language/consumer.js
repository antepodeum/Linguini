import { resolveAcceptLanguageLocale } from "./web/accept-language.js";

const locales = /** @type {const} */ (["en", "de", "fr"]);
/** @param {unknown} value */
const matchLocale = (value) => locales.find((locale) => locale === value);
const input = { headers: new Headers({ "accept-language": "fr, de;q=0.8" }) };
if (resolveAcceptLanguageLocale(input, locales, "en", matchLocale) !== "fr") {
  throw new Error("Accept-Language locale did not resolve");
}

if (false) {
  // @ts-expect-error input must be a record
  resolveAcceptLanguageLocale("fr", locales, "en", matchLocale);
  // @ts-expect-error locales must be strings
  resolveAcceptLanguageLocale(input, [4], "en", matchLocale);
  // @ts-expect-error matcher must return a string locale
  resolveAcceptLanguageLocale(input, locales, "en", () => 4);
}
