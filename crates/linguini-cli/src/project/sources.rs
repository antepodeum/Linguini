use crate::{CliError, CliResult};
use linguini_analyzer::{DiagnosticSeverity, LocaleCoverageOptions};
use linguini_config::{discover_locale_files, discover_schema_files, LinguiniConfig};
use linguini_syntax::{parse_locale, parse_schema};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::fixes::missing_messages_fix_id;
use super::io::{read_file, render_parse_errors};
use super::util::namespace_display;
use super::{ParsedLocaleSource, ParsedSchemaSource};

pub(crate) fn load_schema_sources(
    root: &Path,
    config: &LinguiniConfig,
) -> CliResult<Vec<ParsedSchemaSource>> {
    let mut parsed = Vec::new();
    for file in discover_schema_files(root.join(&config.paths.schema))? {
        let source = read_file(&file.path)?;
        let ast = parse_schema(&source).map_err(|errors| {
            CliError::Diagnostics(render_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                errors,
            ))
        })?;
        parsed.push(ParsedSchemaSource { file, source, ast });
    }
    Ok(parsed)
}

pub(crate) fn load_locale_sources(
    root: &Path,
    config: &LinguiniConfig,
) -> CliResult<Vec<ParsedLocaleSource>> {
    let mut parsed = Vec::new();
    for file in discover_locale_files(root.join(&config.paths.locale))? {
        let source = read_file(&file.path)?;
        let ast = parse_locale(&source).map_err(|errors| {
            CliError::Diagnostics(render_parse_errors(
                root,
                &file.path,
                &source,
                "locale syntax error",
                errors,
            ))
        })?;
        parsed.push(ParsedLocaleSource { file, source, ast });
    }
    Ok(parsed)
}

pub(crate) fn locale_index<'a>(
    locale_files: &'a [ParsedLocaleSource],
) -> BTreeMap<(String, String), &'a ParsedLocaleSource> {
    locale_files
        .iter()
        .map(|file| ((file.file.namespace.clone(), file.file.locale.clone()), file))
        .collect()
}

pub(crate) fn expected_locale_path(
    root: &Path,
    config: &LinguiniConfig,
    namespace: &str,
    locale: &str,
) -> PathBuf {
    let mut path = root.join(&config.paths.locale);
    for part in namespace.split('.').filter(|part| !part.is_empty()) {
        path.push(part);
    }
    path.join(format!("{locale}.lgl"))
}

pub(crate) fn coverage_options(
    config: &LinguiniConfig,
    namespace: &str,
    locale: &str,
) -> LocaleCoverageOptions {
    LocaleCoverageOptions {
        missing_message_severity: if locale == config.project.default_locale {
            DiagnosticSeverity::Error
        } else {
            DiagnosticSeverity::Warning
        },
        subject: format!(
            "locale `{locale}` for schema namespace `{}`",
            namespace_display(namespace)
        ),
        quick_fix_id: Some(missing_messages_fix_id(namespace, locale)),
    }
}
