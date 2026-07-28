use crate::error::{ConfigError, ConfigResult};
use std::path::{Component, Path};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LinguiniConfig {
    pub project: ProjectConfig,
    pub paths: PathsConfig,
    pub targets: TargetsConfig,
    pub web: WebConfig,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectConfig {
    pub name: String,
    pub default_locale: String,
    pub locales: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PathsConfig {
    pub schema: String,
    pub locale: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct TargetsConfig {
    pub ts: Option<TypeScriptTargetConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeScriptTargetConfig {
    pub out: String,
    pub module: String,
    pub declaration: bool,
    pub gitignore: bool,
    pub tree_shaking: bool,
    pub messages: Vec<String>,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebConfig {
    pub configured: bool,
    pub strategy: Vec<String>,
    pub cookie_name: String,
    pub cookie_path: String,
    pub cookie_domain: Option<String>,
    pub cookie_max_age: u64,
    pub cookie_same_site: String,
    pub cookie_secure: bool,
    pub cookie_http_only: bool,
    pub local_storage_key: String,
    pub global_variable_name: Option<String>,
    pub prefix_default_locale: bool,
    pub base_path: String,
    pub trailing_slash: String,
    pub redirect: bool,
    pub origin: Option<String>,
    pub exclude: Vec<String>,
    pub localize_links: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            configured: false,
            strategy: vec![
                "url".to_owned(),
                "cookie".to_owned(),
                "localStorage".to_owned(),
                "preferredLanguage".to_owned(),
                "baseLocale".to_owned(),
            ],
            cookie_name: "LINGUINI_LOCALE".to_owned(),
            cookie_path: "/".to_owned(),
            cookie_domain: None,
            cookie_max_age: 60 * 60 * 24 * 365,
            cookie_same_site: "lax".to_owned(),
            cookie_secure: false,
            cookie_http_only: false,
            local_storage_key: "LINGUINI_LOCALE".to_owned(),
            global_variable_name: None,
            prefix_default_locale: false,
            base_path: String::new(),
            trailing_slash: "ignore".to_owned(),
            redirect: true,
            origin: None,
            exclude: Vec::new(),
            localize_links: true,
        }
    }
}

impl LinguiniConfig {
    pub fn validate(&self) -> ConfigResult<()> {
        validate_relative_path("paths.schema", &self.paths.schema)?;
        validate_relative_path("paths.locale", &self.paths.locale)?;

        validate_locale_tag(&self.project.default_locale)?;

        if !self
            .project
            .locales
            .iter()
            .any(|locale| locale == &self.project.default_locale)
        {
            return Err(ConfigError::MissingField("project.locales default_locale"));
        }

        for locale in &self.project.locales {
            validate_locale_tag(locale)?;
        }

        if let Some(ts) = &self.targets.ts {
            validate_relative_path("targets.ts.out", &ts.out)?;
            reject_path_overlap("targets.ts.out", &ts.out, "paths.schema", &self.paths.schema)?;
            reject_path_overlap("targets.ts.out", &ts.out, "paths.locale", &self.paths.locale)?;
            if ts.module != "esm" {
                return Err(ConfigError::InvalidString(ts.module.clone()));
            }
            if !ts.tree_shaking && !ts.messages.is_empty() {
                return Err(ConfigError::InvalidString(
                    "targets.ts.messages requires tree_shaking = true".to_owned(),
                ));
            }
            if let Some(framework) = &ts.framework {
                match framework.as_str() {
                    "svelte" | "sveltekit" => {}
                    value => return Err(ConfigError::InvalidString(value.to_owned())),
                }
            }
        }

        validate_web_strategy(&self.web.strategy)?;
        match self.web.trailing_slash.as_str() {
            "ignore" | "always" | "never" | "directory" => {}
            value => return Err(ConfigError::InvalidString(value.to_owned())),
        }
        match self.web.cookie_same_site.as_str() {
            "lax" | "strict" | "none" => {}
            value => return Err(ConfigError::InvalidString(value.to_owned())),
        }

        Ok(())
    }
}

fn validate_relative_path(field: &'static str, value: &str) -> ConfigResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "path must not be empty",
        });
    }

    let portable = trimmed.replace('\\', "/");
    let has_windows_prefix = portable
        .as_bytes()
        .get(1)
        .is_some_and(|character| *character == b':');
    if portable.starts_with('/') || has_windows_prefix || Path::new(trimmed).is_absolute() {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "path must be project-relative",
        });
    }

    if portable.split('/').any(|segment| segment == "..")
        || Path::new(trimmed)
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "parent traversal is not allowed",
        });
    }

    if portable
        .split('/')
        .all(|segment| segment.is_empty() || segment == ".")
    {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "path must not resolve to the project root",
        });
    }

    Ok(())
}

fn reject_path_overlap(
    left_field: &'static str,
    left: &str,
    right_field: &'static str,
    right: &str,
) -> ConfigResult<()> {
    let left = portable_components(left);
    let right = portable_components(right);
    if left.starts_with(&right) || right.starts_with(&left) {
        return Err(ConfigError::InvalidPath {
            field: left_field,
            value: left.join("/"),
            reason: match right_field {
                "paths.schema" => "path overlaps the schema source root",
                "paths.locale" => "path overlaps the locale source root",
                _ => "path overlaps another project root",
            },
        });
    }
    Ok(())
}

fn portable_components(value: &str) -> Vec<String> {
    value
        .replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .map(str::to_owned)
        .collect()
}

fn validate_web_strategy(strategy: &[String]) -> ConfigResult<()> {
    if strategy.is_empty() {
        return Err(ConfigError::InvalidArray("web.strategy".to_owned()));
    }
    for item in strategy {
        let is_builtin = matches!(
            item.as_str(),
            "url"
                | "cookie"
                | "localStorage"
                | "header"
                | "navigator"
                | "preferredLanguage"
                | "globalVariable"
                | "baseLocale"
        );
        if !is_builtin && !item.starts_with("custom-") {
            return Err(ConfigError::InvalidString(item.clone()));
        }
    }
    Ok(())
}

pub fn validate_locale_tag(tag: &str) -> ConfigResult<()> {
    let mut parts = tag.split('-');
    let Some(language) = parts.next() else {
        return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
    };

    if language.len() < 2
        || language.len() > 3
        || !language
            .chars()
            .all(|character| character.is_ascii_lowercase())
    {
        return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
    }

    for part in parts {
        let valid = (part.len() == 2
            && part.chars().all(|character| character.is_ascii_uppercase()))
            || (part.len() == 4
                && part.chars().enumerate().all(|(index, character)| {
                    if index == 0 {
                        character.is_ascii_uppercase()
                    } else {
                        character.is_ascii_lowercase()
                    }
                }));

        if !valid {
            return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_locale_tag, LinguiniConfig, PathsConfig, ProjectConfig, TargetsConfig};

    #[test]
    fn accepts_spec_locale_tags() {
        for tag in ["ru", "en", "en-US", "pt-BR", "zh-Hant"] {
            assert!(validate_locale_tag(tag).is_ok(), "{tag}");
        }
    }

    #[test]
    fn rejects_non_bcp47_like_locale_tags() {
        for tag in ["r", "EN", "en-us", "zh-hant", "en-US-extra"] {
            assert!(validate_locale_tag(tag).is_err(), "{tag}");
        }
    }

    #[test]
    fn rejects_unsafe_or_overlapping_codegen_paths() {
        for out in [
            "/tmp/generated",
            "../generated",
            r"..\generated",
            r"C:\generated",
            ".",
            "schema/generated",
            "locales",
        ] {
            let config = LinguiniConfig {
                project: ProjectConfig {
                    name: "shop".to_owned(),
                    default_locale: "en".to_owned(),
                    locales: vec!["en".to_owned()],
                },
                paths: PathsConfig {
                    schema: "schema".to_owned(),
                    locale: "locales".to_owned(),
                },
                targets: TargetsConfig {
                    ts: Some(super::TypeScriptTargetConfig {
                        out: out.to_owned(),
                        module: "esm".to_owned(),
                        declaration: true,
                        gitignore: true,
                        tree_shaking: false,
                        messages: Vec::new(),
                        framework: None,
                    }),
                },
                web: super::WebConfig::default(),
            };

            assert!(config.validate().is_err(), "{out}");
        }
    }
}
