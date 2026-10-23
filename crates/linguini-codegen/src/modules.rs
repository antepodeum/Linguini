//! Versioned, language-independent transport for build-time module configuration.

use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePlan {
    modules: Vec<ModuleConfiguration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleConfiguration {
    id: String,
    #[serde(default = "empty_options")]
    options: Value,
}

fn empty_options() -> Value {
    Value::Object(Default::default())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlan {
    version: u32,
    modules: Vec<ModuleConfiguration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModulePlanError {
    InvalidJson(String),
    UnsupportedVersion(u32),
    DuplicateModule(String),
    InvalidModule(String),
}

impl fmt::Display for ModulePlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(reason) => write!(formatter, "invalid module plan: {reason}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported module plan version {version}")
            }
            Self::DuplicateModule(id) => write!(formatter, "duplicate module `{id}`"),
            Self::InvalidModule(id) => write!(
                formatter,
                "module `{id}` requires a nonempty id and an options object"
            ),
        }
    }
}

impl std::error::Error for ModulePlanError {}

impl ModulePlan {
    pub fn empty() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    /// Validates transport identity and shape. Owning modules validate their own options.
    pub fn parse(source: &str) -> Result<Self, ModulePlanError> {
        let raw: RawPlan = serde_json::from_str(source)
            .map_err(|error| ModulePlanError::InvalidJson(error.to_string()))?;
        if raw.version != 1 {
            return Err(ModulePlanError::UnsupportedVersion(raw.version));
        }
        let mut ids = BTreeSet::new();
        for module in &raw.modules {
            if module.id.is_empty() || !module.options.is_object() {
                return Err(ModulePlanError::InvalidModule(module.id.clone()));
            }
            if !ids.insert(&module.id) {
                return Err(ModulePlanError::DuplicateModule(module.id.clone()));
            }
        }
        Ok(Self {
            modules: raw.modules,
        })
    }

    pub fn modules(&self) -> &[ModuleConfiguration] {
        &self.modules
    }
}

impl ModuleConfiguration {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn options(&self) -> &Value {
        &self.options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_is_versioned_closed_and_independent_of_backend_names() {
        let plan = ModulePlan::parse(
            r#"{"version":1,"modules":[{"id":"future.language","options":{"target":"native"}}]}"#,
        )
        .unwrap();
        assert_eq!(plan.modules()[0].id(), "future.language");
        assert_eq!(plan.modules()[0].options()["target"], "native");
        for invalid in [
            r#"{"version":2,"modules":[]}"#,
            r#"{"version":1,"modules":[],"unknown":true}"#,
            r#"{"version":1,"modules":[{"id":"a"},{"id":"a"}]}"#,
            r#"{"version":1,"modules":[{"id":""}]}"#,
            r#"{"version":1,"modules":[{"id":"a","options":[]}] }"#,
            r#"{"version":1,"modules":[{"id":"a","unknown":1}] }"#,
        ] {
            assert!(ModulePlan::parse(invalid).is_err(), "{invalid}");
        }
        assert_eq!(ModulePlan::empty().modules(), []);
    }
}
