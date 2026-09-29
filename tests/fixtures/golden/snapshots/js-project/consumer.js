import {
  baseLocale,
  configureLinguini,
  createLinguini,
  createLinguiniProvider,
  getTextDirection,
  isLocale,
  localeLoaders,
  localeModules,
  locales,
  normalizeLocale,
  prepareLinguini,
} from "./index.js";

/**
 * @param {unknown} actual
 * @param {unknown} expected
 */
function assertEqual(actual, expected) {
  if (actual !== expected) {
    throw new Error(`Expected ${String(expected)}, received ${String(actual)}`);
  }
}

assertEqual(baseLocale, "en");
assertEqual(locales.join(","), "en,fr");
assertEqual(normalizeLocale("FR-latn"), "fr");
assertEqual(isLocale("en-AU"), true);
assertEqual(getTextDirection("fr"), "ltr");
assertEqual(createLinguini("en").hello("Ada"), "Hello Ada");
assertEqual(configureLinguini({ language: "en" }).hello("Ada"), "Hello Ada");
assertEqual(createLinguiniProvider().hello("Ada"), "Hello Ada");
assertEqual(localeModules.fr, undefined);
void localeLoaders.fr;
void prepareLinguini("fr");

if (false) {
  // @ts-expect-error only generated locale identities are accepted
  createLinguini("de");
  // @ts-expect-error loader input retains the locale union
  prepareLinguini("de");
  // @ts-expect-error message parameters retain their schema type
  createLinguini("en").hello(42);
  // @ts-expect-error provider callbacks retain the locale union
  createLinguiniProvider({ getLocale: () => "de" });
  // @ts-expect-error configure callbacks retain the locale union
  configureLinguini({ language: () => "de" });
}
