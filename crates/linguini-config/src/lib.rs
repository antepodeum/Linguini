mod discovery;
mod error;
mod model;
mod parser;

pub use discovery::{
    discover_locale_files, discover_schema_files, locale_scope_chain, LocaleFile, SchemaFile,
};
pub use error::{ConfigError, ConfigResult};
pub use model::{
    LinguiniConfig, PathsConfig, ProjectConfig, TargetsConfig, TypeScriptTargetConfig, WebConfig,
};
pub use parser::parse_config;

pub const DEFAULT_CONFIG_FILE: &str = "linguini.toml";

#[cfg(test)]
mod tests;
