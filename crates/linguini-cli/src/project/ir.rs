//! Validated project IR shared by native build and sample generation.

use super::check::reject_locale_files_without_schema_namespace;
use super::io::{path_for_output, render_file_diagnostics};
use super::sources::{
    coverage_options, expected_locale_path, load_locale_sources, load_schema_sources, locale_index,
    schema_project_diagnostics,
};
use super::{ParsedLocaleSource, ParsedSchemaSource};
use crate::{CliError, CliResult};
use linguini_analyzer::DiagnosticSeverity;
use linguini_codegen::ProjectLocalePlan;
use linguini_config::LinguiniConfig;
use linguini_ir::{
    ensure_no_unresolved_references, lower_locale, lower_schema, qualify_module, IrModule,
    IrModuleBuilder, IrSymbolKind, LocaleIr, SchemaIr,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Keeps parsed source identities beside the validated, fallback-composed declarations.
pub(super) struct ProjectIr {
    pub schema_files: Vec<ParsedSchemaSource>,
    pub locale_files: Vec<ParsedLocaleSource>,
    pub schema: SchemaIr,
    pub locales: Vec<(String, LocaleIr)>,
}

pub(super) fn load_project_ir(root: &Path, config: &LinguiniConfig) -> CliResult<ProjectIr> {
    let schema_files = load_schema_sources(root, config)?;
    let namespaces = schema_files
        .iter()
        .map(|source| source.file.namespace.clone())
        .collect();
    ensure_schema_project_valid(root, &schema_files)?;
    let schema = merge_schema_ir(&schema_files)?;
    let locale_files = load_locale_sources(root, config)?;
    let index = locale_index(&locale_files)?;
    reject_locale_files_without_schema_namespace(root, &namespaces, &locale_files)?;
    let plan = ProjectLocalePlan::new(config.project().locales().iter().map(String::as_str));
    let mut locales = Vec::new();
    for locale in config.project().locales() {
        let module = build_locale_ir(root, config, &schema_files, &index, locale, &plan)?;
        ensure_locale_ir_resolves(&schema, &module, locale)?;
        locales.push((locale.clone(), module));
    }
    Ok(ProjectIr {
        schema_files,
        locale_files,
        schema,
        locales,
    })
}

fn build_locale_ir(
    root: &Path,
    config: &LinguiniConfig,
    schema_files: &[ParsedSchemaSource],
    locale_index: &BTreeMap<(String, String), &ParsedLocaleSource>,
    locale: &str,
    plan: &ProjectLocalePlan<'_>,
) -> CliResult<LocaleIr> {
    let mut locale_ir = IrModule::default();
    let fallbacks = project_locale_fallbacks(plan, locale, config.project().default_locale());

    for schema_file in schema_files {
        let namespace = &schema_file.file.namespace;
        let locale_key = (namespace.clone(), locale.to_owned());
        let locale_file = locale_index.get(&locale_key);

        if locale == config.project().default_locale() && locale_file.is_none() {
            let path = expected_locale_path(root, config, namespace, locale);
            return Err(CliError::Diagnostics(format!(
                "required locale file is missing for schema namespace `{namespace}`: `{locale}`\nexpected path: {}\n",
                path_for_output(root, &path)
            )));
        }

        if let Some(locale_file) = locale_file {
            ensure_locale_has_required_messages(root, config, schema_file, locale_file, locale)?;
            merge_module(
                &mut locale_ir,
                namespaced_module(lower_locale(&locale_file.ast), namespace),
            )?;
        }

        for fallback_locale in &fallbacks {
            let fallback_key = (namespace.clone(), fallback_locale.to_owned());
            if let Some(default_locale_file) = locale_index.get(&fallback_key) {
                merge_module_fallback(
                    &mut locale_ir,
                    namespaced_module(lower_locale(&default_locale_file.ast), namespace),
                )?;
            }
        }
    }

    LocaleIr::try_from_module(locale_ir)
        .map_err(|error| CliError::Diagnostics(format!("invalid locale IR: {error}\n")))
}

fn project_locale_fallbacks(
    plan: &ProjectLocalePlan<'_>,
    locale: &str,
    default_locale: &str,
) -> Vec<String> {
    plan.fallback_chain(locale, Some(default_locale))
        .into_iter()
        .filter(|fallback| !fallback.eq_ignore_ascii_case(locale))
        .collect()
}

fn ensure_locale_has_required_messages(
    root: &Path,
    config: &LinguiniConfig,
    schema_file: &ParsedSchemaSource,
    locale_file: &ParsedLocaleSource,
    locale: &str,
) -> CliResult<()> {
    let diagnostics = linguini_schema::SchemaDatabase::build(&schema_file.ast)
        .analyze_locale_with_options(
            &locale_file.ast,
            coverage_options(config, &schema_file.file.namespace, locale),
        );
    let blocking = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .cloned()
        .collect::<Vec<_>>();
    if blocking.is_empty() {
        return Ok(());
    }

    Err(CliError::Diagnostics(render_file_diagnostics(
        root,
        &locale_file.file.path,
        &locale_file.source,
        &blocking,
    )))
}

fn ensure_schema_project_valid(root: &Path, schema_files: &[ParsedSchemaSource]) -> CliResult<()> {
    let diagnostics = schema_project_diagnostics(schema_files)
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        return Ok(());
    }

    let mut rendered = String::new();
    for source in schema_files {
        let source_diagnostics = diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .source_span
                    .is_some_and(|span| span.source == source.ast.span().source)
            })
            .cloned()
            .collect::<Vec<_>>();
        if !source_diagnostics.is_empty() {
            rendered.push_str(&render_file_diagnostics(
                root,
                &source.file.path,
                &source.source,
                &source_diagnostics,
            ));
        }
    }

    Err(CliError::Diagnostics(rendered))
}

fn ensure_locale_ir_resolves(
    schema: &IrModule,
    locale_ir: &IrModule,
    locale: &str,
) -> CliResult<()> {
    if let Err(errors) = ensure_no_unresolved_references(schema, locale_ir) {
        let rendered = errors
            .into_iter()
            .map(|error| format!("{locale}: {}", error.message))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(CliError::Diagnostics(format!("{rendered}\n")));
    }
    Ok(())
}

fn merge_schema_ir(schema_files: &[ParsedSchemaSource]) -> CliResult<SchemaIr> {
    let mut schema = IrModule::default();
    for file in schema_files {
        merge_module(
            &mut schema,
            namespaced_module(lower_schema(&file.ast), &file.file.namespace),
        )?;
    }
    SchemaIr::try_from_module(schema)
        .map_err(|error| CliError::Diagnostics(format!("invalid schema IR: {error}\n")))
}

pub(super) fn namespaced_module(mut module: IrModule, namespace: &str) -> IrModule {
    qualify_module(&mut module, namespace);
    module
}

pub(super) fn merge_module(target: &mut IrModule, source: IrModule) -> CliResult<()> {
    target.try_append(source).map_err(|conflict| {
        CliError::Diagnostics(format!("duplicate project symbol: {conflict}\n"))
    })
}

pub(super) fn merge_module_fallback(target: &mut IrModule, source: IrModule) -> CliResult<()> {
    // Fallback composition inserts only symbols the target lacks outright,
    // including ancestor names that a non-group declaration already owns.
    // Symbols queued earlier from this same source participate in both guards
    // through `inserted`, mirroring sequential insertion into `target`.
    let mut addition = IrModuleBuilder::new();
    let mut inserted = BTreeSet::new();

    macro_rules! insert_missing {
        ($field:ident, $kind:expr, $push:ident) => {
            for item in source.$field() {
                let pending_ancestor_conflict = item
                    .name
                    .rsplit_once('.')
                    .map(|(prefix, _)| prefix)
                    .map(|ancestor| {
                        walk_ancestors(ancestor).any(|candidate| {
                            inserted.iter().any(|(kind, name)| {
                                *kind != IrSymbolKind::Group && name == candidate
                            })
                        })
                    })
                    .unwrap_or(false);
                if can_insert_fallback_symbol(target, &item.name)
                    && !pending_ancestor_conflict
                    && !inserted.contains(&($kind, item.name.clone()))
                {
                    inserted.insert(($kind, item.name.clone()));
                    addition = addition.$push(item.clone());
                }
            }
        };
    }

    insert_missing!(enums, IrSymbolKind::Enum, push_enum);
    insert_missing!(type_aliases, IrSymbolKind::TypeAlias, push_type_alias);
    insert_missing!(messages, IrSymbolKind::Message, push_message);
    insert_missing!(groups, IrSymbolKind::Group, push_group);
    insert_missing!(variables, IrSymbolKind::Variable, push_variable);
    insert_missing!(forms, IrSymbolKind::Form, push_form);
    insert_missing!(functions, IrSymbolKind::Function, push_function);

    for origin in source.origins() {
        if inserted.contains(&(origin.kind, origin.name.clone())) {
            addition = addition.push_origin(origin.clone());
        }
    }

    let composed = addition.build().map_err(|conflict| {
        CliError::Diagnostics(format!("invalid fallback composition: {conflict}\n"))
    })?;
    target.try_append(composed).map_err(|conflict| {
        CliError::Diagnostics(format!("invalid fallback composition: {conflict}\n"))
    })?;
    Ok(())
}

fn walk_ancestors(name: &str) -> impl Iterator<Item = &str> {
    let mut next = Some(name);
    std::iter::from_fn(move || {
        let current = next?;
        next = current.rsplit_once('.').map(|(prefix, _)| prefix);
        Some(current)
    })
}

fn can_insert_fallback_symbol(module: &IrModule, name: &str) -> bool {
    let owned_exact = IrSymbolKind::non_group_kinds()
        .into_iter()
        .chain([IrSymbolKind::Group])
        .any(|kind| module.contains_symbol(kind, name));
    if owned_exact {
        return false;
    }

    !walk_ancestors(name)
        .skip(1)
        .any(|ancestor| module.contains_non_group_symbol(ancestor))
}

#[cfg(test)]
mod tests;
