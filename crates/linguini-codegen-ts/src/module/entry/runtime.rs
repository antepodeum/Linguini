//! Shared lazy locale preparation and message provider runtime.

use crate::ecmascript::EcmaScriptTarget;
use crate::ecmascript::RuntimeCode;

pub(super) fn runtime(target: EcmaScriptTarget) -> String {
    let mut code = RuntimeCode::default();
    code.typed(
        "export type LinguiniProviderOptions = {\n  getLocale?: () => LinguiniLanguageInput;\n  resolveLanguage?: () => LinguiniLanguageInput;\n};\n\n",
        "",
    );
    code.typed(
        "const pendingLocales = new Map<Locale, Promise<Linguini>>();\n\n",
        "/** @type {Map<Locale, Promise<Linguini>>} */\nconst pendingLocales = new Map();\n\n",
    );
    code.typed(
        "export async function prepareLinguini(language: LinguiniLanguageInput): Promise<Linguini> {\n",
        "/**\n * @param {Locale} language\n * @returns {Promise<Linguini>}\n */\nexport async function prepareLinguini(language) {\n",
    );
    code.shared(
        r#"  const locale = normalizeLocale(language) ?? baseLocale;
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

"#,
    );
    code.typed(
        r#"export function createLinguini(language: LinguiniLanguageInput): Linguini {
"#,
        r#"/**
 * @param {Locale} language
 * @returns {Linguini}
 */
export function createLinguini(language) {
"#,
    );
    code.shared(r#"  const locale = normalizeLocale(language) ?? baseLocale;
  const messages = localeModules[locale];
  if (messages) return messages;
  throw new Error(`Linguini: locale ${JSON.stringify(locale)} is not prepared; call prepareLinguini(locale) first`);
}

"#);
    code.typed(
        r#"export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {
"#,
        r#"/**
 * @param {LinguiniProviderOptions} [options]
 * @returns {Linguini}
 */
export function createLinguiniProvider(options = {}) {
"#,
    );
    code.shared(
        r#"  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);
"#,
    );
    code.typed(
        r#"  return new Proxy({} as Linguini, {
"#,
        r#"  return new Proxy(/** @type {Linguini} */ ({}), {
"#,
    );
    code.shared(
        r#"    get(_target, property) {
"#,
    );
    code.typed(
        r#"      return createLinguini(resolve())[property as keyof Linguini];
"#,
        r#"      return createLinguini(resolve())[/** @type {keyof Linguini} */ (property)];
"#,
    );
    code.shared(
        r#"    },
  });
}

"#,
    );
    code.typed(
        r#"export function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini {
"#,
        r#"/**
 * @param {{language: Locale | (() => Locale)}} options
 * @returns {Linguini}
 */
export function configureLinguini(options) {
"#,
    );
    code.shared(
        r#"  if (typeof options.language === "function") {
    return createLinguiniProvider({ resolveLanguage: options.language });
  }
  return createLinguini(options.language);
}

"#,
    );
    code.typed(
        r#"export const lgl: Linguini = createLinguini(baseLocale);
"#,
        r#"/** @type {Linguini} */
export const lgl = createLinguini(baseLocale);
"#,
    );
    code.render(target)
}
