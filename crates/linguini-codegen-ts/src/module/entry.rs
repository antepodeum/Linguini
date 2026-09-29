//! Project `locale` and `index` entry modules for TypeScript and checked JavaScript.

use std::collections::BTreeMap;

use linguini_cldr::{
    built_in_text_direction, canonicalize_locale, locale_resolution_candidates, maximize_locale,
};

use crate::ecmascript::{EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaStatement};

use super::names::{escape_string, import_path_for_target, property_key, safe_identifier};
use super::{
    TypeScriptLocaleModule, TypeScriptProjectArtifact, TypeScriptProjectArtifactKind,
    ValidatedTypeScriptProject,
};

/// One source-mapped project entry module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTypeScriptProjectModule {
    pub artifact: TypeScriptProjectArtifact,
    pub code: String,
    pub source_map: String,
}

pub type CompiledJavaScriptProjectModule = CompiledTypeScriptProjectModule;

pub fn compile_typescript_project_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptProjectArtifact,
) -> CompiledTypeScriptProjectModule {
    compile_project_artifact_module(project, artifact, EcmaScriptTarget::TypeScript)
}

pub fn compile_javascript_project_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptProjectArtifact,
) -> CompiledJavaScriptProjectModule {
    compile_project_artifact_module(project, artifact, EcmaScriptTarget::JavaScript)
}

fn compile_project_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptProjectArtifact,
    target: EcmaScriptTarget,
) -> CompiledTypeScriptProjectModule {
    let output_path = target_path(&artifact.module_path, target);
    let module = project_module(
        &project.locales,
        project.options.base_locale.as_deref(),
        artifact.kind,
        EcmaModuleOutput::new(target, output_path, None),
    );
    let rendered = module.render(&[]);
    CompiledTypeScriptProjectModule {
        artifact: artifact.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
    }
}

pub(super) fn generate_project_locale(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    project_module(
        locales,
        base_locale,
        TypeScriptProjectArtifactKind::Locale,
        EcmaModuleOutput::new(
            EcmaScriptTarget::TypeScript,
            "locale.ts",
            Some("locale.d.ts".into()),
        ),
    )
    .render_code()
}

pub(super) fn generate_project_index(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    project_module(
        locales,
        base_locale,
        TypeScriptProjectArtifactKind::Index,
        EcmaModuleOutput::new(
            EcmaScriptTarget::TypeScript,
            "index.ts",
            Some("index.d.ts".into()),
        ),
    )
    .render_code()
}

pub(super) fn generate_project_locale_declaration(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    locale_declaration(locales, base_locale)
}

pub(super) fn generate_project_index_declaration() -> String {
    index_declaration()
}

fn project_module(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
    kind: TypeScriptProjectArtifactKind,
    output: EcmaModuleOutput,
) -> EcmaModule {
    let target = output.target();
    let code = match (kind, target) {
        (TypeScriptProjectArtifactKind::Locale, EcmaScriptTarget::TypeScript) => {
            typescript_locale(locales, base_locale)
        }
        (TypeScriptProjectArtifactKind::Locale, EcmaScriptTarget::JavaScript) => {
            javascript_locale(locales, base_locale)
        }
        (TypeScriptProjectArtifactKind::Index, EcmaScriptTarget::TypeScript) => {
            typescript_index(locales, base_locale)
        }
        (TypeScriptProjectArtifactKind::Index, EcmaScriptTarget::JavaScript) => {
            javascript_index(locales, base_locale)
        }
    };
    let mut module = EcmaModule::new(output);
    module.push_statement(EcmaStatement::generated(code));
    module
}

fn typescript_locale(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    format!(
        concat!(
            "export const locales = [{}] as const;\n",
            "export const baseLocale = {};\n\n",
            "export const localeDirections = {{\n{}}} as const;\n\n",
            "export type Locale = (typeof locales)[number];\n",
            "export type TextDirection = \"ltr\" | \"rtl\";\n\n",
            "const localeResolution: Readonly<Record<string, Locale>> = {{\n{}}};\n\n",
            "export function isLocale(locale: unknown): locale is Locale {{\n",
            "  return normalizeLocale(locale) !== undefined;\n",
            "}}\n\n",
            "export function normalizeLocale(locale: unknown): Locale | undefined {{\n",
            "  if (typeof locale !== \"string\") return undefined;\n",
            "  return localeResolution[locale.toLowerCase()];\n",
            "}}\n\n",
            "export function getTextDirection(locale: Locale): TextDirection {{\n",
            "  return localeDirections[normalizeLocale(locale) ?? baseLocale];\n",
            "}}\n",
        ),
        locale_literals(locales).join(", "),
        base_locale_literal(locales, base_locale),
        project_locale_directions(locales),
        project_locale_resolution(locales),
    )
}

fn javascript_locale(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    format!(
        concat!(
            "export const locales = /** @type {{const}} */ ([{}]);\n",
            "export const baseLocale = {};\n\n",
            "export const localeDirections = /** @type {{const}} */ ({{\n{}",
            "}});\n\n",
            "/** @typedef {{(typeof locales)[number]}} Locale */\n",
            "/** @typedef {{\"ltr\" | \"rtl\"}} TextDirection */\n\n",
            "/** @type {{Readonly<Record<string, Locale>>}} */\n",
            "const localeResolution = {{\n{}",
            "}};\n\n",
            "/**\n",
            " * @param {{unknown}} locale\n",
            " * @returns {{locale is Locale}}\n",
            " */\n",
            "export function isLocale(locale) {{\n",
            "  return normalizeLocale(locale) !== undefined;\n",
            "}}\n\n",
            "/**\n",
            " * @param {{unknown}} locale\n",
            " * @returns {{Locale | undefined}}\n",
            " */\n",
            "export function normalizeLocale(locale) {{\n",
            "  if (typeof locale !== \"string\") return undefined;\n",
            "  return localeResolution[locale.toLowerCase()];\n",
            "}}\n\n",
            "/**\n",
            " * @param {{Locale}} locale\n",
            " * @returns {{TextDirection}}\n",
            " */\n",
            "export function getTextDirection(locale) {{\n",
            "  return localeDirections[normalizeLocale(locale) ?? baseLocale];\n",
            "}}\n",
        ),
        locale_literals(locales).join(", "),
        base_locale_literal(locales, base_locale),
        project_locale_directions(locales),
        project_locale_resolution(locales),
    )
}

fn typescript_index(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    let base_locale = base_locale.expect("validated TypeScript projects have a base locale");
    format!(
        concat!(
            "{}",
            "import {{ baseLocale, normalizeLocale, type Locale }} from \"./locale\";\n",
            "export {{\n",
            "  locales,\n",
            "  baseLocale,\n",
            "  localeDirections,\n",
            "  isLocale,\n",
            "  normalizeLocale,\n",
            "  getTextDirection,\n",
            "}} from \"./locale\";\n",
            "export type {{ Locale, TextDirection }} from \"./locale\";\n",
            "export type * from \"./shared\";\n",
            "import type {{ LinguiniMessages }} from \"./messages\";\n",
            "export type {{ LinguiniMessages }} from \"./messages\";\n\n",
            "export const localeModules: Partial<Record<Locale, LinguiniMessages>> = {{\n{}",
            "}} as const;\n\n",
            "export const localeLoaders = {{\n{}",
            "}} as const;\n\n",
            "type LinguiniLanguage = Locale;\n",
            "export type Linguini = LinguiniMessages;\n\n",
            "type LinguiniLanguageInput = LinguiniLanguage;\n\n",
            "{}",
        ),
        project_locale_import(base_locale, EcmaScriptTarget::TypeScript),
        project_locale_modules(base_locale),
        project_locale_loaders(locales, base_locale, EcmaScriptTarget::TypeScript),
        typescript_index_runtime(),
    )
}

fn javascript_index(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    let base_locale = base_locale.expect("validated TypeScript projects have a base locale");
    format!(
        concat!(
            "{}",
            "import {{ baseLocale, normalizeLocale }} from \"./locale.js\";\n",
            "export {{\n",
            "  locales,\n",
            "  baseLocale,\n",
            "  localeDirections,\n",
            "  isLocale,\n",
            "  normalizeLocale,\n",
            "  getTextDirection,\n",
            "}} from \"./locale.js\";\n\n",
            "/** @typedef {{import(\"./locale.js\").Locale}} Locale */\n",
            "/** @typedef {{typeof {}}} LinguiniMessages */\n",
            "/** @typedef {{LinguiniMessages}} Linguini */\n",
            "/**\n",
            " * @typedef {{object}} LinguiniProviderOptions\n",
            " * @property {{(() => Locale)=}} getLocale\n",
            " * @property {{(() => Locale)=}} resolveLanguage\n",
            " */\n\n",
            "/** @type {{Partial<Record<Locale, LinguiniMessages>>}} */\n",
            "export const localeModules = {{\n{}",
            "}};\n\n",
            "/** @type {{Record<Locale, () => Promise<LinguiniMessages>>}} */\n",
            "export const localeLoaders = {{\n{}",
            "}};\n\n",
            "{}",
        ),
        project_locale_import(base_locale, EcmaScriptTarget::JavaScript),
        locale_identifier(base_locale),
        project_locale_modules(base_locale),
        project_locale_loaders(locales, base_locale, EcmaScriptTarget::JavaScript),
        javascript_index_runtime(),
    )
}

fn typescript_index_runtime() -> &'static str {
    concat!(
        "export type LinguiniProviderOptions = {\n",
        "  getLocale?: () => LinguiniLanguageInput;\n",
        "  resolveLanguage?: () => LinguiniLanguageInput;\n",
        "};\n\n",
        "const pendingLocales = new Map<Locale, Promise<Linguini>>();\n\n",
        "export async function prepareLinguini(language: LinguiniLanguageInput): Promise<Linguini> {\n",
        "  const locale = normalizeLocale(language) ?? baseLocale;\n",
        "  const available = localeModules[locale];\n",
        "  if (available) return available;\n",
        "  const pending = pendingLocales.get(locale);\n",
        "  if (pending) return pending;\n",
        "  const task = localeLoaders[locale]().then((loaded) => {\n",
        "    localeModules[locale] = loaded;\n",
        "    return loaded;\n",
        "  }).finally(() => {\n",
        "    pendingLocales.delete(locale);\n",
        "  });\n",
        "  pendingLocales.set(locale, task);\n",
        "  return task;\n",
        "}\n\n",
        "export function createLinguini(language: LinguiniLanguageInput): Linguini {\n",
        "  const locale = normalizeLocale(language) ?? baseLocale;\n",
        "  const messages = localeModules[locale];\n",
        "  if (messages) return messages;\n",
        "  throw new Error(`Linguini: locale ${JSON.stringify(locale)} is not prepared; call prepareLinguini(locale) first`);\n",
        "}\n\n",
        "export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {\n",
        "  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);\n",
        "  return new Proxy({} as Linguini, {\n",
        "    get(_target, property) {\n",
        "      return createLinguini(resolve())[property as keyof Linguini];\n",
        "    },\n",
        "  });\n",
        "}\n\n",
        "export function configureLinguini(options: {\n",
        "  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n",
        "}): Linguini {\n",
        "  if (typeof options.language === \"function\") {\n",
        "    return createLinguiniProvider({ resolveLanguage: options.language });\n",
        "  }\n",
        "  return createLinguini(options.language);\n",
        "}\n\n",
        "export const lgl: Linguini = createLinguini(baseLocale);\n",
    )
}

fn javascript_index_runtime() -> &'static str {
    concat!(
        "/** @type {Map<Locale, Promise<Linguini>>} */\n",
        "const pendingLocales = new Map();\n\n",
        "/**\n",
        " * @param {Locale} language\n",
        " * @returns {Promise<Linguini>}\n",
        " */\n",
        "export async function prepareLinguini(language) {\n",
        "  const locale = normalizeLocale(language) ?? baseLocale;\n",
        "  const available = localeModules[locale];\n",
        "  if (available) return available;\n",
        "  const pending = pendingLocales.get(locale);\n",
        "  if (pending) return pending;\n",
        "  const task = localeLoaders[locale]().then((loaded) => {\n",
        "    localeModules[locale] = loaded;\n",
        "    return loaded;\n",
        "  }).finally(() => {\n",
        "    pendingLocales.delete(locale);\n",
        "  });\n",
        "  pendingLocales.set(locale, task);\n",
        "  return task;\n",
        "}\n\n",
        "/**\n",
        " * @param {Locale} language\n",
        " * @returns {Linguini}\n",
        " */\n",
        "export function createLinguini(language) {\n",
        "  const locale = normalizeLocale(language) ?? baseLocale;\n",
        "  const messages = localeModules[locale];\n",
        "  if (messages) return messages;\n",
        "  throw new Error(`Linguini: locale ${JSON.stringify(locale)} is not prepared; call prepareLinguini(locale) first`);\n",
        "}\n\n",
        "/**\n",
        " * @param {LinguiniProviderOptions} [options]\n",
        " * @returns {Linguini}\n",
        " */\n",
        "export function createLinguiniProvider(options = {}) {\n",
        "  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);\n",
        "  return new Proxy(/** @type {Linguini} */ ({}), {\n",
        "    get(_target, property) {\n",
        "      return createLinguini(resolve())[/** @type {keyof Linguini} */ (property)];\n",
        "    },\n",
        "  });\n",
        "}\n\n",
        "/**\n",
        " * @param {{language: Locale | (() => Locale)}} options\n",
        " * @returns {Linguini}\n",
        " */\n",
        "export function configureLinguini(options) {\n",
        "  if (typeof options.language === \"function\") {\n",
        "    return createLinguiniProvider({ resolveLanguage: options.language });\n",
        "  }\n",
        "  return createLinguini(options.language);\n",
        "}\n\n",
        "/** @type {Linguini} */\n",
        "export const lgl = createLinguini(baseLocale);\n",
    )
}

fn locale_declaration(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    format!(
        concat!(
            "export declare const locales: readonly [{}];\n",
            "export declare const baseLocale: {};\n\n",
            "export declare const localeDirections: {{\n{}",
            "}};\n\n",
            "export type Locale = (typeof locales)[number];\n",
            "export type TextDirection = \"ltr\" | \"rtl\";\n\n",
            "export declare function isLocale(locale: unknown): locale is Locale;\n",
            "export declare function normalizeLocale(locale: unknown): Locale | undefined;\n",
            "export declare function getTextDirection(locale: Locale): TextDirection;\n",
        ),
        locale_literals(locales).join(", "),
        base_locale_literal(locales, base_locale),
        project_locale_direction_declarations(locales),
    )
}

fn index_declaration() -> String {
    concat!(
        "import type { Locale } from \"./locale\";\n",
        "export {\n",
        "  locales,\n",
        "  baseLocale,\n",
        "  localeDirections,\n",
        "  isLocale,\n",
        "  normalizeLocale,\n",
        "  getTextDirection,\n",
        "} from \"./locale\";\n",
        "export type { Locale, TextDirection } from \"./locale\";\n",
        "export type * from \"./shared\";\n",
        "import type { LinguiniMessages } from \"./messages\";\n",
        "export type { LinguiniMessages } from \"./messages\";\n\n",
        "export declare const localeModules: Partial<Record<Locale, LinguiniMessages>>;\n\n",
        "export declare const localeLoaders: Record<Locale, () => Promise<LinguiniMessages>>;\n\n",
        "type LinguiniLanguage = Locale;\n",
        "export type Linguini = LinguiniMessages;\n\n",
        "type LinguiniLanguageInput = LinguiniLanguage;\n\n",
        "export type LinguiniProviderOptions = {\n",
        "  getLocale?: () => LinguiniLanguageInput;\n",
        "  resolveLanguage?: () => LinguiniLanguageInput;\n",
        "};\n\n",
        "export declare function prepareLinguini(language: LinguiniLanguageInput): Promise<Linguini>;\n\n",
        "export declare function createLinguini(language: LinguiniLanguageInput): Linguini;\n\n",
        "export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;\n\n",
        "export declare function configureLinguini(options: {\n",
        "  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n",
        "}): Linguini;\n\n",
        "export declare const lgl: Linguini;\n",
    )
    .to_owned()
}

fn target_path(path: &str, target: EcmaScriptTarget) -> String {
    match target {
        EcmaScriptTarget::TypeScript => path.to_owned(),
        EcmaScriptTarget::JavaScript => path
            .strip_suffix(".ts")
            .map_or_else(|| path.to_owned(), |stem| format!("{stem}.js")),
    }
}

fn project_locale_resolution(locales: &[TypeScriptLocaleModule]) -> String {
    let mut candidates = locale_resolution_candidates()
        .iter()
        .map(|locale| (*locale).to_owned())
        .collect::<Vec<_>>();
    for locale in locales {
        let Ok(canonical) = canonicalize_locale(&locale.locale) else {
            continue;
        };
        let Some(language) = canonical.split('-').next() else {
            continue;
        };
        let Ok(maximized) = maximize_locale(language) else {
            continue;
        };
        if let Some(script) = maximized
            .split('-')
            .nth(1)
            .filter(|subtag| subtag.len() == 4)
        {
            candidates.push(format!("{language}-{script}"));
        }
    }
    candidates.sort();
    candidates.dedup();

    let mut resolution = locales
        .iter()
        .map(|locale| (locale.locale.to_ascii_lowercase(), locale.locale.clone()))
        .collect::<BTreeMap<_, _>>();
    for candidate in candidates {
        if let Some(resolved) = super::locale_fallback_chain(locales, &candidate, None)
            .into_iter()
            .next()
        {
            resolution.insert(candidate.to_ascii_lowercase(), resolved);
        }
    }
    resolution
        .into_iter()
        .map(|(candidate, resolved)| {
            format!(
                "  \"{}\": \"{}\",\n",
                escape_string(&candidate),
                escape_string(&resolved)
            )
        })
        .collect()
}

fn project_locale_import(locale: &str, target: EcmaScriptTarget) -> String {
    format!(
        "import {} from \"{}\";\n",
        locale_identifier(locale),
        import_path_for_target(&format!("./locales/{}", escape_string(locale)), target)
    )
}

fn project_locale_directions(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  {}: \"{}\",\n",
                property_key(&locale.locale),
                locale_direction(&locale.locale)
            )
        })
        .collect()
}

fn project_locale_direction_declarations(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  readonly {}: \"{}\";\n",
                property_key(&locale.locale),
                locale_direction(&locale.locale)
            )
        })
        .collect()
}

fn project_locale_modules(locale: &str) -> String {
    format!(
        "  {}: {},\n",
        property_key(locale),
        locale_identifier(locale)
    )
}

fn project_locale_loaders(
    locales: &[TypeScriptLocaleModule],
    base_locale: &str,
    target: EcmaScriptTarget,
) -> String {
    locales
        .iter()
        .map(|locale| {
            if locale.locale == base_locale {
                format!(
                    "  {}: () => Promise.resolve({}),\n",
                    property_key(&locale.locale),
                    locale_identifier(&locale.locale)
                )
            } else {
                let path = import_path_for_target(
                    &format!("./locales/{}", escape_string(&locale.locale)),
                    target,
                );
                format!(
                    "  {}: () => import(\"{path}\").then((module) => module.default),\n",
                    property_key(&locale.locale)
                )
            }
        })
        .collect()
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
    let locale = base_locale.expect("validated TypeScript projects have an explicit base locale");
    debug_assert!(locales.iter().any(|entry| entry.locale == locale));
    format!("\"{}\"", escape_string(locale))
}

fn locale_direction(locale: &str) -> &'static str {
    built_in_text_direction(locale).unwrap_or("ltr")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    use super::{
        compile_javascript_project_artifact_module, compile_typescript_project_artifact_module,
    };
    use crate::ecmascript::EcmaSource;
    use crate::module::{
        compile_javascript_locale_artifact_module, generate_javascript_schema_files,
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectArtifactKind,
        TypeScriptProjectOptions, ValidatedTypeScriptProject,
    };

    #[test]
    fn project_artifacts_share_target_aware_entry_emission() {
        let schema_text = "hello(name: String)\n";
        let english_text = "hello = Hello {name}\n";
        let french_text = "hello = Salut {name}\n";
        let schema =
            lower_schema(&parse_schema_in(schema_text, SourceId(201)).expect("schema parses"));
        let locales = [
            TypeScriptLocaleModule {
                locale: "en".into(),
                module: lower_locale(
                    &parse_locale_in(english_text, SourceId(202)).expect("English"),
                ),
            },
            TypeScriptLocaleModule {
                locale: "fr".into(),
                module: lower_locale(&parse_locale_in(french_text, SourceId(203)).expect("French")),
            },
        ];
        let options = TypeScriptProjectOptions {
            base_locale: Some("en".into()),
            ..TypeScriptProjectOptions::default()
        };
        let project =
            ValidatedTypeScriptProject::try_new(&schema, &locales, &options).expect("project");
        let artifacts = project.project_artifacts();
        assert_eq!(
            artifacts
                .iter()
                .map(|artifact| artifact.module_path.as_str())
                .collect::<Vec<_>>(),
            ["locale.ts", "index.ts"]
        );
        assert_eq!(artifacts, project.project_artifacts());
        let project_files = generate_typescript_project_files(&project).expect("project files");

        for artifact in &artifacts {
            let typescript = compile_typescript_project_artifact_module(&project, artifact);
            let javascript = compile_javascript_project_artifact_module(&project, artifact);
            let project_file = project_files
                .iter()
                .find(|file| file.path == artifact.module_path)
                .expect("project module");
            let map_name = format!("{}.map", artifact.module_path);
            assert_eq!(
                typescript
                    .code
                    .strip_suffix(&format!("//# sourceMappingURL={map_name}\n"))
                    .expect("TypeScript source-map trailer"),
                project_file.contents
            );
            assert!(javascript.code.ends_with(&format!(
                "//# sourceMappingURL={}\n",
                map_name.replace(".ts.map", ".js.map")
            )));
            assert!(javascript.source_map.contains(&format!(
                "\"file\":\"{}\"",
                artifact.module_path.replace(".ts", ".js")
            )));
            assert!(!javascript.code.contains(" as const"));
            assert!(!javascript.code.contains("import type"));
            assert!(!javascript.code.contains("export type"));
        }

        let locale = compile_javascript_project_artifact_module(&project, &artifacts[0]);
        assert_eq!(artifacts[0].kind, TypeScriptProjectArtifactKind::Locale);
        assert!(locale.code.contains("@returns {locale is Locale}"));
        assert!(locale.code.contains("\"fr\": \"fr\""));

        let index = compile_javascript_project_artifact_module(&project, &artifacts[1]);
        assert_eq!(artifacts[1].kind, TypeScriptProjectArtifactKind::Index);
        assert!(index.code.contains("import(\"./locales/fr.js\")"));
        assert!(index.code.contains("const pendingLocales = new Map();"));
        assert!(index
            .code
            .contains("export async function prepareLinguini(language)"));

        let snapshot_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-project");
        let typescript_root = snapshot_root.join("typescript");
        let sources = [
            EcmaSource::new(SourceId(201), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(202), "en.lgl", english_text),
            EcmaSource::new(SourceId(203), "fr.lgl", french_text),
        ];
        let mut snapshots = generate_javascript_schema_files(&schema)
            .into_iter()
            .filter(|file| file.path.ends_with(".js"))
            .map(|file| (snapshot_root.join(file.path), file.contents))
            .collect::<Vec<_>>();
        for artifact in project.locale_artifacts().expect("locale artifacts") {
            let compiled = compile_javascript_locale_artifact_module(&project, &artifact, &sources)
                .expect("JavaScript locale");
            snapshots.push((
                snapshot_root.join(artifact.module_path.replace(".ts", ".js")),
                compiled.code,
            ));
            snapshots.push((
                snapshot_root.join(artifact.source_map_path.replace(".ts.map", ".js.map")),
                compiled.source_map,
            ));
        }
        for artifact in &artifacts {
            let compiled = compile_javascript_project_artifact_module(&project, artifact);
            snapshots.push((
                snapshot_root.join(artifact.module_path.replace(".ts", ".js")),
                compiled.code,
            ));
            snapshots.push((
                snapshot_root.join(artifact.source_map_path.replace(".ts.map", ".js.map")),
                compiled.source_map,
            ));
        }
        snapshots.extend(
            project_files
                .iter()
                .filter(|file| {
                    matches!(
                        file.path.as_str(),
                        "shared.ts"
                            | "messages.ts"
                            | "locales/en.ts"
                            | "locales/fr.ts"
                            | "locale.ts"
                            | "index.ts"
                    )
                })
                .map(|file| (typescript_root.join(&file.path), file.contents.clone())),
        );
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write project snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read project snapshot")
            );
        }
    }
}
