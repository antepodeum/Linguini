use crate::error::{ConfigError, ConfigResult};
use crate::model::{
    LinguiniConfig, PathsConfig, ProjectConfig, TargetsConfig, TypeScriptTargetConfig,
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
struct TypeScriptTargetBuilder {
    out: Option<String>,
    module: Option<String>,
    declaration: Option<bool>,
    tree_shaking: Option<bool>,
    messages: Option<Vec<String>>,
}

pub fn parse_config(source: &str) -> ConfigResult<LinguiniConfig> {
    let mut section = String::new();
    let mut project = ProjectBuilder::default();
    let mut paths = PathsBuilder::default();
    let mut targets = TargetsBuilder::default();

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
                "project" | "paths" | "targets.ts" => section = name.to_owned(),
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
            }),
        },
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
            "#,
        )
        .expect("valid config");

        let target = config.targets.ts.expect("ts target");
        assert_eq!(target.out, "src/generated/linguini");
        assert_eq!(target.module, "esm");
        assert!(!target.declaration);
        assert!(target.tree_shaking);
        assert_eq!(target.messages, ["delivery", "email_input.label"]);
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
