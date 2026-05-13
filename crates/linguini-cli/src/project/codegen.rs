use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use linguini_analyzer::DiagnosticSeverity;
use linguini_codegen_ts::{
    generate_typescript_project_files, TypeScriptGeneratedFile, TypeScriptLocaleModule,
    TypeScriptProjectOptions,
};
use linguini_config::{LinguiniConfig, TypeScriptTargetConfig};
use linguini_ir::{ensure_no_unresolved_references, lower_locale, lower_schema, IrModule};

use crate::{CliError, CliResult};

use super::check::{check_project, reject_locale_files_without_schema_namespace};
use super::io::{
    create_dir_all, path_for_output, read_project_config, render_file_diagnostics, write_file,
};
use super::sources::{
    coverage_options, expected_locale_path, load_locale_sources, load_schema_sources, locale_index,
};
use super::{ParsedLocaleSource, ParsedSchemaSource};

pub fn build_project(root: &Path) -> CliResult<String> {
    let check_output = check_project(root)?;
    let config = read_project_config(root)?;
    let codegen_output = generate_project(root, &config)?;

    Ok(format!("{check_output}{codegen_output}build: ok\n"))
}

fn generate_project(root: &Path, config: &LinguiniConfig) -> CliResult<String> {
    let Some(target) = &config.targets.ts else {
        return Ok("codegen targets: none\n".to_owned());
    };

    generate_typescript_target(root, config, target)
}

fn generate_typescript_target(
    root: &Path,
    config: &LinguiniConfig,
    target: &TypeScriptTargetConfig,
) -> CliResult<String> {
    let schema_files = load_schema_sources(root, config)?;
    let schema_namespaces: BTreeSet<_> = schema_files
        .iter()
        .map(|schema| schema.file.namespace.clone())
        .collect();
    let schema = merge_schema_ir(&schema_files);
    let locale_files = load_locale_sources(root, config)?;
    let locale_index = locale_index(&locale_files);

    reject_locale_files_without_schema_namespace(root, &schema_namespaces, &locale_files)?;

    let mut locales = Vec::new();
    for locale in &config.project.locales {
        let locale_ir = build_locale_ir(root, config, &schema_files, &locale_index, locale)?;
        ensure_locale_ir_resolves(&schema, &locale_ir, locale)?;
        locales.push(TypeScriptLocaleModule {
            locale: locale.clone(),
            module: locale_ir,
        });
    }

    let files = generate_typescript_project_files(
        &schema,
        &locales,
        &TypeScriptProjectOptions {
            declaration: target.declaration,
            tree_shaking: target.tree_shaking,
            included_messages: target.messages.clone(),
        },
    )
    .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;

    write_codegen_tree(root, &root.join(&target.out), &files)
}

fn build_locale_ir(
    root: &Path,
    config: &LinguiniConfig,
    schema_files: &[ParsedSchemaSource],
    locale_index: &BTreeMap<(String, String), &ParsedLocaleSource>,
    locale: &str,
) -> CliResult<IrModule> {
    let mut locale_ir = IrModule::default();

    for schema_file in schema_files {
        let namespace = &schema_file.file.namespace;
        let locale_key = (namespace.clone(), locale.to_owned());
        let default_key = (namespace.clone(), config.project.default_locale.clone());
        let locale_file = locale_index.get(&locale_key);

        if locale == config.project.default_locale.as_str() && locale_file.is_none() {
            let path = expected_locale_path(root, config, namespace, locale);
            return Err(CliError::Diagnostics(format!(
                "required locale file is missing for schema namespace `{namespace}`: `{locale}`\nexpected path: {}\n",
                path_for_output(root, &path)
            )));
        }

        if let Some(locale_file) = locale_file {
            ensure_locale_has_required_messages(root, config, schema_file, locale_file, locale)?;
            merge_module(&mut locale_ir, lower_locale(&locale_file.ast));
        }

        if locale != config.project.default_locale.as_str() {
            if let Some(default_locale_file) = locale_index.get(&default_key) {
                merge_module_fallback(&mut locale_ir, lower_locale(&default_locale_file.ast));
            }
        }
    }

    Ok(locale_ir)
}

fn ensure_locale_has_required_messages(
    root: &Path,
    config: &LinguiniConfig,
    schema_file: &ParsedSchemaSource,
    locale_file: &ParsedLocaleSource,
    locale: &str,
) -> CliResult<()> {
    let diagnostics = linguini_analyzer::analyze_locale_coverage_with_options(
        &schema_file.ast,
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

fn write_codegen_tree(
    root: &Path,
    out_dir: &Path,
    files: &[TypeScriptGeneratedFile],
) -> CliResult<String> {
    let staging_dir = temp_codegen_path(out_dir, "tmp");
    let backup_dir = temp_codegen_path(out_dir, "old");

    remove_path_if_exists(&staging_dir)?;
    remove_path_if_exists(&backup_dir)?;
    create_dir_all(&staging_dir)?;

    let mut output = String::from("generated files:\n");
    for file in files {
        let relative_path = relative_codegen_path(&file.path)?;
        let staging_path = staging_dir.join(&relative_path);
        if let Some(parent) = staging_path.parent() {
            create_dir_all(parent)?;
        }
        write_file(&staging_path, &file.contents)?;
        output.push_str(&format!(
            "- {}\n",
            path_for_output(root, &out_dir.join(relative_path))
        ));
    }

    commit_codegen_tree(out_dir, &staging_dir, &backup_dir)?;
    output.push_str(&format!(
        "replaced generated tree: {}\n",
        path_for_output(root, out_dir)
    ));
    Ok(output)
}

fn commit_codegen_tree(out_dir: &Path, staging_dir: &Path, backup_dir: &Path) -> CliResult<()> {
    if let Some(parent) = out_dir.parent() {
        create_dir_all(parent)?;
    }

    remove_path_if_exists(backup_dir)?;
    let had_existing = match fs::symlink_metadata(out_dir) {
        Ok(_) => {
            fs::rename(out_dir, backup_dir).map_err(|source| CliError::Io {
                path: out_dir.to_path_buf(),
                source,
            })?;
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(source) => {
            return Err(CliError::Io {
                path: out_dir.to_path_buf(),
                source,
            });
        }
    };

    match fs::rename(staging_dir, out_dir) {
        Ok(()) => {
            if had_existing {
                remove_path_if_exists(backup_dir)?;
            }
            Ok(())
        }
        Err(source) => {
            if had_existing {
                let _ = remove_path_if_exists(out_dir);
                let _ = fs::rename(backup_dir, out_dir);
            }
            Err(CliError::Io {
                path: out_dir.to_path_buf(),
                source,
            })
        }
    }
}

fn relative_codegen_path(path: &str) -> CliResult<PathBuf> {
    let path = Path::new(path);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(CliError::Diagnostics(format!(
            "codegen backend returned unsafe output path `{}`\n",
            path.display()
        )));
    }
    Ok(path.to_path_buf())
}

fn temp_codegen_path(out_dir: &Path, purpose: &str) -> PathBuf {
    let name = out_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("linguini-generated");
    out_dir.with_file_name(format!(".{name}.{purpose}-{}", std::process::id()))
}

fn remove_path_if_exists(path: &Path) -> CliResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path).map_err(|source| CliError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
        Ok(_) => fs::remove_file(path).map_err(|source| CliError::Io {
            path: path.to_path_buf(),
            source,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(CliError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn merge_schema_ir(schema_files: &[ParsedSchemaSource]) -> IrModule {
    let mut schema = IrModule::default();
    for file in schema_files {
        merge_module(&mut schema, lower_schema(&file.ast));
    }
    schema
}

pub(super) fn merge_module(target: &mut IrModule, source: IrModule) {
    target.enums.extend(source.enums);
    target.type_aliases.extend(source.type_aliases);
    target.messages.extend(source.messages);
    target.forms.extend(source.forms);
    target.functions.extend(source.functions);
}

pub(super) fn merge_module_fallback(target: &mut IrModule, source: IrModule) {
    for item in source.enums {
        if !target
            .enums
            .iter()
            .any(|existing| existing.name == item.name)
        {
            target.enums.push(item);
        }
    }
    for item in source.type_aliases {
        if !target
            .type_aliases
            .iter()
            .any(|existing| existing.name == item.name)
        {
            target.type_aliases.push(item);
        }
    }
    for item in source.messages {
        if !target
            .messages
            .iter()
            .any(|existing| existing.name == item.name)
        {
            target.messages.push(item);
        }
    }
    for item in source.forms {
        if !target
            .forms
            .iter()
            .any(|existing| existing.name == item.name)
        {
            target.forms.push(item);
        }
    }
    for item in source.functions {
        if !target
            .functions
            .iter()
            .any(|existing| existing.name == item.name)
        {
            target.functions.push(item);
        }
    }
}
