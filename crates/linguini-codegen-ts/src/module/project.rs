use super::names::{escape_string, property_key, safe_identifier};
use super::TypeScriptLocaleModule;

pub fn generate_project_index(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("export const locales = [");
    output.push_str(&locale_literals(locales).join(", "));
    output.push_str("] as const;\n");
    output.push_str("export const baseLocale = ");
    output.push_str(&base_locale_literal(locales, base_locale));
    output.push_str(";\n\n");
    output.push_str("export const localeModules = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: {},\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export const localeLoaders = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: () => Promise.resolve({}),\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Locale = (typeof locales)[number];\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_types(&mut output);
    output
        .push_str("export function createLinguini(language: LinguiniLanguageInput): Linguini {\n");
    output.push_str("  return localeModules[language as LinguiniLanguage];\n");
    output.push_str("}\n\n");
    output.push_str("export function getLocale(): Locale {\n");
    output.push_str("  return activeLocale ?? baseLocale;\n");
    output.push_str("}\n\n");
    output.push_str("export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {\n");
    output
        .push_str("  const resolve = options.getLocale ?? options.resolveLanguage ?? getLocale;\n");
    output.push_str("  return new Proxy({} as Linguini, {\n");
    output.push_str("    get(_target, property) {\n");
    output.push_str("      return createLinguini(resolve())[property as keyof Linguini];\n");
    output.push_str("    },\n");
    output.push_str("  });\n");
    output.push_str("}\n\n");
    output.push_str("export function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini {\n");
    output.push_str("  if (typeof options.language === \"function\") {\n");
    output.push_str("    return createLinguiniProvider({ resolveLanguage: options.language });\n");
    output.push_str("  }\n");
    output.push_str("  return createLinguini(options.language);\n");
    output.push_str("}\n\n");
    output.push_str("export const lgl: Linguini = createLinguini(");
    output.push_str("baseLocale");
    output.push_str(");\n");
    push_runtime_functions(&mut output);
    output
}

pub fn generate_project_index_declaration(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("export declare const locales: readonly [");
    output.push_str(&locale_literals(locales).join(", "));
    output.push_str("];\n");
    output.push_str("export declare const baseLocale: ");
    output.push_str(&base_locale_literal(locales, base_locale));
    output.push_str(";\n\n");
    output.push_str("export declare const localeModules: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: typeof {};\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("export declare const localeLoaders: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: () => Promise<typeof {}>;\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Locale = (typeof locales)[number];\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_declarations(&mut output);
    output.push_str(
        "export declare function createLinguini(language: LinguiniLanguageInput): Linguini;\n\n",
    );
    output.push_str("export declare function getLocale(): Locale;\n\n");
    output.push_str(
        "export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;\n\n",
    );
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini;\n\n");
    output.push_str("export declare const lgl: Linguini;\n");
    output.push('\n');
    push_runtime_function_declarations(&mut output);
    output
}

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}

fn locale_literals(locales: &[TypeScriptLocaleModule]) -> Vec<String> {
    locales
        .iter()
        .map(|locale| format!("\"{}\"", escape_string(&locale.locale)))
        .collect()
}

fn base_locale_literal(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    let selected = base_locale
        .filter(|base_locale| locales.iter().any(|locale| locale.locale == *base_locale))
        .or_else(|| locales.first().map(|locale| locale.locale.as_str()));

    selected
        .map(|locale| format!("\"{}\"", escape_string(locale)))
        .unwrap_or_else(|| "\"\" as LinguiniLanguageInput".to_owned())
}

fn push_runtime_types(output: &mut String) {
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n");
    output.push_str("  resolveLanguage?: () => LinguiniLanguageInput;\n");
    output.push_str("};\n");
    output.push_str("let activeLocale: Locale = baseLocale;\n");
    output.push('\n');
}

fn push_runtime_functions(output: &mut String) {
    output.push_str("\nexport function setLocale(locale: LinguiniLanguageInput): Locale {\n");
    output.push_str("  const resolved = normalizeLocale(locale) ?? baseLocale;\n");
    output.push_str("  activeLocale = resolved;\n");
    output.push_str("  return activeLocale;\n");
    output.push_str("}\n\n");
    push_private_runtime_helpers(output);
}

fn push_private_runtime_helpers(output: &mut String) {
    output.push_str("function normalizeLocale(locale: unknown): Locale | undefined {\n");
    output.push_str("  if (typeof locale !== \"string\") return undefined;\n");
    output.push_str("  if (locales.includes(locale as Locale)) return locale as Locale;\n");
    output.push_str("  const language = locale.toLowerCase().split(\"-\")[0];\n");
    output.push_str("  return locales.find((candidate) => candidate.toLowerCase() === language || candidate.toLowerCase().startsWith(`${language}-`));\n");
    output.push_str("}\n\n");
}

fn push_runtime_declarations(output: &mut String) {
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n  resolveLanguage?: () => LinguiniLanguageInput;\n};\n");
    output.push('\n');
}

fn push_runtime_function_declarations(output: &mut String) {
    output.push_str("export declare function setLocale(locale: LinguiniLanguageInput): Locale;\n");
}
