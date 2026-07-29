mod discovery;
mod error;
mod model;
mod parser;

pub use discovery::{
    discover_locale_files, discover_schema_files, locale_scope_chain, LocaleFile, SchemaFile,
};
pub use error::{ConfigError, ConfigResult};
pub use model::{
    AnalysisConfig, CanonicalMode, CookiePath, LinguiniConfig, LinkMode, LocalePrefixMode,
    LocaleSource, LocaleSwitchPlan, PathsConfig, ProjectConfig, SameSite, SecurePolicy,
    TargetsConfig, TypeScriptTargetConfig, UnusedMessagesConfig, WebConfig, WebCookieConfig,
    WebLinksConfig, WebLocalStorageConfig, WebLocaleConfig, WebRoutesConfig, WebRoutingConfig,
    WebSwitchRouteConfig,
};
pub use parser::parse_config;

pub const DEFAULT_CONFIG_FILE: &str = "linguini.toml";

#[cfg(test)]
mod tests;
