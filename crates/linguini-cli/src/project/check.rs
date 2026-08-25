use crate::{CliError, CliResult, DiagnosticFormat};
use linguini_analyzer::{
    analyze_locale_coverage_with_options, analyze_unused_messages, schema_public_messages,
    ApplicationUsage, Diagnostic, DiagnosticCategory, DiagnosticSeverity, PublicMessage, QuickFix,
};
use linguini_config::{
    discover_application_source_files, discover_locale_files, discover_schema_files, LinguiniConfig,
};
use linguini_syntax::{parse_locale_with_recovery_in, parse_schema_with_recovery_in, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::diagnostics::ProjectDiagnostics;
use super::fixes::missing_locale_fix_id;
use super::io::{path_for_output, read_file, read_project_config, render_file_diagnostics};
use super::sources::{
    coverage_options, expected_locale_path, locale_index, project_source_id,
    schema_project_diagnostics, SourceKind,
};
use super::util::{namespace_display, pluralize};
use super::{ParsedLocaleSource, ParsedSchemaSource};

pub fn check_project(root: &Path) -> CliResult<String> {
    check_project_with_options(root, false, DiagnosticFormat::Human)
}

pub(crate) fn check_project_with_options(
    root: &Path,
    deny_warnings: bool,
    format: DiagnosticFormat,
) -> CliResult<String> {
    let config = read_project_config(root)?;
    let schema_files = discover_schema_files(root.join(&config.paths.schema))?;
    let locale_files = discover_locale_files(root.join(&config.paths.locale))?;
    let mut parsed_schema_files = Vec::new();
    let mut parsed_locale_files = Vec::new();
    let mut invalid_locale_keys = BTreeSet::new();
    let mut diagnostics = ProjectDiagnostics::default();

    let mut output = String::new();
    output.push_str("schema files:\n");
    for (index, file) in schema_files.iter().enumerate() {
        output.push_str(&format!(
            "- {} [{}]\n",
            path_for_output(root, &file.path),
            file.namespace
        ));
        let source = read_file(&file.path)?;
        let source_id = project_source_id(SourceKind::Schema, index)?;
        diagnostics.register_source(source_id, root, &file.path, &source);
        let parsed = parse_schema_with_recovery_in(&source, source_id);
        let has_syntax_errors = !parsed.errors.is_empty();
        if has_syntax_errors {
            diagnostics.push_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                parsed.errors,
            );
        }
        if !has_syntax_errors {
            let Some(ast) = parsed.ast else {
                continue;
            };
            parsed_schema_files.push(ParsedSchemaSource {
                file: file.clone(),
                source,
                ast,
            });
        }
    }

    output.push_str("locale files:\n");
    for (index, file) in locale_files.iter().enumerate() {
        output.push_str(&format!(
            "- {} [{}:{}]\n",
            path_for_output(root, &file.path),
            file.locale,
            file.namespace
        ));
        let source = read_file(&file.path)?;
        let source_id = project_source_id(SourceKind::Locale, index)?;
        diagnostics.register_source(source_id, root, &file.path, &source);
        let parsed = parse_locale_with_recovery_in(&source, source_id);
        let has_syntax_errors = !parsed.errors.is_empty();
        if has_syntax_errors {
            invalid_locale_keys.insert((file.namespace.clone(), file.locale.clone()));
            diagnostics.push_parse_errors(
                root,
                &file.path,
                &source,
                "locale syntax error",
                parsed.errors,
            );
        }
        if !has_syntax_errors {
            let Some(ast) = parsed.ast else {
                continue;
            };
            parsed_locale_files.push(ParsedLocaleSource {
                file: file.clone(),
                source,
                ast,
            });
        }
    }

    let schema_semantics = schema_project_diagnostics(&parsed_schema_files);
    for schema_file in &parsed_schema_files {
        let file_diagnostics = schema_semantics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .source_span
                    .is_some_and(|span| span.source == schema_file.ast.span().source)
            })
            .cloned()
            .collect::<Vec<_>>();
        diagnostics.push(
            root,
            &schema_file.file.path,
            &schema_file.source,
            &file_diagnostics,
        );
    }

    collect_unused_message_diagnostics(root, &config, &parsed_schema_files, &mut diagnostics)?;

    let schema_namespaces = schema_files
        .iter()
        .map(|file| file.namespace.clone())
        .collect::<BTreeSet<_>>();
    collect_project_coverage_diagnostics(
        root,
        &config,
        &parsed_schema_files,
        &parsed_locale_files,
        &schema_namespaces,
        &invalid_locale_keys,
        &mut diagnostics,
    )?;

    let has_errors = diagnostics.has_errors();
    let has_non_errors = diagnostics.has_non_errors();
    let failed = has_errors || (deny_warnings && has_non_errors);
    if format == DiagnosticFormat::Human {
        if has_errors {
            return Err(CliError::Diagnostics(diagnostics.render_human(true)));
        }
        if deny_warnings && has_non_errors {
            return Err(CliError::Diagnostics(diagnostics.render_human(false)));
        }
        if has_non_errors {
            output.push_str(&diagnostics.render_human(false));
        }
        return Ok(output);
    }

    let output = diagnostics
        .render_machine(format, failed)
        .map_err(|error| {
            CliError::Diagnostics(format!("failed to serialize diagnostics: {error}\n"))
        })?;
    if failed {
        Err(CliError::MachineDiagnostics(output))
    } else {
        Ok(output)
    }
}

fn collect_unused_message_diagnostics(
    root: &Path,
    config: &LinguiniConfig,
    schema_files: &[ParsedSchemaSource],
    output: &mut ProjectDiagnostics,
) -> CliResult<()> {
    let Some(options) = &config.analysis.unused_messages else {
        return Ok(());
    };

    let mut excluded = options.exclude.clone();
    if let Some(target) = &config.targets.ts {
        if !excluded.contains(&target.out) {
            excluded.push(target.out.clone());
        }
    }
    let application_files = discover_application_source_files(root, &options.sources, &excluded)?;
    let mut usage = ApplicationUsage::default();
    for path in application_files {
        usage.extend_source(&read_file(&path)?);
    }

    for schema_file in schema_files {
        let messages = schema_public_messages(&schema_file.ast)
            .into_iter()
            .map(|message| {
                let name = if schema_file.file.namespace.is_empty() {
                    message.name
                } else {
                    format!("{}.{}", schema_file.file.namespace, message.name)
                };
                PublicMessage::new(name, message.span)
            })
            .collect::<Vec<_>>();
        let diagnostics = analyze_unused_messages(&messages, &usage, &options.ignore);
        output.push(
            root,
            &schema_file.file.path,
            &schema_file.source,
            &diagnostics,
        );
    }
    Ok(())
}

fn collect_project_coverage_diagnostics(
    root: &Path,
    config: &LinguiniConfig,
    schema_files: &[ParsedSchemaSource],
    locale_files: &[ParsedLocaleSource],
    schema_namespaces: &BTreeSet<String>,
    invalid_locale_keys: &BTreeSet<(String, String)>,
    output: &mut ProjectDiagnostics,
) -> CliResult<()> {
    let locale_index = locale_index(locale_files)?;

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
                None if invalid_locale_keys
                    .contains(&(schema_file.file.namespace.clone(), locale.clone())) => {}
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
            output,
        );
        emit_missing_locale_files(
            root,
            config,
            schema_file,
            &missing_secondary_locales,
            DiagnosticSeverity::Warning,
            output,
        );
    }

    for locale_files in locale_files_without_schema_namespace(locale_files, schema_namespaces) {
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
        .with_code("linguini.unknown_locale_namespace")
        .with_category(DiagnosticCategory::Project)
        .without_source()
        .with_note(format!(
            "move these files under locales/<schema-namespace>/<locale>.lgl: {affected}"
        ));
        output.push(root, &primary.file.path, &primary.source, &[diagnostic]);
    }

    Ok(())
}

fn emit_missing_locale_files(
    root: &Path,
    config: &LinguiniConfig,
    schema_file: &ParsedSchemaSource,
    locales: &[String],
    severity: DiagnosticSeverity,
    output: &mut ProjectDiagnostics,
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
                .with_code("linguini.unknown_locale_namespace")
                .with_category(DiagnosticCategory::Project)
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
    .with_code("linguini.missing_locale_file")
    .with_category(DiagnosticCategory::Project)
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
