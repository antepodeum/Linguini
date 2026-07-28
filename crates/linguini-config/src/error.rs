use std::fmt::{self, Display};
use std::io::ErrorKind;
use std::path::PathBuf;

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConfigError {
    DiscoveryLimit {
        root: PathBuf,
        limit: usize,
    },
    DuplicateKey(String),
    InvalidArray(String),
    InvalidLocaleTag(String),
    InvalidPath {
        field: &'static str,
        value: String,
        reason: &'static str,
    },
    InvalidString(String),
    Io {
        path: PathBuf,
        kind: ErrorKind,
        message: String,
    },
    MissingField(&'static str),
    NonUtf8Path(PathBuf),
    PathOutsideRoot {
        root: PathBuf,
        path: PathBuf,
    },
    RemovedField {
        field: &'static str,
        replacement: &'static str,
    },
    Toml {
        message: String,
        span: Option<(usize, usize)>,
    },
    UnexpectedSection(String),
    UnknownKey {
        section: String,
        key: String,
    },
    UnreadableDirectory(PathBuf),
    UnsupportedSymlink(PathBuf),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscoveryLimit { root, limit } => write!(
                f,
                "source discovery under `{}` exceeded the limit of {limit} entries",
                root.display()
            ),
            Self::DuplicateKey(key) => write!(f, "duplicate config key `{key}`"),
            Self::InvalidArray(value) => write!(f, "invalid string array `{value}`"),
            Self::InvalidLocaleTag(tag) => write!(f, "invalid locale tag `{tag}`"),
            Self::InvalidPath {
                field,
                value,
                reason,
            } => write!(f, "invalid path for `{field}`: `{value}` ({reason})"),
            Self::InvalidString(value) => write!(f, "invalid string value `{value}`"),
            Self::Io {
                path,
                kind,
                message,
            } => write!(f, "{} ({kind:?}): {message}", path.display()),
            Self::MissingField(field) => write!(f, "missing required config field `{field}`"),
            Self::NonUtf8Path(path) => {
                write!(
                    f,
                    "path contains non-UTF-8 components: `{}`",
                    path.display()
                )
            }
            Self::PathOutsideRoot { root, path } => write!(
                f,
                "path `{}` is outside configured root `{}`",
                path.display(),
                root.display()
            ),
            Self::RemovedField { field, replacement } => {
                write!(f, "config field `{field}` was removed; {replacement}")
            }
            Self::Toml { message, span } => {
                if let Some((start, end)) = span {
                    write!(f, "invalid TOML at bytes {start}..{end}: {message}")
                } else {
                    write!(f, "invalid TOML: {message}")
                }
            }
            Self::UnexpectedSection(section) => write!(f, "unexpected config section `{section}`"),
            Self::UnknownKey { section, key } => {
                write!(f, "unknown config key `{section}.{key}`")
            }
            Self::UnreadableDirectory(path) => {
                write!(f, "could not read directory `{}`", path.display())
            }
            Self::UnsupportedSymlink(path) => write!(
                f,
                "symbolic links are not supported during source discovery: `{}`",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ConfigError {}
