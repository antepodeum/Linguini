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
    output.push_str("export function createLinguini(language: LinguiniLanguage): Linguini {\n");
    output.push_str("  return localeModules[language];\n");
    output.push_str("}\n\n");
    output.push_str("export function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguage | (() => LinguiniLanguage);\n");
    output.push_str("}): { readonly lgl: Linguini } {\n");
    output.push_str("  return {\n");
    output.push_str("    get lgl() {\n");
    output.push_str("      const language = typeof options.language === \"function\" ? options.language() : options.language;\n");
    output.push_str("      return createLinguini(language);\n");
    output.push_str("    },\n");
    output.push_str("  };\n");
    output.push_str("}\n");
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
    output.push_str(
        "export declare function createLinguini(language: LinguiniLanguage): Linguini;\n\n",
    );
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguage | (() => LinguiniLanguage);\n");
    output.push_str("}): { readonly lgl: Linguini };\n");
    output
}

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}
