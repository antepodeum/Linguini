use crate::error::{ConfigError, ConfigResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LinguiniConfig {
    pub project: ProjectConfig,
    pub paths: PathsConfig,
    pub targets: TargetsConfig,
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
}

impl LinguiniConfig {
    pub fn validate(&self) -> ConfigResult<()> {
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
            if ts.out.trim().is_empty() {
                return Err(ConfigError::MissingField("targets.ts.out"));
            }
            if ts.module != "esm" {
                return Err(ConfigError::InvalidString(ts.module.clone()));
            }
        }

        Ok(())
    }
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
    use super::validate_locale_tag;

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
}
