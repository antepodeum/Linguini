use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebCodegenError {
    InvalidModuleOptions {
        reason: String,
    },
    DuplicateWebSource {
        source: String,
    },
    WebLocaleSwitchPlanMismatch {
        source: &'static str,
        expected: bool,
        actual: bool,
    },
}

impl fmt::Display for WebCodegenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModuleOptions { reason } => write!(formatter, "invalid web module options: {reason}"),
            Self::DuplicateWebSource { source } => {
                write!(formatter, "generated web features contain duplicate locale source `{source}`")
            }
            Self::WebLocaleSwitchPlanMismatch {
                source,
                expected,
                actual,
            } => write!(
                formatter,
                "generated locale switch plan for source `{source}` is {actual}, expected {expected}"
            ),
        }
    }
}

impl std::error::Error for WebCodegenError {}
