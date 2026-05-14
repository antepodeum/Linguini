mod project;

#[cfg(test)]
mod tests;

use clap::{Args, Parser, Subcommand};
use linguini_format::format_path_source;
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::PathBuf;

pub use project::{build_project, check_project, init_project};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Args(clap::Error),
    Config(linguini_config::ConfigError),
    Diagnostics(String),
    Format(linguini_format::FormatError),
    Io { path: PathBuf, source: io::Error },
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Args(error) => Display::fmt(error, f),
            Self::Config(error) => Display::fmt(error, f),
            Self::Diagnostics(output) => f.write_str(output),
            Self::Format(error) => Display::fmt(error, f),
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
    /// Generate rendered sample data for configured locales and enum variants
    Generate,
    /// Format `.lgs` and `.lgl` files
    Format(FormatArgs),
    /// Start the Linguini language server over stdio
    Lsp,
}

#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub(crate) struct FixArgs {
    /// Apply every available automatic fix
    #[arg(long)]
    pub(crate) all: bool,
    /// Fix ids printed by `linguini check`, for example `missing-messages:shop:ru`
    pub(crate) ids: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub(crate) struct FormatArgs {
    /// Check whether files are already formatted without writing changes
    #[arg(long)]
    pub(crate) check: bool,
    /// Files to format. Defaults to discovered schema and locale files.
    pub(crate) paths: Vec<PathBuf>,
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
        CliCommand::Generate => project::generate_project_data(&root),
        CliCommand::Format(args) => format_project(&root, &args),
        CliCommand::Lsp => {
            linguini_lsp::run_stdio_blocking();
            Ok(String::new())
        }
    }
}

fn format_project(root: &std::path::Path, args: &FormatArgs) -> CliResult<String> {
    let paths = if args.paths.is_empty() {
        let config_path = root.join(linguini_config::DEFAULT_CONFIG_FILE);
        let config_source = fs::read_to_string(&config_path).map_err(|source| CliError::Io {
            path: config_path,
            source,
        })?;
        let config = linguini_config::parse_config(&config_source)?;
        linguini_config::discover_schema_files(root.join(&config.paths.schema))?
            .iter()
            .map(|file| file.path.clone())
            .chain(
                linguini_config::discover_locale_files(root.join(&config.paths.locale))?
                    .iter()
                    .map(|file| file.path.clone()),
            )
            .collect()
    } else {
        args.paths.clone()
    };

    let mut changed = Vec::new();
    for relative_path in paths {
        let path = if relative_path.is_absolute() {
            relative_path
        } else {
            root.join(relative_path)
        };
        let source = fs::read_to_string(&path).map_err(|source| CliError::Io {
            path: path.clone(),
            source,
        })?;
        let formatted = format_path_source(&path, &source).map_err(CliError::Format)?;
        if formatted != source {
            changed.push(path.strip_prefix(root).unwrap_or(&path).display().to_string());
            if !args.check {
                fs::write(&path, formatted).map_err(|source| CliError::Io {
                    path: path.clone(),
                    source,
                })?;
            }
        }
    }

    if args.check && !changed.is_empty() {
        return Err(CliError::Diagnostics(format!(
            "format check failed:\n{}\n",
            changed.join("\n")
        )));
    }

    if changed.is_empty() {
        Ok("format: ok\n".to_owned())
    } else {
        Ok(format!("formatted files:\n{}\n", changed.join("\n")))
    }
}
