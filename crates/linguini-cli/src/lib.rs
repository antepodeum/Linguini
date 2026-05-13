mod project;

#[cfg(test)]
mod tests;

use clap::{Args, Parser, Subcommand};
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

pub use project::{build_project, check_project, init_project};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Args(clap::Error),
    Config(linguini_config::ConfigError),
    Diagnostics(String),
    Io { path: PathBuf, source: io::Error },
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Args(error) => Display::fmt(error, f),
            Self::Config(error) => Display::fmt(error, f),
            Self::Diagnostics(output) => f.write_str(output),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for CliError {}

impl From<clap::Error> for CliError {
    fn from(error: clap::Error) -> Self {
        Self::Args(error)
    }
}

impl From<linguini_config::ConfigError> for CliError {
    fn from(error: linguini_config::ConfigError) -> Self {
        Self::Config(error)
    }
}

#[derive(Debug, Parser)]
#[command(name = "linguini", about = "Experimental localization toolkit CLI")]
pub struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
enum CliCommand {
    /// Create a Linguini project skeleton
    Init,
    /// Parse configured schema and locale files and report diagnostics
    Check,
    /// Apply analyzer quick fixes such as missing locale files and message stubs
    Fix(FixArgs),
    /// Build the localization project and write configured codegen outputs
    Build,
}

#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub(crate) struct FixArgs {
    /// Apply every available automatic fix
    #[arg(long)]
    pub(crate) all: bool,
    /// Fix ids printed by `linguini check`, for example `missing-messages:shop:ru`
    pub(crate) ids: Vec<String>,
}

pub fn run(
    args: impl IntoIterator<Item = String>,
    current_dir: io::Result<PathBuf>,
) -> CliResult<String> {
    let root = current_dir.map_err(|source| CliError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    let cli = Cli::try_parse_from(std::iter::once("linguini".to_owned()).chain(args))?;

    match cli.command {
        CliCommand::Init => init_project(&root),
        CliCommand::Check => check_project(&root),
        CliCommand::Fix(args) => project::fix_project(&root, &args),
        CliCommand::Build => build_project(&root),
    }
}
