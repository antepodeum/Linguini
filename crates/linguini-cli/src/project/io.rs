use crate::{CliError, CliResult};
use linguini_analyzer::{render_diagnostics_with_color, Diagnostic};
use linguini_config::{parse_config, DEFAULT_CONFIG_FILE};
use std::fs;
use std::io::IsTerminal;
use std::path::Path;

pub fn init_project(root: &Path) -> CliResult<String> {
    let schema_dir = root.join("schema");
    let locale_dir = root.join("locales");
    create_dir_all(&schema_dir)?;
    create_dir_all(&locale_dir)?;

    let config_path = root.join(DEFAULT_CONFIG_FILE);
    if !config_path.exists() {
        write_file(&config_path, default_config())?;
    }

    Ok(format!(
        "created {}\ncreated {}\ncreated {}\n",
        DEFAULT_CONFIG_FILE, "schema", "locales"
    ))
}

pub(crate) fn read_project_config(root: &Path) -> CliResult<linguini_config::LinguiniConfig> {
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
schema = "schema"
locale = "locales"

[targets.ts]
out = "src/generated/linguini"
module = "esm"
declaration = true
"#
}

pub(crate) fn path_for_output(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn create_dir_all(path: &Path) -> CliResult<()> {
    fs::create_dir_all(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn read_file(path: &Path) -> CliResult<String> {
    fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn write_file(path: &Path, contents: &str) -> CliResult<()> {
    fs::write(path, contents).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn render_parse_errors(
    root: &Path,
    path: &Path,
    source: &str,
    note: &str,
    errors: Vec<linguini_syntax::ParseError>,
) -> String {
    let diagnostics: Vec<_> = errors
        .into_iter()
        .map(|error| Diagnostic::error(error.message, error.span).with_note(note))
        .collect();

    render_file_diagnostics(root, path, source, &diagnostics)
}

pub(crate) fn render_file_diagnostics(
    root: &Path,
    path: &Path,
    source: &str,
    diagnostics: &[Diagnostic],
) -> String {
    let relative_path = path_for_output(root, path);
    render_diagnostics_with_color(
        &relative_path,
        source,
        diagnostics,
        std::io::stderr().is_terminal(),
    )
    .unwrap_or_else(|error| format!("failed to render diagnostics for {relative_path}: {error}"))
}
