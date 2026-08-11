use crate::error::{ConfigError, ConfigResult};
use crate::model::{
    canonicalize_locale_tag, AnalysisConfig, CanonicalMode, CookiePath, LinguiniConfig, LinkMode,
    LocalePrefixMode, LocaleSource, LocaleSwitchPlan, PathsConfig, ProjectConfig, SameSite,
    SecurePolicy, TargetsConfig, TypeScriptBundlerConfig, TypeScriptBundlerDynamicConfig,
    TypeScriptBundlerDynamicMode, TypeScriptBundlerLocaleLoading, TypeScriptTargetConfig,
    UnusedMessagesConfig, WebConfig, WebCookieConfig, WebLinksConfig, WebLocalStorageConfig,
    WebLocaleConfig, WebRoutesConfig, WebRoutingConfig, WebSwitchRouteConfig,
};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    project: Option<RawProjectConfig>,
    paths: Option<RawPathsConfig>,
    targets: Option<RawTargetsConfig>,
    analysis: Option<RawAnalysisConfig>,
    web: Option<RawWebConfig>,
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
    module: Option<toml::Value>,
    declaration: Option<bool>,
    gitignore: Option<bool>,
    tree_shaking: Option<bool>,
    messages: Option<Vec<String>>,
    framework: Option<String>,
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

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebConfig {
    routing: Option<RawWebRoutingConfig>,
    locale: Option<RawWebLocaleConfig>,
    cookie: Option<RawWebCookieConfig>,
    local_storage: Option<RawWebLocalStorageConfig>,
    links: Option<RawWebLinksConfig>,
    routes: Option<RawWebRoutesConfig>,
    switch_route: Option<RawWebSwitchRouteConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebRoutingConfig {
    locale_prefix: Option<String>,
    canonical: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebLocaleConfig {
    sources: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebCookieConfig {
    name: Option<String>,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<String>,
    same_site: Option<String>,
    secure: Option<RawSecurePolicy>,
    http_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawSecurePolicy {
    Boolean(bool),
    Name(String),
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebLocalStorageConfig {
    key: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebLinksConfig {
    mode: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebRoutesConfig {
    exclude: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWebSwitchRouteConfig {
    path: Option<String>,
    return_query: Option<String>,
    status: Option<u16>,
}

pub fn parse_config(source: &str) -> ConfigResult<LinguiniConfig> {
    let raw: RawConfig = toml::from_str(source).map_err(|error| {
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
    let web = build_web(raw.web, &project.name)?;
    let config = LinguiniConfig {
        project,
        paths,
        targets,
        analysis,
        web,
    };
    config.validate()?;
    Ok(config)
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
    if raw.module.is_some() {
        return Err(ConfigError::RemovedField {
            field: "targets.ts.module",
            replacement: "generated web modules are always ESM",
        });
    }
    Ok(TypeScriptTargetConfig {
        out: normalize_project_path(
            raw.out
                .unwrap_or_else(|| "src/generated/linguini".to_owned()),
        ),
        declaration: raw.declaration.unwrap_or(true),
        gitignore: raw.gitignore.unwrap_or(true),
        tree_shaking: raw.tree_shaking.unwrap_or(false),
        messages: raw.messages.unwrap_or_default(),
        framework: raw.framework,
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

fn build_web(raw: Option<RawWebConfig>, project_name: &str) -> ConfigResult<WebConfig> {
    let configured = raw.is_some();
    let raw = raw.unwrap_or_default();
    let raw_routing = raw.routing.unwrap_or_default();
    let locale_prefix = match raw_routing
        .locale_prefix
        .as_deref()
        .unwrap_or("except-default")
    {
        "always" => LocalePrefixMode::Always,
        "except-default" => LocalePrefixMode::ExceptDefault,
        "never" => LocalePrefixMode::Never,
        value => return Err(ConfigError::InvalidString(value.to_owned())),
    };
    let canonical = match raw_routing.canonical.as_deref().unwrap_or("redirect") {
        "redirect" => CanonicalMode::Redirect,
        "preserve" => CanonicalMode::Preserve,
        value => return Err(ConfigError::InvalidString(value.to_owned())),
    };

    let raw_sources = raw.locale.unwrap_or_default().sources;
    let sources = match raw_sources {
        Some(sources) => sources
            .iter()
            .map(|source| LocaleSource::parse(source))
            .collect::<ConfigResult<Vec<_>>>()?,
        None if locale_prefix == LocalePrefixMode::Never => {
            vec![LocaleSource::Cookie, LocaleSource::AcceptLanguage]
        }
        None => vec![
            LocaleSource::Path,
            LocaleSource::Cookie,
            LocaleSource::AcceptLanguage,
        ],
    };

    let wants_cookie = sources.contains(&LocaleSource::Cookie);
    if !wants_cookie && raw.cookie.is_some() {
        return Err(ConfigError::InvalidString(
            "web.cookie is configured but `cookie` is absent from web.locale.sources".to_owned(),
        ));
    }
    let cookie = wants_cookie
        .then(|| build_cookie(raw.cookie.unwrap_or_default(), project_name))
        .transpose()?;

    let wants_storage = sources.contains(&LocaleSource::LocalStorage);
    if !wants_storage && raw.local_storage.is_some() {
        return Err(ConfigError::InvalidString(
            "web.local_storage is configured but `local-storage` is absent from web.locale.sources"
                .to_owned(),
        ));
    }
    let local_storage = wants_storage.then(|| WebLocalStorageConfig {
        key: raw
            .local_storage
            .unwrap_or_default()
            .key
            .unwrap_or_else(|| format!("linguini:{}:locale", project_key(project_name))),
    });

    let links = match raw
        .links
        .unwrap_or_default()
        .mode
        .as_deref()
        .unwrap_or("transform")
    {
        "transform" => LinkMode::Transform,
        "runtime" => LinkMode::Runtime,
        "manual" => LinkMode::Manual,
        value => return Err(ConfigError::InvalidString(value.to_owned())),
    };
    let routes = WebRoutesConfig {
        exclude: raw.routes.unwrap_or_default().exclude.unwrap_or_default(),
    };
    let switch_route = raw.switch_route.map(|route| WebSwitchRouteConfig {
        path: route
            .path
            .unwrap_or_else(|| "/_linguini/locale/{locale}".to_owned()),
        return_query: route.return_query.unwrap_or_else(|| "return".to_owned()),
        status: route.status.unwrap_or(303),
    });
    let locale_switch = LocaleSwitchPlan::compile(&sources);

    Ok(WebConfig {
        configured,
        routing: WebRoutingConfig {
            locale_prefix,
            canonical,
        },
        locale: WebLocaleConfig { sources },
        cookie,
        local_storage,
        links: WebLinksConfig { mode: links },
        routes,
        switch_route,
        locale_switch,
    })
}

fn build_cookie(raw: RawWebCookieConfig, project_name: &str) -> ConfigResult<WebCookieConfig> {
    let path = match raw.path.as_deref().unwrap_or("auto") {
        "auto" => CookiePath::Auto,
        path => CookiePath::Explicit(path.to_owned()),
    };
    let same_site = match raw.same_site.as_deref().unwrap_or("lax") {
        "lax" => SameSite::Lax,
        "strict" => SameSite::Strict,
        "none" => SameSite::None,
        value => return Err(ConfigError::InvalidString(value.to_owned())),
    };
    let secure = match raw
        .secure
        .unwrap_or(RawSecurePolicy::Name("auto".to_owned()))
    {
        RawSecurePolicy::Boolean(true) => SecurePolicy::Always,
        RawSecurePolicy::Boolean(false) => SecurePolicy::Never,
        RawSecurePolicy::Name(value) if value == "auto" => SecurePolicy::Auto,
        RawSecurePolicy::Name(value) => return Err(ConfigError::InvalidString(value)),
    };

    Ok(WebCookieConfig {
        name: raw
            .name
            .unwrap_or_else(|| format!("LINGUINI_{}_LOCALE", environment_key(project_name))),
        path,
        domain: raw.domain,
        max_age_seconds: parse_duration(
            raw.max_age.as_deref().unwrap_or("365d"),
            "web.cookie.max_age",
        )?,
        same_site,
        secure,
        http_only: raw.http_only.unwrap_or(false),
    })
}

fn parse_duration(value: &str, field: &'static str) -> ConfigResult<u64> {
    let split = value
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(value.len());
    let (amount, unit) = value.split_at(split);
    if amount.is_empty() || unit.len() > 1 {
        return Err(ConfigError::InvalidString(format!("{field} = {value}")));
    }
    let amount = amount
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidString(format!("{field} = {value}")))?;
    let multiplier = match unit {
        "" | "s" => 1,
        "m" => 60,
        "h" => 60 * 60,
        "d" => 24 * 60 * 60,
        "w" => 7 * 24 * 60 * 60,
        _ => return Err(ConfigError::InvalidString(format!("{field} = {value}"))),
    };
    amount
        .checked_mul(multiplier)
        .ok_or_else(|| ConfigError::InvalidString(format!("{field} = {value}")))
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

fn environment_key(project_name: &str) -> String {
    let key = project_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    key.trim_matches('_').to_owned()
}

fn project_key(project_name: &str) -> String {
    let key = project_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    key.trim_matches('-').to_owned()
}

fn required<T>(value: Option<T>, field: &'static str) -> ConfigResult<T> {
    value.ok_or(ConfigError::MissingField(field))
}

#[cfg(test)]
mod tests {
    use super::parse_config;
    use crate::{
        CanonicalMode, CookiePath, LinkMode, LocalePrefixMode, LocaleSource, SameSite,
        SecurePolicy, TypeScriptBundlerDynamicMode, TypeScriptBundlerLocaleLoading,
    };

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
        assert!(!config.web.configured);
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
            framework = "sveltekit"
            "#,
        )
        .expect("valid config");

        let target = config.targets.ts.expect("ts target");
        assert_eq!(target.out, "src/generated/linguini");
        assert!(!target.declaration);
        assert!(!target.gitignore);
        assert!(target.tree_shaking);
        assert_eq!(target.messages, ["delivery", "email_input.label"]);
        assert_eq!(target.framework.as_deref(), Some("sveltekit"));
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
            framework = "svelte"
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
                framework = "svelte"
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
        .expect_err("bundler framework capability required");
        assert!(without_framework
            .to_string()
            .contains("targets.ts.bundler requires targets.ts.framework"));
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
            framework = "svelte"
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
                framework = "svelte"
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
            framework = "svelte"
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
            framework = "svelte"
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
    fn parses_nested_web_policy_and_compiles_switch_plan() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en", "ru"]
            [paths]
            schema = "schema"
            locale = "locale"

            [web.routing]
            locale_prefix = "always"
            canonical = "preserve"
            [web.locale]
            sources = ["path", "cookie", "local-storage", "accept-language"]
            [web.cookie]
            name = "SHOP_LOCALE"
            path = "/shop"
            domain = "example.com"
            max_age = "1d"
            same_site = "strict"
            secure = true
            http_only = false
            [web.local_storage]
            key = "shop:locale"
            [web.links]
            mode = "manual"
            [web.routes]
            exclude = ["/api/**", "/assets/**"]
            [web.switch_route]
            path = "/_linguini/locale/{locale}"
            return_query = "return"
            status = 303
            "#,
        )
        .expect("valid config");

        assert!(config.web.configured);
        assert_eq!(config.web.routing.locale_prefix, LocalePrefixMode::Always);
        assert_eq!(config.web.routing.canonical, CanonicalMode::Preserve);
        assert_eq!(
            config.web.locale.sources,
            [
                LocaleSource::Path,
                LocaleSource::Cookie,
                LocaleSource::LocalStorage,
                LocaleSource::AcceptLanguage
            ]
        );
        let features = config.web.features();
        assert_eq!(features.source_order, config.web.locale.sources);
        assert_eq!(features.locale_prefix, LocalePrefixMode::Always);
        assert_eq!(features.canonical, CanonicalMode::Preserve);
        assert_eq!(features.links, LinkMode::Manual);
        assert!(features.cookie.is_some());
        assert!(features.local_storage.is_some());
        assert!(features.locale_switch.writes_path);
        let cookie = config.web.cookie.expect("cookie feature");
        assert_eq!(cookie.name, "SHOP_LOCALE");
        assert_eq!(cookie.path, CookiePath::Explicit("/shop".to_owned()));
        assert_eq!(cookie.domain.as_deref(), Some("example.com"));
        assert_eq!(cookie.max_age_seconds, 86_400);
        assert_eq!(cookie.same_site, SameSite::Strict);
        assert_eq!(cookie.secure, SecurePolicy::Always);
        assert_eq!(config.web.links.mode, LinkMode::Manual);
        assert!(config.web.locale_switch.writes_path);
        assert!(config.web.locale_switch.writes_cookie);
        assert!(config.web.locale_switch.writes_local_storage);
    }

    #[test]
    fn derives_web_defaults_from_routing() {
        let prefixed = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [web]
            "#,
        )
        .expect("prefixed defaults");
        assert_eq!(
            prefixed.web.locale.sources,
            [
                LocaleSource::Path,
                LocaleSource::Cookie,
                LocaleSource::AcceptLanguage
            ]
        );

        let pathless = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [web.routing]
            locale_prefix = "never"
            "#,
        )
        .expect("pathless defaults");
        assert_eq!(
            pathless.web.locale.sources,
            [LocaleSource::Cookie, LocaleSource::AcceptLanguage]
        );
    }

    #[test]
    fn accepts_empty_locale_sources_but_rejects_impossible_switch_route() {
        let base = r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [web.routing]
            locale_prefix = "never"
            [web.locale]
            sources = []
        "#;
        assert!(parse_config(base).is_ok());

        let impossible = format!("{base}\n[web.switch_route]\n");
        assert!(parse_config(&impossible).is_err());
    }

    #[test]
    fn rejects_path_source_in_pathless_mode_and_duplicate_sources() {
        for sources in [
            r#"["path"]"#,
            r#"["cookie", "cookie"]"#,
            r#"["preferredLanguage"]"#,
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
                [web.routing]
                locale_prefix = "never"
                [web.locale]
                sources = {sources}
                "#
            );
            assert!(parse_config(&source).is_err(), "{sources}");
        }
    }

    #[test]
    fn enforces_same_site_none_secure_constraint() {
        let source = r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]
            [paths]
            schema = "schema"
            locale = "locale"
            [web.locale]
            sources = ["cookie"]
            [web.cookie]
            same_site = "none"
            secure = "auto"
        "#;
        assert!(parse_config(source).is_err());
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
