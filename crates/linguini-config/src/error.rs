use std::fmt::{self, Display};
use std::path::PathBuf;

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConfigError {
    DuplicateKey(String),
    InvalidArray(String),
    InvalidLocaleTag(String),
    InvalidString(String),
    MissingField(&'static str),
    UnexpectedSection(String),
    UnknownKey { section: String, key: String },
    UnreadableDirectory(PathBuf),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey(key) => write!(f, "duplicate config key `{key}`"),
            Self::InvalidArray(value) => write!(f, "invalid string array `{value}`"),
            Self::InvalidLocaleTag(tag) => write!(f, "invalid locale tag `{tag}`"),
            Self::InvalidString(value) => write!(f, "invalid string value `{value}`"),
            Self::MissingField(field) => write!(f, "missing required config field `{field}`"),
            Self::UnexpectedSection(section) => write!(f, "unexpected config section `{section}`"),
            Self::UnknownKey { section, key } => {
                write!(f, "unknown config key `{section}.{key}`")
            }
            Self::UnreadableDirectory(path) => {
                write!(f, "could not read directory `{}`", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}
