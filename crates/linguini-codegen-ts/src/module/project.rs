use super::names::{escape_string, property_key, safe_identifier};
use super::TypeScriptLocaleModule;

pub fn generate_project_index(locales: &[TypeScriptLocaleModule]) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("const localeModules = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: {},\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    output
        .push_str("export function createLinguini(language: LinguiniLanguageInput): Linguini {\n");
    output.push_str("  return localeModules[language as LinguiniLanguage];\n");
    output.push_str("}\n\n");
    output.push_str("export function createLinguiniProvider(options: {\n");
    output.push_str("  resolveLanguage: () => LinguiniLanguageInput;\n");
    output.push_str("}): Linguini {\n");
    output.push_str("  return new Proxy({} as Linguini, {\n");
    output.push_str("    get(_target, property) {\n");
    output.push_str(
        "      return createLinguini(options.resolveLanguage())[property as keyof Linguini];\n",
    );
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
    output.push_str(&default_language_literal(locales));
    output.push_str(");\n");
    output
}

pub fn generate_project_index_declaration(locales: &[TypeScriptLocaleModule]) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("declare const localeModules: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: typeof {};\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    output.push_str(
        "export declare function createLinguini(language: LinguiniLanguageInput): Linguini;\n\n",
    );
    output.push_str("export declare function createLinguiniProvider(options: {\n");
    output.push_str("  resolveLanguage: () => LinguiniLanguageInput;\n");
    output.push_str("}): Linguini;\n\n");
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini;\n\n");
    output.push_str("export declare const lgl: Linguini;\n");
    output
}

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}

fn default_language_literal(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .first()
        .map(|locale| format!("\"{}\"", escape_string(&locale.locale)))
        .unwrap_or_else(|| "\"\" as LinguiniLanguageInput".to_owned())
}
