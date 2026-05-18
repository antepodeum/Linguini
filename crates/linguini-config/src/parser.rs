use crate::error::{ConfigError, ConfigResult};
use crate::model::{
    LinguiniConfig, PathsConfig, ProjectConfig, TargetsConfig, TypeScriptTargetConfig, WebConfig,
};

#[derive(Default)]
struct ProjectBuilder {
    name: Option<String>,
    default_locale: Option<String>,
    locales: Option<Vec<String>>,
}

#[derive(Default)]
struct PathsBuilder {
    schema: Option<String>,
    locale: Option<String>,
}

#[derive(Default)]
struct TargetsBuilder {
    ts: Option<TypeScriptTargetBuilder>,
}

#[derive(Default)]
struct WebBuilder {
    strategy: Option<Vec<String>>,
    cookie_name: Option<String>,
    cookie_path: Option<String>,
    cookie_domain: Option<String>,
    cookie_max_age: Option<u64>,
    cookie_same_site: Option<String>,
    cookie_secure: Option<bool>,
    cookie_http_only: Option<bool>,
    local_storage_key: Option<String>,
    global_variable_name: Option<String>,
    prefix_default_locale: Option<bool>,
    base_path: Option<String>,
    trailing_slash: Option<String>,
    redirect: Option<bool>,
    origin: Option<String>,
    exclude: Option<Vec<String>>,
    localize_links: Option<bool>,
}

#[derive(Default)]
struct TypeScriptTargetBuilder {
    out: Option<String>,
    module: Option<String>,
    declaration: Option<bool>,
    tree_shaking: Option<bool>,
    messages: Option<Vec<String>>,
    framework: Option<String>,
}

pub fn parse_config(source: &str) -> ConfigResult<LinguiniConfig> {
    let mut section = String::new();
    let mut project = ProjectBuilder::default();
    let mut paths = PathsBuilder::default();
    let mut targets = TargetsBuilder::default();
    let mut web = WebBuilder::default();

    for raw_line in source.lines() {
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|line| line.strip_suffix(']'))
        {
            match name {
                "project" | "paths" | "targets.ts" | "web" => section = name.to_owned(),
                name => return Err(ConfigError::UnexpectedSection(name.to_owned())),
            }
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(ConfigError::UnknownKey {
                section: section.clone(),
                key: line.to_owned(),
            });
        };

        assign_value(
            &section,
            key.trim(),
            value.trim(),
            &mut project,
            &mut paths,
            &mut targets,
            &mut web,
        )?;
    }

    let config = LinguiniConfig {
        project: ProjectConfig {
            name: required(project.name, "project.name")?,
            default_locale: required(project.default_locale, "project.default_locale")?,
            locales: required(project.locales, "project.locales")?,
        },
        paths: PathsConfig {
            schema: required(paths.schema, "paths.schema")?,
            locale: required(paths.locale, "paths.locale")?,
        },
        targets: TargetsConfig {
            ts: targets.ts.map(|ts| TypeScriptTargetConfig {
                out: ts
                    .out
                    .unwrap_or_else(|| "src/generated/linguini".to_owned()),
                module: ts.module.unwrap_or_else(|| "esm".to_owned()),
                declaration: ts.declaration.unwrap_or(true),
                tree_shaking: ts.tree_shaking.unwrap_or(false),
                messages: ts.messages.unwrap_or_default(),
                framework: ts.framework,
            }),
        },
        web: build_web_config(web),
    };

    config.validate()?;
    Ok(config)
}

fn assign_value(
    section: &str,
    key: &str,
    value: &str,
    project: &mut ProjectBuilder,
    paths: &mut PathsBuilder,
    targets: &mut TargetsBuilder,
    web: &mut WebBuilder,
) -> ConfigResult<()> {
    match (section, key) {
        ("project", "name") => assign_string(&mut project.name, key, value),
        ("project", "default_locale") => assign_string(&mut project.default_locale, key, value),
        ("project", "locales") => assign_array(&mut project.locales, key, value),
        ("paths", "schema") => assign_string(&mut paths.schema, key, value),
        ("paths", "locale") => assign_string(&mut paths.locale, key, value),
        ("paths", "cache") => {
            parse_string(value)?;
            Ok(())
        }
        ("targets.ts", "out") => {
            let ts = targets
                .ts
                .get_or_insert_with(TypeScriptTargetBuilder::default);
            assign_string(&mut ts.out, key, value)
        }
        ("targets.ts", "module") => {
            let ts = targets
                .ts
                .get_or_insert_with(TypeScriptTargetBuilder::default);
            assign_string(&mut ts.module, key, value)
        }
        ("targets.ts", "declaration") => {
            let ts = targets
                .ts
                .get_or_insert_with(TypeScriptTargetBuilder::default);
            assign_bool(&mut ts.declaration, key, value)
        }
        ("targets.ts", "tree_shaking") => {
            let ts = targets
                .ts
                .get_or_insert_with(TypeScriptTargetBuilder::default);
            assign_bool(&mut ts.tree_shaking, key, value)
        }
        ("targets.ts", "messages") => {
            let ts = targets
                .ts
                .get_or_insert_with(TypeScriptTargetBuilder::default);
            assign_array(&mut ts.messages, key, value)
        }
        ("targets.ts", "framework") => {
            let ts = targets
                .ts
                .get_or_insert_with(TypeScriptTargetBuilder::default);
            assign_string(&mut ts.framework, key, value)
        }
        ("web", "strategy") => assign_array(&mut web.strategy, key, value),
        ("web", "cookie_name") => assign_string(&mut web.cookie_name, key, value),
        ("web", "cookie_path") => assign_string(&mut web.cookie_path, key, value),
        ("web", "cookie_domain") => assign_string(&mut web.cookie_domain, key, value),
        ("web", "cookie_max_age") => assign_u64(&mut web.cookie_max_age, key, value),
        ("web", "cookie_same_site") => assign_string(&mut web.cookie_same_site, key, value),
        ("web", "cookie_secure") => assign_bool(&mut web.cookie_secure, key, value),
        ("web", "cookie_http_only") => assign_bool(&mut web.cookie_http_only, key, value),
        ("web", "local_storage_key") => assign_string(&mut web.local_storage_key, key, value),
        ("web", "global_variable_name") => assign_string(&mut web.global_variable_name, key, value),
        ("web", "prefix_default_locale") => assign_bool(&mut web.prefix_default_locale, key, value),
        ("web", "base_path") => assign_string(&mut web.base_path, key, value),
        ("web", "trailing_slash") => assign_string(&mut web.trailing_slash, key, value),
        ("web", "redirect") => assign_bool(&mut web.redirect, key, value),
        ("web", "origin") => assign_string(&mut web.origin, key, value),
        ("web", "exclude") => assign_array(&mut web.exclude, key, value),
        ("web", "localize_links") => assign_bool(&mut web.localize_links, key, value),
        (section, key) => Err(ConfigError::UnknownKey {
            section: section.to_owned(),
            key: key.to_owned(),
        }),
    }
}

fn assign_string(slot: &mut Option<String>, key: &str, value: &str) -> ConfigResult<()> {
    if slot.is_some() {
        return Err(ConfigError::DuplicateKey(key.to_owned()));
    }

    *slot = Some(parse_string(value)?);
    Ok(())
}

fn assign_array(slot: &mut Option<Vec<String>>, key: &str, value: &str) -> ConfigResult<()> {
    if slot.is_some() {
        return Err(ConfigError::DuplicateKey(key.to_owned()));
    }

    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Err(ConfigError::InvalidArray(value.to_owned()));
    };

    let mut values = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();

        if part.is_empty() {
            continue;
        }

        values.push(parse_string(part)?);
    }

    *slot = Some(values);
    Ok(())
}

fn assign_bool(slot: &mut Option<bool>, key: &str, value: &str) -> ConfigResult<()> {
    if slot.is_some() {
        return Err(ConfigError::DuplicateKey(key.to_owned()));
    }

    *slot = Some(match value {
        "true" => true,
        "false" => false,
        value => return Err(ConfigError::InvalidString(value.to_owned())),
    });
    Ok(())
}

fn assign_u64(slot: &mut Option<u64>, key: &str, value: &str) -> ConfigResult<()> {
    if slot.is_some() {
        return Err(ConfigError::DuplicateKey(key.to_owned()));
    }

    *slot = Some(
        value
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidString(value.to_owned()))?,
    );
    Ok(())
}

fn parse_string(value: &str) -> ConfigResult<String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| ConfigError::InvalidString(value.to_owned()))
}

fn required<T>(value: Option<T>, field: &'static str) -> ConfigResult<T> {
    value.ok_or(ConfigError::MissingField(field))
}

fn build_web_config(web: WebBuilder) -> WebConfig {
    let defaults = WebConfig::default();
    WebConfig {
        strategy: web.strategy.unwrap_or(defaults.strategy),
        cookie_name: web.cookie_name.unwrap_or(defaults.cookie_name),
        cookie_path: web.cookie_path.unwrap_or(defaults.cookie_path),
        cookie_domain: web.cookie_domain.or(defaults.cookie_domain),
        cookie_max_age: web.cookie_max_age.unwrap_or(defaults.cookie_max_age),
        cookie_same_site: web.cookie_same_site.unwrap_or(defaults.cookie_same_site),
        cookie_secure: web.cookie_secure.unwrap_or(defaults.cookie_secure),
        cookie_http_only: web.cookie_http_only.unwrap_or(defaults.cookie_http_only),
        local_storage_key: web.local_storage_key.unwrap_or(defaults.local_storage_key),
        global_variable_name: web.global_variable_name.or(defaults.global_variable_name),
        prefix_default_locale: web
            .prefix_default_locale
            .unwrap_or(defaults.prefix_default_locale),
        base_path: web.base_path.unwrap_or(defaults.base_path),
        trailing_slash: web.trailing_slash.unwrap_or(defaults.trailing_slash),
        redirect: web.redirect.unwrap_or(defaults.redirect),
        origin: web.origin.or(defaults.origin),
        exclude: web.exclude.unwrap_or(defaults.exclude),
        localize_links: web.localize_links.unwrap_or(defaults.localize_links),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_config;

    #[test]
    fn parses_required_project_config() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "ru"
            locales = ["ru", "en-US"]

            [paths]
            schema = "linguini/schema"
            locale = "linguini/locale"
            "#,
        )
        .expect("valid config");

        assert_eq!(config.project.name, "shop");
        assert_eq!(config.project.locales, ["ru", "en-US"]);
        assert_eq!(config.paths.schema, "linguini/schema");
        assert_eq!(config.paths.locale, "linguini/locale");
        assert!(config.targets.ts.is_none());
    }

    #[test]
    fn accepts_legacy_cache_path_without_exposing_it_to_runtime() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]

            [paths]
            schema = "linguini/schema"
            locale = "linguini/locale"
            cache = ".linguini/cache"
            "#,
        )
        .expect("legacy config");

        assert_eq!(config.paths.schema, "linguini/schema");
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
            module = "esm"
            declaration = false
            tree_shaking = true
            messages = ["delivery", "email_input.label"]
            framework = "sveltekit"
            "#,
        )
        .expect("valid config");

        let target = config.targets.ts.expect("ts target");
        assert_eq!(target.out, "src/generated/linguini");
        assert_eq!(target.module, "esm");
        assert!(!target.declaration);
        assert!(target.tree_shaking);
        assert_eq!(target.messages, ["delivery", "email_input.label"]);
        assert_eq!(target.framework.as_deref(), Some("sveltekit"));
    }

    #[test]
    fn rejects_unknown_typescript_framework() {
        let error = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en"]

            [paths]
            schema = "schema"
            locale = "locale"

            [targets.ts]
            framework = "react"
            "#,
        )
        .expect_err("invalid framework");

        assert_eq!(error.to_string(), "invalid string value `react`");
    }

    #[test]
    fn parses_web_runtime_config() {
        let config = parse_config(
            r#"
            [project]
            name = "shop"
            default_locale = "en"
            locales = ["en", "ru"]

            [paths]
            schema = "schema"
            locale = "locale"

            [web]
            strategy = ["url", "cookie", "header", "baseLocale"]
            cookie_name = "SHOP_LOCALE"
            cookie_path = "/shop"
            cookie_domain = "example.com"
            cookie_max_age = 86400
            cookie_same_site = "strict"
            cookie_secure = true
            cookie_http_only = true
            local_storage_key = "SHOP_LOCALE"
            global_variable_name = "__SHOP_LOCALE__"
            prefix_default_locale = true
            base_path = "/shop"
            trailing_slash = "never"
            redirect = false
            origin = "https://example.com"
            exclude = ["/api/**", "/assets/**"]
            localize_links = false
            "#,
        )
        .expect("valid config");

        assert_eq!(
            config.web.strategy,
            ["url", "cookie", "header", "baseLocale"]
        );
        assert_eq!(config.web.cookie_name, "SHOP_LOCALE");
        assert_eq!(config.web.cookie_path, "/shop");
        assert_eq!(config.web.cookie_domain.as_deref(), Some("example.com"));
        assert_eq!(config.web.cookie_max_age, 86400);
        assert_eq!(config.web.cookie_same_site, "strict");
        assert!(config.web.cookie_secure);
        assert!(config.web.cookie_http_only);
        assert_eq!(config.web.local_storage_key, "SHOP_LOCALE");
        assert_eq!(
            config.web.global_variable_name.as_deref(),
            Some("__SHOP_LOCALE__")
        );
        assert!(config.web.prefix_default_locale);
        assert_eq!(config.web.base_path, "/shop");
        assert_eq!(config.web.trailing_slash, "never");
        assert!(!config.web.redirect);
        assert_eq!(config.web.origin.as_deref(), Some("https://example.com"));
        assert_eq!(config.web.exclude, ["/api/**", "/assets/**"]);
        assert!(!config.web.localize_links);
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
