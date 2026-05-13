use clap::{Parser, Subcommand};
use linguini_analyzer::{render_diagnostics, Diagnostic};
use linguini_config::{discover_locale_files, discover_schema_files, parse_config, DEFAULT_CONFIG_FILE};
use linguini_syntax::{parse_locale_with_recovery, parse_schema_with_recovery};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Subcommand)]
enum CliCommand {
    /// Create a Linguini project skeleton
    Init,
    /// Parse configured schema and locale files and report diagnostics
    Check,
    /// Validate the project using built-in CLDR rules compiled into this binary
    Build,
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
        CliCommand::Build => build_project(&root),
    }
}

pub fn build_project(root: &Path) -> CliResult<String> {
    let check_output = check_project(root)?;

    Ok(format!("{check_output}build: ok\n"))
}

pub fn init_project(root: &Path) -> CliResult<String> {
    let schema_dir = root.join("linguini/schema");
    let locale_dir = root.join("linguini/locale");
    create_dir_all(&schema_dir)?;
    create_dir_all(&locale_dir)?;

    let config_path = root.join(DEFAULT_CONFIG_FILE);
    if !config_path.exists() {
        write_file(&config_path, default_config())?;
    }

    Ok(format!(
        "created {}\ncreated {}\ncreated {}\n",
        DEFAULT_CONFIG_FILE, "linguini/schema", "linguini/locale"
    ))
}

pub fn check_project(root: &Path) -> CliResult<String> {
    let config = read_project_config(root)?;
    let schema_root = root.join(&config.paths.schema);
    let locale_root = root.join(&config.paths.locale);
    let schema_files = discover_schema_files(schema_root)?;
    let locale_files = discover_locale_files(locale_root)?;
    let mut diagnostic_output = String::new();

    let mut output = String::new();
    output.push_str("schema files:\n");
    for file in &schema_files {
        output.push_str(&format!(
            "- {} [{}]\n",
            path_for_output(root, &file.path),
            file.namespace
        ));
        let source = read_file(&file.path)?;
        let parsed = parse_schema_with_recovery(&source);
        if !parsed.errors.is_empty() {
            diagnostic_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                parsed.errors,
            ));
        }
    }

    output.push_str("locale files:\n");
    for file in &locale_files {
        output.push_str(&format!(
            "- {} [{}:{}]\n",
            path_for_output(root, &file.path),
            file.locale,
            file.namespace
        ));
        let source = read_file(&file.path)?;
        let parsed = parse_locale_with_recovery(&source);
        if !parsed.errors.is_empty() {
            diagnostic_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "locale syntax error",
                parsed.errors,
            ));
        }
    }

    if !diagnostic_output.is_empty() {
        return Err(CliError::Diagnostics(diagnostic_output));
    }

    Ok(output)
}

fn read_project_config(root: &Path) -> CliResult<linguini_config::LinguiniConfig> {
    let config_path = root.join(DEFAULT_CONFIG_FILE);
    let source = read_file(&config_path)?;
    Ok(parse_config(&source)?)
}

fn default_config() -> &'static str {
    r#"[project]
name = "linguini-app"
default_locale = "en"
locales = ["en"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"
"#
}

fn path_for_output(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn create_dir_all(path: &Path) -> CliResult<()> {
    fs::create_dir_all(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_file(path: &Path) -> CliResult<String> {
    fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, contents: &str) -> CliResult<()> {
    fs::write(path, contents).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn render_parse_errors(
    root: &Path,
    path: &Path,
    source: &str,
    note: &str,
    errors: Vec<linguini_syntax::ParseError>,
) -> String {
    let relative_path = path_for_output(root, path);
    let diagnostics: Vec<_> = errors
        .into_iter()
        .map(|error| Diagnostic::error(error.message, error.span).with_note(note))
        .collect();

    render_diagnostics(&relative_path, source, &diagnostics).unwrap_or_else(|error| {
        format!("failed to render diagnostics for {relative_path}: {error}")
    })
}

#[cfg(test)]
mod tests {
    use super::{build_project, check_project, init_project, Cli};
    use clap::CommandFactory;
    use linguini_test_support::temp_project_dir;
    use std::fs;

    #[test]
    fn cli_argument_parser_is_clap_backed() {
        let command = Cli::command();
        let subcommands: Vec<_> = command
            .get_subcommands()
            .map(|command| command.get_name().to_owned())
            .collect();

        assert!(subcommands.contains(&"init".to_owned()));
        assert!(subcommands.contains(&"check".to_owned()));
        assert!(subcommands.contains(&"build".to_owned()));
        assert!(!subcommands.contains(&"cldr".to_owned()));
    }

    #[test]
    fn init_creates_valid_project() {
        let project = temp_project_dir("init_creates_valid_project");

        init_project(project.path()).expect("init project");

        assert!(project.path().join("linguini.toml").exists());
        assert!(project.path().join("linguini/schema").is_dir());
        assert!(project.path().join("linguini/locale").is_dir());
        let config = fs::read_to_string(project.path().join("linguini.toml")).expect("config");
        assert!(!config.contains("cache"));
    }

    #[test]
    fn check_lists_discovered_files() {
        let project = temp_project_dir("check_lists_discovered_files");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("linguini/schema/shop");
        let locale_dir = project.path().join("linguini/locale/shop/delivery");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::create_dir_all(&locale_dir).expect("locale dir");
        fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
        fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

        let output = check_project(project.path()).expect("check project");

        assert!(output.contains("linguini/schema/shop/delivery.lqs [shop.delivery]"));
        assert!(output.contains("linguini/locale/shop/delivery/en.lgl [en:shop.delivery]"));
    }

    #[test]
    fn check_reports_schema_syntax_diagnostics() {
        let project = temp_project_dir("check_reports_schema_syntax_diagnostics");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("linguini/schema/shop");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::write(schema_dir.join("broken.lqs"), "delivery(fruit: Fruit\n").expect("schema file");

        let error = check_project(project.path()).expect_err("check fails");
        let rendered = error.to_string();

        assert!(rendered.contains("Error:"));
        assert!(rendered.contains("linguini/schema/shop/broken.lqs"));
        assert!(rendered.contains("schema syntax error"));
    }

    #[test]
    fn build_uses_built_in_cldr_and_does_not_require_cache() {
        let project = temp_project_dir("build_uses_built_in_cldr");
        init_project(project.path()).expect("init project");

        let output = build_project(project.path()).expect("build project");

        assert!(output.contains("schema files:"));
        assert!(output.contains("locale files:"));
        assert!(output.contains("build: ok"));
        assert!(!project.path().join(".linguini/cache").exists());
    }
}
