//! Accept-Language locale-resolution leaf for generated web runtimes.

use crate::ecmascript::{
    EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

const RESOLVE_BODY: &str = r#"  const header = readHeader(input.headers, "accept-language");
  const value = header !== undefined
    ? resolveAcceptLanguage(locales, baseLocale, header)
    : resolveNavigatorLanguage(locales, baseLocale, input.navigator);
  return matchLocale(value);
}

"#;

const READ_HEADER_AFTER_GETTER: &str = r#"  if (typeof getter !== "function") return undefined;
  try {
    return getter.call(headers, name) ?? undefined;
  } catch {
    return undefined;
  }
}

"#;

const PARSE_BODY: &str = r#"  if (!header) return [];
  return String(header).split(",").flatMap((part, index) => {
    const [rawRange, ...parameters] = part.split(";");
    const range = rawRange.trim();
    if (!/^(?:\*|[A-Za-z]{1,8}(?:-[A-Za-z0-9]{1,8})*)$/.test(range)) return [];
    let quality = 1;
    for (const parameter of parameters) {
      const [rawName, ...rawValue] = parameter.split("=");
      if (rawName.trim().toLowerCase() !== "q") continue;
      const value = rawValue.join("=").trim();
      if (!/^(?:0(?:\.\d{0,3})?|1(?:\.0{0,3})?)$/.test(value)) return [];
      quality = Number(value);
      break;
    }
    return [{ range, quality, index, specificity: range === "*" ? 0 : range.split("-").length }];
  });
}

"#;

const ACCEPT_BEFORE_BEST: &str = r#"  const preferences = parseAcceptLanguage(header);
"#;
const ACCEPT_AFTER_BEST: &str = r#"  for (const [localeIndex, locale] of locales.entries()) {
    const preference = preferences
      .filter((candidate) => languageRangeMatches(candidate.range, locale))
      .sort((left, right) => right.specificity - left.specificity || left.index - right.index)[0];
    if (!preference || preference.quality <= 0) continue;
    const candidate = { locale, quality: preference.quality, preferenceIndex: preference.index, base: locale.toLowerCase() === baseLocale.toLowerCase(), localeIndex };
    if (!best || candidate.quality > best.quality ||
      (candidate.quality === best.quality && candidate.preferenceIndex < best.preferenceIndex) ||
      (candidate.quality === best.quality && candidate.preferenceIndex === best.preferenceIndex && candidate.base && !best.base) ||
      (candidate.quality === best.quality && candidate.preferenceIndex === best.preferenceIndex && candidate.base === best.base && candidate.localeIndex < best.localeIndex)) best = candidate;
  }
  return best?.locale;
}

"#;

const NAVIGATOR_BEFORE_PREFERENCES: &str = r#"  if (!value || (typeof value !== "object" && typeof value !== "function")) return undefined;
"#;
const NAVIGATOR_BEFORE_LANGUAGES: &str = r#"  try {
    const languages = "#;
const NAVIGATOR_AFTER_LANGUAGES: &str = r#";
    if (Array.isArray(languages)) for (const language of languages) if (typeof language === "string") preferences.push(language);
  } catch {}
  try {
    const language = "#;
const NAVIGATOR_AFTER_LANGUAGE: &str = r#";
    if (typeof language === "string") preferences.push(language);
  } catch {}
  return preferences.length === 0 ? undefined : resolveAcceptLanguage(locales, baseLocale, preferences.join(","));
}

"#;

const RANGE_BODY: &str = r#"  if (range === "*") return true;
  const normalizedRange = range.toLowerCase();
  const normalizedLocale = locale.toLowerCase();
  return normalizedRange === normalizedLocale || normalizedLocale.startsWith(`${normalizedRange}-`) || normalizedRange.startsWith(`${normalizedLocale}-`);
}
"#;

pub(super) fn generate_typescript_web_accept_language_module() -> String {
    web_accept_language_module(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/accept-language.ts",
        Some("web/accept-language.d.ts".to_owned()),
    ))
    .render_code()
}

pub(super) fn generate_web_accept_language_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/accept-language.d.ts",
        None,
    ));
    module.push_statement(EcmaStatement::type_declaration(
        concat!(
            "export declare function resolveAcceptLanguageLocale<Locale extends string>(\n",
            "  input: Record<string, unknown>,\n",
            "  locales: readonly Locale[],\n",
            "  baseLocale: Locale,\n",
            "  matchLocale: (value: unknown) => Locale | undefined,\n",
            "): Locale | undefined;\n",
        ),
        None,
    ));
    module.render_code()
}

/// Compile Accept-Language resolution as source-mapped TypeScript.
pub fn compile_typescript_web_accept_language_module() -> RenderedEcmaModule {
    compile_web_accept_language_module(EcmaScriptTarget::TypeScript)
}

/// Compile Accept-Language resolution as checked JavaScript.
pub fn compile_javascript_web_accept_language_module() -> RenderedEcmaModule {
    compile_web_accept_language_module(EcmaScriptTarget::JavaScript)
}

fn compile_web_accept_language_module(target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = match target {
        EcmaScriptTarget::TypeScript => "web/accept-language.ts",
        EcmaScriptTarget::JavaScript => "web/accept-language.js",
    };
    web_accept_language_module(EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn web_accept_language_module(output: EcmaModuleOutput) -> EcmaModule {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    module.push_statement(EcmaStatement::generated(
        [
            language_preference_type(target),
            resolve_locale_function(target),
            read_header_function(target),
            parse_header_function(target),
            resolve_header_function(target),
            resolve_navigator_function(target),
            range_matches_function(target),
        ]
        .concat(),
    ));
    module
}

fn language_preference_type(target: EcmaScriptTarget) -> String {
    match target {
        EcmaScriptTarget::TypeScript => concat!(
            "type LanguagePreference = {\n",
            "  range: string;\n",
            "  quality: number;\n",
            "  index: number;\n",
            "  specificity: number;\n",
            "};\n\n",
        ),
        EcmaScriptTarget::JavaScript => {
            "/** @typedef {{range: string, quality: number, index: number, specificity: number}} LanguagePreference */\n\n"
        }
    }
    .to_owned()
}

fn resolve_locale_function(target: EcmaScriptTarget) -> String {
    let signature = match target {
        EcmaScriptTarget::TypeScript => concat!(
            "export function resolveAcceptLanguageLocale<Locale extends string>(\n",
            "  input: Record<string, unknown>,\n",
            "  locales: readonly Locale[],\n",
            "  baseLocale: Locale,\n",
            "  matchLocale: (value: unknown) => Locale | undefined,\n",
            "): Locale | undefined {\n",
        ),
        EcmaScriptTarget::JavaScript => concat!(
            "/**\n",
            " * @template {string} Locale\n",
            " * @param {Record<string, unknown>} input\n",
            " * @param {readonly Locale[]} locales\n",
            " * @param {Locale} baseLocale\n",
            " * @param {(value: unknown) => Locale | undefined} matchLocale\n",
            " * @returns {Locale | undefined}\n",
            " */\n",
            "export function resolveAcceptLanguageLocale(input, locales, baseLocale, matchLocale) {\n",
        ),
    };
    format!("{signature}{RESOLVE_BODY}")
}

fn read_header_function(target: EcmaScriptTarget) -> String {
    let (signature, getter) = match target {
        EcmaScriptTarget::TypeScript => (
            "function readHeader(headers: unknown, name: string) {\n",
            "  const getter = (headers as { get?: (header: string) => string | null | undefined }).get;\n",
        ),
        EcmaScriptTarget::JavaScript => (
            concat!(
                "/**\n",
                " * @param {unknown} headers\n",
                " * @param {string} name\n",
                " * @returns {string | undefined}\n",
                " */\n",
                "function readHeader(headers, name) {\n",
            ),
            "  const getter = (/** @type {{get?: (header: string) => string | null | undefined}} */ (headers)).get;\n",
        ),
    };
    format!("{signature}  if (!headers) return undefined;\n{getter}{READ_HEADER_AFTER_GETTER}")
}

fn parse_header_function(target: EcmaScriptTarget) -> String {
    let signature = match target {
        EcmaScriptTarget::TypeScript => {
            "function parseAcceptLanguage(header: string | null | undefined): LanguagePreference[] {\n"
        }
        EcmaScriptTarget::JavaScript => concat!(
            "/**\n",
            " * @param {string | null | undefined} header\n",
            " * @returns {LanguagePreference[]}\n",
            " */\n",
            "function parseAcceptLanguage(header) {\n",
        ),
    };
    format!("{signature}{PARSE_BODY}")
}

fn resolve_header_function(target: EcmaScriptTarget) -> String {
    let (signature, best) = match target {
        EcmaScriptTarget::TypeScript => (
            concat!(
                "function resolveAcceptLanguage<Locale extends string>(\n",
                "  locales: readonly Locale[], baseLocale: Locale, header: string | null | undefined,\n",
                "): Locale | undefined {\n",
            ),
            "  let best: { locale: Locale; quality: number; preferenceIndex: number; base: boolean; localeIndex: number } | undefined;\n",
        ),
        EcmaScriptTarget::JavaScript => (
            concat!(
                "/**\n",
                " * @template {string} Locale\n",
                " * @param {readonly Locale[]} locales\n",
                " * @param {Locale} baseLocale\n",
                " * @param {string | null | undefined} header\n",
                " * @returns {Locale | undefined}\n",
                " */\n",
                "function resolveAcceptLanguage(locales, baseLocale, header) {\n",
            ),
            "  /** @type {{locale: Locale, quality: number, preferenceIndex: number, base: boolean, localeIndex: number} | undefined} */ let best;\n",
        ),
    };
    format!("{signature}{ACCEPT_BEFORE_BEST}{best}{ACCEPT_AFTER_BEST}")
}

fn resolve_navigator_function(target: EcmaScriptTarget) -> String {
    let (signature, preferences, languages, language) = match target {
        EcmaScriptTarget::TypeScript => (
            "function resolveNavigatorLanguage<Locale extends string>(locales: readonly Locale[], baseLocale: Locale, value: unknown) {\n",
            "  const preferences: string[] = [];\n",
            "(value as { languages?: unknown }).languages",
            "(value as { language?: unknown }).language",
        ),
        EcmaScriptTarget::JavaScript => (
            concat!(
                "/**\n",
                " * @template {string} Locale\n",
                " * @param {readonly Locale[]} locales\n",
                " * @param {Locale} baseLocale\n",
                " * @param {unknown} value\n",
                " * @returns {Locale | undefined}\n",
                " */\n",
                "function resolveNavigatorLanguage(locales, baseLocale, value) {\n",
            ),
            "  /** @type {string[]} */ const preferences = [];\n",
            "(/** @type {{languages?: unknown}} */ (value)).languages",
            "(/** @type {{language?: unknown}} */ (value)).language",
        ),
    };
    format!(
        "{signature}{NAVIGATOR_BEFORE_PREFERENCES}{preferences}{NAVIGATOR_BEFORE_LANGUAGES}{languages}{NAVIGATOR_AFTER_LANGUAGES}{language}{NAVIGATOR_AFTER_LANGUAGE}"
    )
}

fn range_matches_function(target: EcmaScriptTarget) -> String {
    let signature = match target {
        EcmaScriptTarget::TypeScript => {
            "function languageRangeMatches(range: string, locale: string) {\n"
        }
        EcmaScriptTarget::JavaScript => concat!(
            "/**\n",
            " * @param {string} range\n",
            " * @param {string} locale\n",
            " * @returns {boolean}\n",
            " */\n",
            "function languageRangeMatches(range, locale) {\n",
        ),
    };
    format!("{signature}{RANGE_BODY}")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        compile_javascript_web_accept_language_module,
        compile_typescript_web_accept_language_module,
        generate_typescript_web_accept_language_module, generate_web_accept_language_declaration,
    };

    #[test]
    fn accept_language_leaf_uses_one_target_aware_body() {
        let typescript = compile_typescript_web_accept_language_module();
        let javascript = compile_javascript_web_accept_language_module();
        assert_eq!(
            typescript
                .code
                .strip_suffix("//# sourceMappingURL=accept-language.ts.map\n")
                .expect("TypeScript map trailer"),
            generate_typescript_web_accept_language_module()
        );
        assert!(javascript.code.contains("@template {string} Locale"));
        assert!(!javascript.code.contains("let best:"));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=accept-language.js.map\n"));
        assert!(javascript
            .source_map
            .contains("\"file\":\"web/accept-language.js\""));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-accept-language");
        let snapshots = [
            (root.join("web/accept-language.js"), javascript.code),
            (
                root.join("web/accept-language.js.map"),
                javascript.source_map,
            ),
            (
                root.join("typescript/web/accept-language.ts"),
                typescript.code,
            ),
            (
                root.join("typescript/web/accept-language.d.ts"),
                generate_web_accept_language_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write Accept-Language snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read Accept-Language snapshot")
            );
        }
    }
}
