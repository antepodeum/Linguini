import { resolveCookieLocale } from "./web/cookie.js";

const locales = /** @type {const} */ (["en", "de", "fr"]);
/** @param {unknown} value */
const matchLocale = (value) => locales.find((locale) => locale === value);
const options = { cookie: { name: "LINGUINI_LOCALE" } };
if (resolveCookieLocale({ cookie: "LINGUINI_LOCALE=de" }, options, matchLocale) !== "de") {
  throw new Error("cookie locale did not resolve");
}

if (false) {
  // @ts-expect-error input must be a record
  resolveCookieLocale("LINGUINI_LOCALE=de", options, matchLocale);
  // @ts-expect-error cookie name must be a string
  resolveCookieLocale({}, { cookie: { name: 4 } }, matchLocale);
  // @ts-expect-error matcher must return a string locale
  resolveCookieLocale({}, options, () => 4);
}
