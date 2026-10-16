//! Project locale metadata and resolution with one runtime for both targets.

use super::{
    base_locale_literal, locale_literals, project_locale_directions, project_locale_resolution,
    EcmaScriptTarget, TypeScriptLocaleModule,
};
use crate::ecmascript::RuntimeCode;

pub(super) fn render(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
    target: EcmaScriptTarget,
) -> String {
    let mut code = RuntimeCode::default();
    code.shared("export const locales = ");
    code.typed("[", "/** @type {const} */ ([");
    code.generated(locale_literals(locales).join(", "));
    code.typed("] as const;\n", "]);\n");
    code.generated(format!(
        "export const baseLocale = {};\n\n",
        base_locale_literal(locales, base_locale)
    ));
    code.shared("export const localeDirections = ");
    code.typed("{\n", "/** @type {const} */ ({\n");
    code.generated(project_locale_directions(locales));
    code.typed("} as const;\n\n", "});\n\n");
    code.typed(
        "export type Locale = (typeof locales)[number];\nexport type TextDirection = \"ltr\" | \"rtl\";\n\nconst localeResolution: Readonly<Record<string, Locale>> = {\n",
        "/** @typedef {(typeof locales)[number]} Locale */\n/** @typedef {\"ltr\" | \"rtl\"} TextDirection */\n\n/** @type {Readonly<Record<string, Locale>>} */\nconst localeResolution = {\n",
    );
    code.generated(project_locale_resolution(locales));
    code.shared("};\n\n");
    code.typed(
        "export function isLocale(locale: unknown): locale is Locale {\n",
        "/**\n * @param {unknown} locale\n * @returns {locale is Locale}\n */\nexport function isLocale(locale) {\n",
    );
    code.shared("  return normalizeLocale(locale) !== undefined;\n}\n\n");
    code.typed(
        "export function normalizeLocale(locale: unknown): Locale | undefined {\n",
        "/**\n * @param {unknown} locale\n * @returns {Locale | undefined}\n */\nexport function normalizeLocale(locale) {\n",
    );
    code.shared("  if (typeof locale !== \"string\") return undefined;\n  return localeResolution[locale.toLowerCase()];\n}\n\n");
    code.typed(
        "export function getTextDirection(locale: Locale): TextDirection {\n",
        "/**\n * @param {Locale} locale\n * @returns {TextDirection}\n */\nexport function getTextDirection(locale) {\n",
    );
    code.shared("  return localeDirections[normalizeLocale(locale) ?? baseLocale];\n}\n");
    code.render(target)
}
