use crate::error::{ConfigError, ConfigResult};
use crate::model::{
    canonicalize_locale_tag, AnalysisConfig, LinguiniConfig, PathsConfig, ProjectConfig,
    TargetsConfig, TypeScriptBundlerConfig, TypeScriptBundlerDynamicConfig,
    TypeScriptBundlerDynamicMode, TypeScriptBundlerLocaleLoading, TypeScriptTargetConfig,
    UnusedMessagesConfig,
};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    project: Option<RawProjectConfig>,
    paths: Option<RawPathsConfig>,
    targets: Option<RawTargetsConfig>,
    analysis: Option<RawAnalysisConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProjectConfig {
    name: Option<String>,
    default_locale: Option<String>,
    locales: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPathsConfig {
    schema: Option<String>,
    locale: Option<String>,
    cache: Option<toml::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTargetsConfig {
    ts: Option<RawTypeScriptTargetConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAnalysisConfig {
    unused_messages: Option<RawUnusedMessagesConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawUnusedMessagesConfig {
    sources: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    ignore: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTypeScriptTargetConfig {
    out: Option<String>,
    declaration: Option<bool>,
    gitignore: Option<bool>,
    tree_shaking: Option<bool>,
    messages: Option<Vec<String>>,
    bundler: Option<RawTypeScriptBundlerConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTypeScriptBundlerConfig {
    sources: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    locale_loading: Option<String>,
    dynamic: Option<RawTypeScriptBundlerDynamicConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTypeScriptBundlerDynamicConfig {
    mode: Option<String>,
    allow: Option<Vec<String>>,
}

pub fn parse_config(source: &str) -> ConfigResult<LinguiniConfig> {
    let raw: RawConfig = toml::from_str(source).map_err(|error| {
        if let Some((field, replacement)) = removed_module_field(source) {
            return ConfigError::RemovedField { field, replacement };
        }
        let span = error.span().map(|span| (span.start, span.end));
        ConfigError::Toml {
            message: error.to_string(),
            span,
        }
    })?;

    let project = build_project(required(raw.project, "project")?)?;
    let paths = build_paths(required(raw.paths, "paths")?)?;
    let targets = build_targets(raw.targets.unwrap_or_default())?;
    let analysis = build_analysis(raw.analysis.unwrap_or_default());
    let config = LinguiniConfig {
        project,
        paths,
        targets,
        analysis,
    };
    config.validate()?;
    Ok(config)
}

fn removed_module_field(source: &str) -> Option<(&'static str, &'static str)> {
    let value: toml::Value = toml::from_str(source).ok()?;
    if value.get("web").is_some() {
        return Some((
            "web",
            "configure web runtime adapters in application modules and hooks",
        ));
    }
    let target = value.get("targets")?.get("ts")?;
    if target.get("framework").is_some() {
        return Some((
            "targets.ts.framework",
            "select Svelte project bindings through the build integration",
        ));
    }
    target.get("module").map(|_| {
        (
            "targets.ts.module",
            "generated ECMAScript modules are always ESM",
        )
    })
}

fn build_project(raw: RawProjectConfig) -> ConfigResult<ProjectConfig> {
    let name = required(raw.name, "project.name")?.trim().to_owned();
    let default_locale =
        canonicalize_locale_tag(&required(raw.default_locale, "project.default_locale")?)?;
    let locales = required(raw.locales, "project.locales")?
        .into_iter()
        .map(|locale| canonicalize_locale_tag(&locale))
        .collect::<ConfigResult<Vec<_>>>()?;
    Ok(ProjectConfig {
        name,
        default_locale,
        locales,
    })
}

fn build_paths(raw: RawPathsConfig) -> ConfigResult<PathsConfig> {
    if raw.cache.is_some() {
        return Err(ConfigError::RemovedField {
            field: "paths.cache",
            replacement: "remove it; Linguini no longer uses a source-tree cache",
        });
    }
    Ok(PathsConfig {
        schema: normalize_project_path(required(raw.schema, "paths.schema")?),
        locale: normalize_project_path(required(raw.locale, "paths.locale")?),
    })
}

fn build_targets(raw: RawTargetsConfig) -> ConfigResult<TargetsConfig> {
    let ts = raw.ts.map(build_typescript_target).transpose()?;
    Ok(TargetsConfig { ts })
}

fn build_analysis(raw: RawAnalysisConfig) -> AnalysisConfig {
    AnalysisConfig {
        unused_messages: raw.unused_messages.map(|unused| UnusedMessagesConfig {
            sources: unused
                .sources
                .unwrap_or_default()
                .into_iter()
                .map(normalize_project_path)
                .collect(),
            exclude: unused
                .exclude
                .unwrap_or_default()
                .into_iter()
                .map(normalize_project_path)
                .collect(),
            ignore: unused
                .ignore
                .unwrap_or_default()
                .into_iter()
                .map(|prefix| prefix.trim().to_owned())
                .collect(),
        }),
    }
}

fn build_typescript_target(raw: RawTypeScriptTargetConfig) -> ConfigResult<TypeScriptTargetConfig> {
    Ok(TypeScriptTargetConfig {
        out: normalize_project_path(
            raw.out
                .unwrap_or_else(|| "src/generated/linguini".to_owned()),
        ),
        declaration: raw.declaration.unwrap_or(true),
        gitignore: raw.gitignore.unwrap_or(true),
        tree_shaking: raw.tree_shaking.unwrap_or(false),
        messages: raw.messages.unwrap_or_default(),
        bundler: raw.bundler.map(build_typescript_bundler).transpose()?,
    })
}

fn build_typescript_bundler(
    raw: RawTypeScriptBundlerConfig,
) -> ConfigResult<TypeScriptBundlerConfig> {
    Ok(TypeScriptBundlerConfig {
        sources: raw
            .sources
            .unwrap_or_default()
            .into_iter()
            .map(normalize_project_path)
            .collect(),
        exclude: raw
            .exclude
            .unwrap_or_default()
            .into_iter()
            .map(normalize_project_path)
            .collect(),
        locale_loading: match raw.locale_loading.as_deref().unwrap_or("eager") {
            "eager" => TypeScriptBundlerLocaleLoading::Eager,
            "dynamic" => TypeScriptBundlerLocaleLoading::Dynamic,
            value => {
                return Err(ConfigError::InvalidString(format!(
                    "targets.ts.bundler.locale_loading = {value}"
                )))
            }
        },
        dynamic: build_typescript_bundler_dynamic(raw.dynamic)?,
    })
}

fn build_typescript_bundler_dynamic(
    raw: Option<RawTypeScriptBundlerDynamicConfig>,
) -> ConfigResult<TypeScriptBundlerDynamicConfig> {
    let Some(raw) = raw else {
        return Ok(TypeScriptBundlerDynamicConfig::default());
    };
    let mode = match raw.mode.as_deref().unwrap_or("error") {
        "error" => TypeScriptBundlerDynamicMode::Error,
        "bundle" => TypeScriptBundlerDynamicMode::Bundle,
        value => {
            return Err(ConfigError::InvalidString(format!(
                "targets.ts.bundler.dynamic.mode = {value}"
            )))
        }
    };
    Ok(TypeScriptBundlerDynamicConfig {
        mode,
        allow: raw.allow.unwrap_or_default(),
    })
}

fn normalize_project_path(value: String) -> String {
    let value = value.trim();
    let is_absolute = value.starts_with('/');
    let normalized = value
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect::<Vec<_>>()
        .join("/");
    if is_absolute {
        format!("/{normalized}")
    } else {
        normalized
    }
}

fn required<T>(value: Option<T>, field: &'static str) -> ConfigResult<T> {
    value.ok_or(ConfigError::MissingField(field))
}

#[cfg(test)]
mod tests {
    use super::parse_config;
    use crate::{TypeScriptBundlerDynamicMode, TypeScriptBundlerLocaleLoading};

    #[test]
    fn parses_required_project_config_and_canonicalizes_locales() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "ru"
            locales = [
              "ru",
              "en-us", # TOML comments and multiline arrays are supported.
              "es-419",
            ]

            [paths]
            schema = "linguini/schema"
            locale = "linguini/locale"
            "#,
        )
        .expect("valid config");

        assert_eq!(config.project.name, "shop");
        assert_eq!(config.project.locales, ["ru", "en-US", "es-419"]);
        assert_eq!(config.paths.schema, "linguini/schema");
        assert_eq!(config.paths.locale, "linguini/locale");
        assert!(config.targets.ts.is_none());
    }

    #[test]
    fn rejects_removed_cache_and_module_fields_with_migrations() {
        let cache = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            cache = ".linguini/cache"
            "#,
        )
        .expect_err("removed cache");
        assert!(cache.to_string().contains("`paths.cache` was removed"));

        let module = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [targets.ts]
            module = "esm"
            "#,
        )
        .expect_err("removed module");
        assert!(module
            .to_string()
            .contains("`targets.ts.module` was removed"));

        let unrelated = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            module = "esm"
            [paths]
            schema = "schema"
            locale = "locale"
            "#,
        )
        .expect_err("unknown project field");
        assert!(unrelated.to_string().contains("unknown field `module`"));
        assert!(!unrelated.to_string().contains("targets.ts.module"));
    }

    #[test]
    fn rejects_absolute_paths_before_normalization() {
        for (field, section) in [
            (
                "paths.schema",
                r#"
                [paths]
                schema = "/linguini/schema"
                locale = "linguini/locale"
                "#,
            ),
            (
                "paths.locale",
                r#"
                [paths]
                schema = "linguini/schema"
                locale = "/linguini/locale"
                "#,
            ),
            (
                "targets.ts.out",
                r#"
                [paths]
                schema = "linguini/schema"
                locale = "linguini/locale"
                [targets.ts]
                out = "/src/generated/linguini"
                "#,
            ),
        ] {
            let source = format!(
                r#"
                [project]
                name = "shop"
                default_locale = "en"
                locales = ["en"]
                {section}
                "#
            );
            let error = parse_config(&source).expect_err(field);
            let message = error.to_string();
            assert!(message.contains(field), "{message}");
            assert!(message.contains("project-relative"), "{message}");
        }
    }

    #[test]
    fn normalizes_safe_project_relative_paths() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = " ./linguini//schema/ "
            locale = "./linguini/locale/"
            [targets.ts]
            out = "./src//generated/linguini/"
            "#,
        )
        .expect("normalized paths");

        assert_eq!(config.paths.schema, "linguini/schema");
        assert_eq!(config.paths.locale, "linguini/locale");
        assert_eq!(
            config.targets.ts.expect("target").out,
            "src/generated/linguini"
        );
    }

    #[test]
    fn parses_typescript_codegen_target() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "ru"
            locales = ["ru"]

            [paths]
            schema = "linguini/schema"
            locale = "linguini/locale"

            [targets.ts]
            out = "src/generated/linguini"
            declaration = false
            gitignore = false
            tree_shaking = true
            messages = ["delivery", "email_input.label"]

            "#,
        )
        .expect("valid config");

        let target = config.targets.ts.expect("ts target");
        assert_eq!(target.out, "src/generated/linguini");
        assert!(!target.declaration);
        assert!(!target.gitignore);
        assert!(target.tree_shaking);
        assert_eq!(target.messages, ["delivery", "email_input.label"]);
        assert!(target.bundler.is_none());
    }

    #[test]
    fn parses_and_validates_typescript_bundler_scope() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [targets.ts]

            [targets.ts.bundler]
            sources = [" ./src/ ", "tests//ui"]
            exclude = ["src/generated"]
            "#,
        )
        .expect("valid bundler config");
        let bundler = config.targets.ts.expect("target").bundler.expect("bundler");
        assert_eq!(bundler.sources, ["src", "tests/ui"]);
        assert_eq!(bundler.exclude, ["src/generated"]);
        assert_eq!(
            bundler.locale_loading,
            TypeScriptBundlerLocaleLoading::Eager
        );
        assert_eq!(bundler.dynamic.mode, TypeScriptBundlerDynamicMode::Error);
        assert!(bundler.dynamic.allow.is_empty());

        for section in [
            "sources = []",
            "sources = [\"../src\"]",
            "sources = [\"src\\\\app\"]",
            "sources = [\"src\", \"./src\"]",
            "sources = [\"src/app\"]\nexclude = [\"src\"]",
            "sources = [\"src\"]\nunknown = true",
        ] {
            let source = format!(
                r#"
                [project]
                name = "shop"
                default_locale = "en"
                locales = ["en"]
                [paths]
                schema = "schema"
                locale = "locale"
                [targets.ts]

                [targets.ts.bundler]
                {section}
                "#
            );
            let error = parse_config(&source).expect_err(section);
            let message = error.to_string();
            assert!(
                message.contains("targets.ts.bundler") || message.contains("unknown field"),
                "{message}"
            );
        }

        let without_framework = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [targets.ts.bundler]
            sources = ["src"]
            "#,
        )
        .expect("bundler planning is framework-independent");
        assert!(without_framework.targets.ts.unwrap().bundler.is_some());
    }

    #[test]
    fn parses_strict_and_finite_dynamic_bundler_policies() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [targets.ts]

            [targets.ts.bundler]
            sources = ["src"]
            [targets.ts.bundler.dynamic]
            mode = "bundle"
            allow = ["main.hero.title", "admin.notice"]
            "#,
        )
        .expect("finite dynamic policy");
        let dynamic = config
            .targets
            .ts
            .expect("target")
            .bundler
            .expect("bundler")
            .dynamic;
        assert_eq!(dynamic.mode, TypeScriptBundlerDynamicMode::Bundle);
        assert_eq!(dynamic.allow, ["main.hero.title", "admin.notice"]);

        for (section, expected) in [
            (
                "mode = \"bundle\"\nallow = []",
                "targets.ts.bundler.dynamic.allow",
            ),
            (
                "mode = \"error\"\nallow = [\"main.title\"]",
                "requires mode = \"bundle\"",
            ),
            ("mode = \"unknown\"", "targets.ts.bundler.dynamic.mode"),
            (
                "mode = \"bundle\"\nallow = [\"main..title\"]",
                "targets.ts.bundler.dynamic.allow",
            ),
            (
                "mode = \"bundle\"\nallow = [\"main/title\"]",
                "targets.ts.bundler.dynamic.allow",
            ),
            (
                "mode = \"bundle\"\nallow = [\"main.*\"]",
                "targets.ts.bundler.dynamic.allow",
            ),
            (
                "mode = \"bundle\"\nallow = [\"main.title\", \"MAIN.TITLE\"]",
                "duplicate config key",
            ),
            (
                "mode = \"bundle\"\nallow = [\"main.title\"]\nunknown = true",
                "unknown field",
            ),
        ] {
            let source = format!(
                r#"
                [project]
                name = "shop"
                default_locale = "en"
                locales = ["en"]
                [paths]
                schema = "schema"
                locale = "locale"
                [targets.ts]

                [targets.ts.bundler]
                sources = ["src"]
                [targets.ts.bundler.dynamic]
                {section}
                "#
            );
            let error = parse_config(&source).expect_err(section);
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn parses_locale_loading_policy_and_rejects_unknown_values() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [targets.ts]

            [targets.ts.bundler]
            sources = ["src"]
            locale_loading = "dynamic"
            "#,
        )
        .expect("dynamic locale loading policy");
        let bundler = config.targets.ts.expect("target").bundler.expect("bundler");
        assert_eq!(
            bundler.locale_loading,
            TypeScriptBundlerLocaleLoading::Dynamic
        );
        assert_eq!(bundler.locale_loading.as_str(), "dynamic");

        let invalid = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [targets.ts]

            [targets.ts.bundler]
            sources = ["src"]
            locale_loading = "lazy"
            "#,
        )
        .expect_err("unknown locale loading policy");
        assert!(invalid
            .to_string()
            .contains("targets.ts.bundler.locale_loading"));
    }

    #[test]
    fn parses_explicit_unused_message_scope() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]

            [paths]
            schema = "linguini/schema"
            locale = "linguini/locale"

            [analysis.unused_messages]
            sources = [" ./src/ ", "tests//ui"]
            exclude = ["src/generated"]
            ignore = ["main.runtime_selected", "admin"]
            "#,
        )
        .expect("valid config");

        let unused = config
            .analysis
            .unused_messages
            .expect("unused-message analysis");
        assert_eq!(unused.sources, ["src", "tests/ui"]);
        assert_eq!(unused.exclude, ["src/generated"]);
        assert_eq!(unused.ignore, ["main.runtime_selected", "admin"]);
    }

    #[test]
    fn rejects_incomplete_or_unsafe_unused_message_scope() {
        for analysis in [
            "[analysis.unused_messages]\nsources = []",
            "[analysis.unused_messages]\nsources = [\"../src\"]",
            "[analysis.unused_messages]\nsources = [\"src\", \"./src\"]",
            "[analysis.unused_messages]\nsources = [\"src\"]\nexclude = [\"/tmp\"]",
            "[analysis.unused_messages]\nsources = [\"src/app\"]\nexclude = [\"src\"]",
            "[analysis.unused_messages]\nsources = [\"SRC/app\"]\nexclude = [\"src\"]",
            "[analysis.unused_messages]\nsources = [\"src\"]\nignore = [\"main..title\"]",
            "[analysis.unused_messages]\nsources = [\"src\"]\nignore = [\"main\", \"main\"]",
        ] {
            let source = format!(
                r#"
                [project]
                name = "shop"
                default_locale = "en"
                locales = ["en"]
                [paths]
                schema = "schema"
                locale = "locale"
                {analysis}
                "#
            );
            assert!(parse_config(&source).is_err(), "{analysis}");
        }
    }

    #[test]
    fn rejects_duplicate_toml_keys_and_unknown_sections() {
        let duplicate = r#"
            [project]
            name = "shop"
            name = "again"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
        "#;
        assert!(parse_config(duplicate).is_err());

        let unknown = r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [webpack]
            enabled = true
        "#;
        assert!(parse_config(unknown).is_err());
    }

    #[test]
    fn validates_required_fields() {
        let error = parse_config("[project]\nname = \"shop\"").expect_err("missing fields");
        assert_eq!(
            error.to_string(),
            "missing required config field `project.default_locale`"
        );
    }
}
