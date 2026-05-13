use crate::{CliError, CliResult};
use linguini_analyzer::{
    analyze_locale_coverage_with_options, analyze_locale_file, Diagnostic, DiagnosticSeverity,
    QuickFix,
};
use linguini_config::{discover_locale_files, discover_schema_files, LinguiniConfig};
use linguini_syntax::{parse_locale_with_recovery, parse_schema_with_recovery, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::fixes::missing_locale_fix_id;
use super::io::{
    path_for_output, read_file, read_project_config, render_file_diagnostics, render_parse_errors,
};
use super::sources::{coverage_options, expected_locale_path, locale_index};
use super::util::{namespace_display, pluralize};
use super::{ParsedLocaleSource, ParsedSchemaSource};

#[derive(Debug, Default)]
struct ProjectDiagnosticOutput {
    errors: String,
    warnings: String,
}

impl ProjectDiagnosticOutput {
    fn push(&mut self, root: &Path, path: &Path, source: &str, diagnostics: &[Diagnostic]) {
        let errors = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .cloned()
            .collect::<Vec<_>>();
        let warnings = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error)
            .cloned()
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            self.errors
                .push_str(&render_file_diagnostics(root, path, source, &errors));
        }
        if !warnings.is_empty() {
            self.warnings
                .push_str(&render_file_diagnostics(root, path, source, &warnings));
        }
    }
}

pub fn check_project(root: &Path) -> CliResult<String> {
    let config = read_project_config(root)?;
    let schema_files = discover_schema_files(root.join(&config.paths.schema))?;
    let locale_files = discover_locale_files(root.join(&config.paths.locale))?;
    let mut parsed_schema_files = Vec::new();
    let mut parsed_locale_files = Vec::new();
    let mut error_output = String::new();
    let mut warning_output = String::new();

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
            error_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                parsed.errors,
            ));
        }
        if let Some(ast) = parsed.ast {
            parsed_schema_files.push(ParsedSchemaSource {
                file: file.clone(),
                source,
                ast,
            });
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
            error_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "locale syntax error",
                parsed.errors,
            ));
        } else if let Some(locale) = parsed.ast.as_ref() {
            let diagnostics = analyze_locale_file(locale);
            if !diagnostics.is_empty() {
                warning_output.push_str(&render_file_diagnostics(
                    root,
                    &file.path,
                    &source,
                    &diagnostics,
                ));
            }
        }
        if let Some(ast) = parsed.ast {
            parsed_locale_files.push(ParsedLocaleSource {
                file: file.clone(),
                source,
                ast,
            });
        }
    }

    if error_output.is_empty() {
        let project_diagnostics = render_project_coverage_diagnostics(
            root,
            &config,
            &parsed_schema_files,
            &parsed_locale_files,
        )?;
        error_output.push_str(&project_diagnostics.errors);
        warning_output.push_str(&project_diagnostics.warnings);
    }

    if !error_output.is_empty() {
        return Err(CliError::Diagnostics(error_output));
    }
    if !warning_output.is_empty() {
        output.push_str(&warning_output);
    }
    Ok(output)
}

fn render_project_coverage_diagnostics(
    root: &Path,
    config: &LinguiniConfig,
    schema_files: &[ParsedSchemaSource],
    locale_files: &[ParsedLocaleSource],
) -> CliResult<ProjectDiagnosticOutput> {
    let schema_namespaces: BTreeSet<_> = schema_files
        .iter()
        .map(|schema| schema.file.namespace.clone())
        .collect();
    let locale_index = locale_index(locale_files);
    let mut output = ProjectDiagnosticOutput::default();

    for schema_file in schema_files {
        let mut missing_default_locale = Vec::new();
        let mut missing_secondary_locales = Vec::new();

        for locale in &config.project.locales {
            match locale_index.get(&(schema_file.file.namespace.clone(), locale.clone())) {
                Some(locale_file) => {
                    let diagnostics = analyze_locale_coverage_with_options(
                        &schema_file.ast,
                        &locale_file.ast,
                        coverage_options(config, &schema_file.file.namespace, locale),
                    );
                    output.push(
                        root,
                        &locale_file.file.path,
                        &locale_file.source,
                        &diagnostics,
                    );
                }
                None if locale == &config.project.default_locale => {
                    missing_default_locale.push(locale.clone());
                }
                None => missing_secondary_locales.push(locale.clone()),
            }
        }

        emit_missing_locale_files(
            root,
            config,
            schema_file,
            &missing_default_locale,
            DiagnosticSeverity::Error,
            &mut output,
        );
        emit_missing_locale_files(
            root,
            config,
            schema_file,
            &missing_secondary_locales,
            DiagnosticSeverity::Warning,
            &mut output,
        );
    }

    for locale_files in locale_files_without_schema_namespace(locale_files, &schema_namespaces) {
        let primary = &locale_files[0];
        let affected = locale_files
            .iter()
            .map(|file| path_for_output(root, &file.file.path))
            .collect::<Vec<_>>()
            .join(", ");
        let diagnostic = Diagnostic::error(
            format!(
                "locale namespace `{}` has no matching schema namespace",
                namespace_display(&primary.file.namespace)
            ),
            Span::new(0, 0),
        )
        .without_source()
        .with_note(format!(
            "move these files under locales/<schema-namespace>/<locale>.lgl: {affected}"
        ));
        output.push(root, &primary.file.path, &primary.source, &[diagnostic]);
    }

    Ok(output)
}

fn emit_missing_locale_files(
    root: &Path,
    config: &LinguiniConfig,
    schema_file: &ParsedSchemaSource,
    locales: &[String],
    severity: DiagnosticSeverity,
    output: &mut ProjectDiagnosticOutput,
) {
    if locales.is_empty() {
        return;
    }

    let diagnostic = missing_locale_files_diagnostic(
        config,
        &schema_file.file.namespace,
        locales,
        severity,
        root,
    );
    output.push(
        root,
        &schema_file.file.path,
        &schema_file.source,
        &[diagnostic],
    );
}

pub(crate) fn reject_locale_files_without_schema_namespace(
    root: &Path,
    schema_namespaces: &BTreeSet<String>,
    locale_files: &[ParsedLocaleSource],
) -> CliResult<()> {
    for locale_file in locale_files {
        if !schema_namespaces.contains(&locale_file.file.namespace) {
            return Err(CliError::Diagnostics(render_file_diagnostics(
                root,
                &locale_file.file.path,
                &locale_file.source,
                &[Diagnostic::error(
                    format!(
                        "locale namespace `{}` has no matching schema namespace",
                        namespace_display(&locale_file.file.namespace)
                    ),
                    Span::new(0, 0),
                )
                .without_source()
                .with_note("move this file under locales/<schema-namespace>/<locale>.lgl")],
            )));
        }
    }
    Ok(())
}

fn missing_locale_files_diagnostic(
    config: &LinguiniConfig,
    namespace: &str,
    locales: &[String],
    severity: DiagnosticSeverity,
    root: &Path,
) -> Diagnostic {
    let expected_paths = locales
        .iter()
        .map(|locale| path_for_output(root, &expected_locale_path(root, config, namespace, locale)))
        .collect::<Vec<_>>();
    let message = format!(
        "{} locale {} missing for schema namespace `{}`: {}",
        if severity == DiagnosticSeverity::Error {
            "required"
        } else {
            "secondary"
        },
        pluralize(locales.len(), "file is", "files are"),
        namespace_display(namespace),
        locales
            .iter()
            .map(|locale| format!("`{locale}`"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    let note = if expected_paths.len() == 1 {
        format!("expected path: {}", expected_paths[0])
    } else {
        format!("expected paths: {}", expected_paths.join(", "))
    };

    let mut diagnostic = match severity {
        DiagnosticSeverity::Error => Diagnostic::error(message, Span::new(0, 0)),
        DiagnosticSeverity::Warning => Diagnostic::warning(message, Span::new(0, 0)),
        DiagnosticSeverity::Advice => Diagnostic::advice(message, Span::new(0, 0)),
    }
    .without_source()
    .with_note(note);

    for locale in locales {
        diagnostic = diagnostic.with_quick_fix(QuickFix::command(
            missing_locale_fix_id(namespace, locale),
            format!("create locale file for `{locale}`"),
        ));
    }
    diagnostic
}

fn locale_files_without_schema_namespace<'a>(
    locale_files: &'a [ParsedLocaleSource],
    schema_namespaces: &BTreeSet<String>,
) -> Vec<Vec<&'a ParsedLocaleSource>> {
    let mut grouped: BTreeMap<&str, Vec<&ParsedLocaleSource>> = BTreeMap::new();
    for locale_file in locale_files {
        if !schema_namespaces.contains(&locale_file.file.namespace) {
            grouped
                .entry(locale_file.file.namespace.as_str())
                .or_default()
                .push(locale_file);
        }
    }
    grouped.into_values().collect()
}
