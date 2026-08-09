use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use linguini_analyzer::{
    ApplicationBindingProvenance, ApplicationDynamicReferenceKind, ApplicationImportBinding,
    ApplicationImportBindingId, ApplicationReferenceKind, ApplicationUsage, DiagnosticSeverity,
};
use linguini_cldr::{canonicalize_locale, locale_fallback_chain};
use linguini_codegen_ts::{
    compile_typescript_bundler_message_module, generate_typescript_project_files, EcmaSource,
    TypeScriptFramework, TypeScriptGeneratedFile, TypeScriptLocaleModule, TypeScriptLocaleSource,
    TypeScriptProjectOptions, TypeScriptWebOptions, ValidatedTypeScriptProject,
};
use linguini_config::{
    discover_application_source_files_with_fields, CanonicalMode, CookiePath, LinguiniConfig,
    LinkMode, LocalePrefixMode, SecurePolicy, TypeScriptBundlerConfig, TypeScriptTargetConfig,
};
use linguini_ir::{
    ensure_no_unresolved_references, lower_locale, lower_schema, qualify_module, IrModule,
    IrSymbolKind,
};
use linguini_syntax::SourceId;
use sha2::{Digest, Sha256};

use crate::{CliError, CliResult, DiagnosticFormat};

use super::check::{check_project_with_options, reject_locale_files_without_schema_namespace};
use super::io::{path_for_output, read_file, read_project_config, render_file_diagnostics};
use super::output::{replace_owned_files, GeneratedFile, SafeOutputRoot};
use super::sources::{
    coverage_options, expected_locale_path, load_locale_sources, load_schema_sources, locale_index,
    schema_project_diagnostics,
};
use super::{ParsedLocaleSource, ParsedSchemaSource};

pub fn build_project(root: &Path) -> CliResult<String> {
    build_project_with_options(root, false, DiagnosticFormat::Human)
}

pub(crate) fn build_project_with_options(
    root: &Path,
    deny_warnings: bool,
    format: DiagnosticFormat,
) -> CliResult<String> {
    let check_output = check_project_with_options(root, deny_warnings, format)?;
    let config = read_project_config(root)?;
    let codegen_output = generate_project(root, &config)?;

    if format == DiagnosticFormat::Human {
        Ok(format!("{check_output}{codegen_output}build: ok\n"))
    } else {
        Ok(check_output)
    }
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
    ensure_schema_project_valid(root, &schema_files)?;
    let schema = merge_schema_ir(&schema_files);
    let locale_files = load_locale_sources(root, config)?;
    let locale_index = locale_index(&locale_files)?;

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

    let options = TypeScriptProjectOptions {
        declaration: target.declaration,
        gitignore: target.gitignore,
        tree_shaking: target.tree_shaking,
        included_messages: target.messages.clone(),
        base_locale: Some(config.project.default_locale.clone()),
        web: config
            .web
            .configured
            .then(|| legacy_web_codegen_options(config)),
        framework: TypeScriptFramework::from_config(target.framework.as_deref()),
    };
    let project = ValidatedTypeScriptProject::try_new(&schema, &locales, &options)
        .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
    let mut files = generate_typescript_project_files(&project)
        .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
    let sources = ecma_sources(root, &schema_files, &locale_files)?;
    files.extend(generate_bundler_files(
        root,
        &project,
        &sources,
        target,
        &config.project.locales,
        &config.project.default_locale,
        config.web.configured,
    )?);

    let output = SafeOutputRoot::new(
        root,
        Path::new(&target.out),
        &[
            Path::new(&config.paths.schema),
            Path::new(&config.paths.locale),
        ],
    )?;
    write_codegen_tree(root, &output, &files)
}

fn ecma_sources(
    root: &Path,
    schema_files: &[ParsedSchemaSource],
    locale_files: &[ParsedLocaleSource],
) -> CliResult<Vec<EcmaSource>> {
    let mut sources = schema_files
        .iter()
        .map(|source| {
            Ok(EcmaSource::new(
                source.ast.span.source,
                project_relative_source_path(root, &source.file.path)?,
                source.source.clone(),
            ))
        })
        .chain(locale_files.iter().map(|source| {
            Ok(EcmaSource::new(
                source.ast.span.source,
                project_relative_source_path(root, &source.file.path)?,
                source.source.clone(),
            ))
        }))
        .collect::<CliResult<Vec<_>>>()?;
    sources.sort_by_key(|source| source.id);
    Ok(sources)
}

fn project_relative_source_path(root: &Path, path: &Path) -> CliResult<String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        CliError::Diagnostics(format!(
            "localization source path is outside the project root: `{}`\n",
            path.display()
        ))
    })?;
    let mut components = Vec::new();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(CliError::Diagnostics(format!(
                "localization source path is not a clean project-relative path: `{}`\n",
                path.display()
            )));
        };
        let component = component.to_str().ok_or_else(|| {
            CliError::Diagnostics(format!(
                "localization source path contains a non-UTF-8 component: `{}`\n",
                path.display()
            ))
        })?;
        if component.is_empty() || component == "." || component == ".." {
            return Err(CliError::Diagnostics(format!(
                "localization source path contains an empty, dot, or parent component: `{}`\n",
                path.display()
            )));
        }
        if component.contains('\\') {
            return Err(CliError::Diagnostics(format!(
                "localization source path contains a literal backslash and cannot be represented as a portable POSIX path: `{}`\n",
                path.display()
            )));
        }
        components.push(component);
    }
    if components.is_empty() {
        return Err(CliError::Diagnostics(format!(
            "localization source path resolves to the project root: `{}`\n",
            path.display()
        )));
    }
    Ok(components.join("/"))
}

fn generate_bundler_files(
    root: &Path,
    project: &ValidatedTypeScriptProject<'_>,
    sources: &[EcmaSource],
    target: &TypeScriptTargetConfig,
    configured_locales: &[String],
    base_locale: &str,
    web_enabled: bool,
) -> CliResult<Vec<TypeScriptGeneratedFile>> {
    let output_root = &target.out;
    let artifacts = project
        .message_artifacts()
        .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
    let runtime_artifacts = project
        .locale_runtime_artifacts()
        .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
    let mut files = Vec::with_capacity(artifacts.len() * 2 + 1);
    let mut messages = BTreeMap::<String, (usize, BTreeMap<String, serde_json::Value>)>::new();
    let effective_locales = project
        .effective_locales()
        .into_iter()
        .collect::<BTreeSet<_>>();

    for artifact in artifacts {
        let map_directory = format!(
            "{output_root}/{}",
            artifact
                .source_map_path
                .rsplit_once('/')
                .map_or("", |(directory, _)| directory)
        );
        let map_sources = sources
            .iter()
            .map(|source| {
                lexical_relative_path(&map_directory, &source.path)
                    .map(|path| EcmaSource::new(source.id, path, source.contents.clone()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|reason| {
                CliError::Diagnostics(format!(
                    "bundler message `{}` locale `{}`: cannot rebase source-map paths: {reason}\n",
                    artifact.message, artifact.locale
                ))
            })?;
        let compiled = compile_typescript_bundler_message_module(
            project,
            &artifact.locale,
            &artifact.message,
            &artifact.output_file_name,
            &artifact.shared_import_path,
            &artifact.runtime_import_path,
            &map_sources,
        )
        .map_err(|error| {
            CliError::Diagnostics(format!(
                "bundler message `{}` locale `{}`: {error}\n",
                artifact.message, artifact.locale
            ))
        })?;
        let locale_entry = serde_json::json!({
            "module": artifact.module_path,
            "source_ids": compiled.source_ids().iter().map(|id| id.0).collect::<Vec<_>>(),
        });
        messages
            .entry(artifact.message.clone())
            .or_insert_with(|| (artifact.arity, BTreeMap::new()))
            .1
            .insert(artifact.locale.clone(), locale_entry);
        files.push(TypeScriptGeneratedFile {
            path: artifact.module_path,
            contents: compiled.code,
        });
        files.push(TypeScriptGeneratedFile {
            path: artifact.source_map_path,
            contents: compiled.source_map,
        });
    }

    let message_arities = messages
        .iter()
        .map(|(message, (arity, _))| (message.clone(), *arity))
        .collect::<BTreeMap<_, _>>();
    let messages = messages
        .into_iter()
        .map(|(message, (arity, locales))| {
            (
                message,
                serde_json::json!({
                    "arity": arity,
                    "locales": locales,
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let source_table = sources
        .iter()
        .map(|source| {
            serde_json::json!({
                "id": source.id.0,
                "path": source.path,
            })
        })
        .collect::<Vec<_>>();
    let mut manifest = serde_json::json!({
        "version": 1,
        "base_locale": base_locale,
        "configured_locales": configured_locales,
        "effective_locales": effective_locales,
        "sources": source_table,
        "messages": messages,
    });
    if let Some(bundler) = &target.bundler {
        let applications = scan_bundler_applications(root, output_root, bundler, &message_arities)?;
        let manifest = manifest.as_object_mut().expect("manifest is an object");
        manifest.insert("version".to_owned(), serde_json::json!(5));
        manifest.insert(
            "locale_loading".to_owned(),
            serde_json::json!(bundler.locale_loading.as_str()),
        );
        manifest.insert("applications".to_owned(), serde_json::json!(applications));
        let message_runtimes = runtime_artifacts
            .iter()
            .map(|artifact| {
                (
                    artifact.locale.clone(),
                    serde_json::json!({
                        "module": artifact.module_path,
                        "source_ids": artifact.source_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                    }),
                )
            })
            .collect::<BTreeMap<_, _>>();
        manifest.insert(
            "message_runtimes".to_owned(),
            serde_json::json!(message_runtimes),
        );
        let mut runtime_helpers = serde_json::Map::new();
        runtime_helpers.insert(
            "svelte_locale".to_owned(),
            serde_json::json!({
                "import": "./svelte-locale.svelte.js",
                "file": "svelte-locale.svelte.ts",
            }),
        );
        if web_enabled {
            runtime_helpers.insert(
                "svelte_effects".to_owned(),
                serde_json::json!({
                    "import": "./svelte-effects.svelte.js",
                    "file": "svelte-effects.svelte.ts",
                }),
            );
        }
        manifest.insert(
            "runtime_helpers".to_owned(),
            serde_json::Value::Object(runtime_helpers),
        );
    }
    let mut contents = serde_json::to_string_pretty(&manifest).map_err(|error| {
        CliError::Diagnostics(format!("failed to serialize bundler manifest: {error}\n"))
    })?;
    contents.push('\n');
    files.push(TypeScriptGeneratedFile {
        path: "bundler/manifest.json".to_owned(),
        contents,
    });
    Ok(files)
}

const APPLICATION_SOURCE_ID_BASE: u32 = 0x8000_0000;

fn scan_bundler_applications(
    root: &Path,
    output_root: &str,
    config: &TypeScriptBundlerConfig,
    messages: &BTreeMap<String, usize>,
) -> CliResult<BTreeMap<String, serde_json::Value>> {
    validate_dynamic_allow_entries(&config.dynamic, messages)?;
    let mut exclude = config.exclude.clone();
    exclude.push(output_root.to_owned());
    let paths = discover_application_source_files_with_fields(
        root,
        &config.sources,
        &exclude,
        "targets.ts.bundler.sources",
        "targets.ts.bundler.exclude",
    )?;
    let mut applications = BTreeMap::new();
    let mut portable_paths = BTreeSet::new();
    for (index, path) in paths.into_iter().enumerate() {
        let path_key = project_relative_source_path(root, &path)?;
        if !portable_paths.insert(path_key.to_ascii_lowercase()) {
            return Err(CliError::Diagnostics(format!(
                "bundler application paths are not case-distinct: `{path_key}`\n"
            )));
        }
        let source = read_file(&path)?;
        // Localization IDs occupy low odd/even values. Sorted app paths occupy high-bit IDs.
        let index = u32::try_from(index).map_err(|_| {
            CliError::Diagnostics("project contains too many application source files\n".to_owned())
        })?;
        let source_id = APPLICATION_SOURCE_ID_BASE
            .checked_add(index)
            .ok_or_else(|| {
                CliError::Diagnostics(
                    "project contains too many application source files\n".to_owned(),
                )
            })?;
        let usage = ApplicationUsage::from_source_in(&source, SourceId(source_id));
        applications.insert(
            path_key.clone(),
            application_entry(
                &path_key,
                &source,
                SourceId(source_id),
                &usage,
                messages,
                &config.dynamic,
            )?,
        );
    }
    Ok(applications)
}

fn application_entry(
    path: &str,
    source: &str,
    source_id: SourceId,
    usage: &ApplicationUsage,
    messages: &BTreeMap<String, usize>,
    dynamic: &linguini_config::TypeScriptBundlerDynamicConfig,
) -> CliResult<serde_json::Value> {
    let mut references = Vec::new();
    let mut unresolved = Vec::new();
    let mut poisoned_imports = BTreeSet::new();
    let mut referenced_paths = BTreeSet::new();
    let import_ids = usage
        .imports()
        .map(|binding| binding.id)
        .collect::<BTreeSet<_>>();
    let import_bindings = usage
        .imports()
        .map(|binding| (binding.id, binding))
        .collect::<BTreeMap<_, _>>();
    for reference in usage.references() {
        if reference.span.start > reference.span.end
            || reference.span.end > source.len()
            || !source.is_char_boundary(reference.span.start)
            || !source.is_char_boundary(reference.span.end)
        {
            return Err(CliError::Diagnostics(format!(
                "bundler analyzer returned an invalid UTF-8 byte span for `{}` in `{path}`: {}..{}\n",
                reference.canonical_path, reference.span.start, reference.span.end
            )));
        }
        if reference
            .import_binding
            .is_some_and(|binding| !import_ids.contains(&binding))
        {
            return Err(CliError::Diagnostics(format!(
                "bundler analyzer returned unknown import identity for `{}` in `{path}`\n",
                reference.binding.local
            )));
        }
        referenced_paths.insert(reference.canonical_path.as_str());
        if source[reference.span.start..reference.span.end].contains("?.")
            || has_optional_invocation(source, reference.span.end)
        {
            poisoned_imports.extend(reference.import_binding);
            unresolved.push((
                reference.span.start,
                reference.span.end,
                reference.canonical_path.clone(),
                serde_json::json!({
                    "message": reference.canonical_path,
                    "start": reference.span.start,
                    "end": reference.span.end,
                    "reason": "optional_chain",
                    "binding_id": reference.import_binding.map(render_import_binding_id),
                }),
            ));
            continue;
        }
        let Some(&arity) = messages.get(&reference.canonical_path) else {
            poisoned_imports.extend(reference.import_binding);
            unresolved.push((
                reference.span.start,
                reference.span.end,
                reference.canonical_path.clone(),
                serde_json::json!({
                    "message": reference.canonical_path,
                    "start": reference.span.start,
                    "end": reference.span.end,
                    "reason": "non_exact_message_path",
                    "binding_id": reference.import_binding.map(render_import_binding_id),
                }),
            ));
            continue;
        };
        let kind = match reference.kind {
            ApplicationReferenceKind::Value => "value",
            ApplicationReferenceKind::Call => "call",
        };
        let compatible = matches!(reference.kind, ApplicationReferenceKind::Value) && arity == 0
            || matches!(reference.kind, ApplicationReferenceKind::Call) && arity > 0;
        if !compatible {
            poisoned_imports.extend(reference.import_binding);
            unresolved.push((
                reference.span.start,
                reference.span.end,
                reference.canonical_path.clone(),
                serde_json::json!({
                    "message": reference.canonical_path,
                    "start": reference.span.start,
                    "end": reference.span.end,
                    "kind": kind,
                    "arity": arity,
                    "reason": "arity_mismatch",
                    "binding_id": reference.import_binding.map(render_import_binding_id),
                }),
            ));
            continue;
        }
        let provenance = match &reference.binding.provenance {
            ApplicationBindingProvenance::Imported {
                module_specifier,
                imported,
            } => serde_json::json!({
                "kind": "imported",
                "module_specifier": module_specifier,
                "symbol": imported,
            }),
            ApplicationBindingProvenance::Factory { factory } => {
                serde_json::json!({"kind": "factory", "factory": factory})
            }
            ApplicationBindingProvenance::Implicit => serde_json::json!({"kind": "implicit"}),
        };
        references.push((
            reference.span.start,
            reference.span.end,
            reference.canonical_path.clone(),
            serde_json::json!({
                "message": reference.canonical_path,
                "start": reference.span.start,
                "end": reference.span.end,
                "kind": kind,
                "local": reference.binding.local,
                "provenance": provenance,
                "arity": arity,
                "binding_id": reference.import_binding.map(render_import_binding_id),
            }),
        ));
    }
    let dynamic_references = application_dynamic_references(
        path,
        source,
        source_id,
        usage,
        messages,
        dynamic,
        &import_bindings,
        &poisoned_imports,
    )?;
    for static_path in usage.static_paths() {
        if !referenced_paths.contains(static_path) {
            unresolved.push((
                usize::MAX - 1,
                usize::MAX - 1,
                static_path.to_owned(),
                serde_json::json!({
                    "message": static_path,
                    "reason": "missing_exact_span",
                }),
            ));
        }
    }
    references
        .sort_by(|left, right| (&left.0, &left.1, &left.2).cmp(&(&right.0, &right.1, &right.2)));
    unresolved
        .sort_by(|left, right| (&left.0, &left.1, &left.2).cmp(&(&right.0, &right.1, &right.2)));
    let digest = Sha256::digest(source.as_bytes());
    let mut previous_end = 0;
    for reference in &references {
        if reference.0 < previous_end {
            return Err(CliError::Diagnostics(format!(
                "bundler analyzer returned overlapping reference spans in `{path}` at bytes {}..{}\n",
                reference.0, reference.1
            )));
        }
        previous_end = reference.1;
    }
    let imports = usage
        .imports()
        .map(|binding| {
            validate_import_binding(path, source, source_id, binding)?;
            Ok(serde_json::json!({
                "binding_id": render_import_binding_id(binding.id),
                "module_specifier": binding.module_specifier,
                "imported": binding.imported,
                "local": binding.local,
                "declaration_start": binding.declaration_span.start,
                "declaration_end": binding.declaration_span.end,
                "item_start": binding.item_span.start,
                "item_end": binding.item_span.end,
                "removal_start": binding.removal_span.start,
                "removal_end": binding.removal_span.end,
                "module_specifier_start": binding.module_specifier_span.start,
                "module_specifier_end": binding.module_specifier_span.end,
                "imported_start": binding.imported_span.start,
                "imported_end": binding.imported_span.end,
                "local_start": binding.local_span.start,
                "local_end": binding.local_span.end,
                "analyzer_exact_uses_only": binding.exact_uses_only,
                "analyzer_tracked_uses_only": binding.tracked_uses_only,
                "transformable": binding.tracked_uses_only && !poisoned_imports.contains(&binding.id),
            }))
        })
        .collect::<CliResult<Vec<_>>>()?;
    Ok(serde_json::json!({
        "sha256": format!("{digest:x}"),
        "byte_length": source.len(),
        "source_id": source_id.0,
        "references": references.into_iter().map(|entry| entry.3).collect::<Vec<_>>(),
        "unresolved": unresolved.into_iter().map(|entry| entry.3).collect::<Vec<_>>(),
        "analysis_dynamic_prefixes": usage.dynamic_prefixes().collect::<Vec<_>>(),
        "dynamic_references": dynamic_references,
        "imports": imports,
    }))
}

fn validate_dynamic_allow_entries(
    dynamic: &linguini_config::TypeScriptBundlerDynamicConfig,
    messages: &BTreeMap<String, usize>,
) -> CliResult<()> {
    if dynamic.mode != linguini_config::TypeScriptBundlerDynamicMode::Bundle {
        return Ok(());
    }
    let unknown = dynamic
        .allow
        .iter()
        .filter(|message| !messages.contains_key(*message))
        .cloned()
        .collect::<Vec<_>>();
    if unknown.is_empty() {
        return Ok(());
    }
    Err(CliError::Diagnostics(format!(
        "targets.ts.bundler.dynamic.allow contains unknown compiled message leaf{}: {}\n",
        if unknown.len() == 1 { "" } else { "s" },
        unknown
            .iter()
            .map(|message| format!("`{message}`"))
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

#[allow(clippy::too_many_arguments)]
fn application_dynamic_references(
    path: &str,
    source: &str,
    source_id: linguini_syntax::SourceId,
    usage: &ApplicationUsage,
    messages: &BTreeMap<String, usize>,
    dynamic: &linguini_config::TypeScriptBundlerDynamicConfig,
    import_bindings: &BTreeMap<ApplicationImportBindingId, &ApplicationImportBinding>,
    poisoned_imports: &BTreeSet<ApplicationImportBindingId>,
) -> CliResult<Vec<serde_json::Value>> {
    let mut output = Vec::new();
    let mut errors = Vec::new();
    for reference in usage.dynamic_references() {
        validate_dynamic_spans(path, source, source_id, reference)?;
        let kind = dynamic_reference_kind_name(reference.kind);
        let reference_kind = application_reference_kind_name(reference.reference_kind);
        if dynamic.mode == linguini_config::TypeScriptBundlerDynamicMode::Error {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                format!(
                    "dynamic message access ({kind}, {reference_kind}) requires a finite escape"
                ),
            ));
            continue;
        }
        if reference.kind != ApplicationDynamicReferenceKind::Computed {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                format!("dynamic reference kind `{kind}` is not bundle-safe"),
            ));
            continue;
        }
        if !matches!(
            &reference.binding.provenance,
            ApplicationBindingProvenance::Imported { .. }
        ) {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                format!(
                    "dynamic reference provenance `{}` is not an imported message root",
                    application_binding_provenance_name(&reference.binding.provenance)
                ),
            ));
            continue;
        }
        let Some(import_binding_id) = reference.import_binding else {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                "dynamic reference has no unique removable import binding".to_owned(),
            ));
            continue;
        };
        let Some(binding) = import_bindings.get(&import_binding_id).copied() else {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                "dynamic reference has an unknown import binding".to_owned(),
            ));
            continue;
        };
        let imported_message_root = matches!(
            (&reference.binding.provenance, binding.imported.as_str()),
            (
                ApplicationBindingProvenance::Imported { .. },
                "l" | "messages"
            )
        );
        if !imported_message_root
            || reference.binding.local != binding.local
            || reference.import_binding != Some(binding.id)
        {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                "dynamic reference must use a unique imported `l` or `messages` binding".to_owned(),
            ));
            continue;
        }
        if !binding.tracked_uses_only {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                "dynamic reference import binding has untracked uses".to_owned(),
            ));
            continue;
        }
        if poisoned_imports.contains(&binding.id) {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                "dynamic reference import binding has poisoned unresolved uses".to_owned(),
            ));
            continue;
        }
        let allowed = immediate_dynamic_messages(&reference.canonical_prefix, dynamic, messages);
        if allowed.is_empty() {
            errors.push(dynamic_diagnostic(
                path,
                reference.span.start,
                reference.span.end,
                format!(
                    "no finite allow entry is an immediate compiled child of prefix `{}`",
                    reference.canonical_prefix
                ),
            ));
            continue;
        }
        let provenance = match &reference.binding.provenance {
            ApplicationBindingProvenance::Imported {
                module_specifier,
                imported,
            } => serde_json::json!({
                "kind": "imported",
                "module_specifier": module_specifier,
                "symbol": imported,
            }),
            ApplicationBindingProvenance::Factory { factory } => {
                serde_json::json!({"kind": "factory", "factory": factory})
            }
            ApplicationBindingProvenance::Implicit => serde_json::json!({"kind": "implicit"}),
        };
        output.push(serde_json::json!({
            "kind": kind,
            "prefix": reference.canonical_prefix,
            "span": span_json(reference.span),
            "receiver_span": span_json(reference.receiver_span),
            "key_span": reference.computed_key_span.map(span_json),
            "reference_kind": reference_kind,
            "local": reference.binding.local,
            "provenance": provenance,
            "binding_id": Some(render_import_binding_id(binding.id)),
            "messages": allowed
                .into_iter()
                .map(|(message, key, arity)| serde_json::json!({
                    "message": message,
                    "key": key,
                    "arity": arity,
                }))
                .collect::<Vec<_>>(),
        }));
    }
    if errors.is_empty() {
        Ok(output)
    } else {
        Err(CliError::Diagnostics(errors.join("")))
    }
}

fn immediate_dynamic_messages<'a>(
    prefix: &str,
    dynamic: &linguini_config::TypeScriptBundlerDynamicConfig,
    messages: &'a BTreeMap<String, usize>,
) -> Vec<(&'a str, &'a str, usize)> {
    let prefix = prefix.strip_suffix('.').unwrap_or(prefix);
    messages
        .iter()
        .filter_map(|(message, arity)| {
            if !dynamic.allow.iter().any(|allowed| allowed == message) {
                return None;
            }
            let key = if prefix.is_empty() {
                message.as_str()
            } else {
                message.strip_prefix(&format!("{prefix}."))?
            };
            if key.is_empty() || key.contains('.') {
                return None;
            }
            Some((message.as_str(), key, *arity))
        })
        .collect()
}

fn validate_dynamic_spans(
    path: &str,
    source: &str,
    source_id: linguini_syntax::SourceId,
    reference: &linguini_analyzer::ApplicationDynamicReference,
) -> CliResult<()> {
    let spans = [
        ("full", reference.span),
        ("receiver", reference.receiver_span),
    ];
    for (name, span) in spans {
        if span.source != source_id
            || span.start >= span.end
            || span.end > source.len()
            || !source.is_char_boundary(span.start)
            || !source.is_char_boundary(span.end)
            || span.start < reference.span.start
            || span.end > reference.span.end
        {
            return Err(CliError::Diagnostics(format!(
                "bundler analyzer returned invalid dynamic {name} span in `{path}`: {}..{}\n",
                span.start, span.end
            )));
        }
    }
    if let Some(span) = reference.computed_key_span {
        if span.source != source_id
            || span.start > span.end
            || span.end > source.len()
            || !source.is_char_boundary(span.start)
            || !source.is_char_boundary(span.end)
            || span.start < reference.receiver_span.end
            || span.end > reference.span.end
        {
            return Err(CliError::Diagnostics(format!(
                "bundler analyzer returned invalid dynamic key span in `{path}`: {}..{}\n",
                span.start, span.end
            )));
        }
    }
    Ok(())
}

fn dynamic_diagnostic(path: &str, start: usize, end: usize, reason: String) -> String {
    format!(
        "{path}:{start}..{end}: {reason}; configure `[targets.ts.bundler.dynamic] mode = \"bundle\" allow = [...]`\n"
    )
}

fn dynamic_reference_kind_name(kind: ApplicationDynamicReferenceKind) -> &'static str {
    match kind {
        ApplicationDynamicReferenceKind::Computed => "computed",
        ApplicationDynamicReferenceKind::Bare => "bare",
        ApplicationDynamicReferenceKind::Factory => "factory",
        ApplicationDynamicReferenceKind::MultipleComputed => "multiple_computed",
        ApplicationDynamicReferenceKind::Uncertain => "uncertain",
    }
}

fn application_reference_kind_name(kind: ApplicationReferenceKind) -> &'static str {
    match kind {
        ApplicationReferenceKind::Value => "value",
        ApplicationReferenceKind::Call => "call",
    }
}

fn application_binding_provenance_name(provenance: &ApplicationBindingProvenance) -> &'static str {
    match provenance {
        ApplicationBindingProvenance::Imported { .. } => "imported",
        ApplicationBindingProvenance::Factory { .. } => "factory",
        ApplicationBindingProvenance::Implicit => "implicit",
    }
}

fn span_json(span: linguini_syntax::Span) -> serde_json::Value {
    serde_json::json!({"start": span.start, "end": span.end})
}

fn render_import_binding_id(id: ApplicationImportBindingId) -> String {
    format!("{}:{}:{}", id.source.0, id.declaration_start, id.item_start)
}

fn validate_import_binding(
    path: &str,
    source: &str,
    source_id: SourceId,
    binding: &linguini_analyzer::ApplicationImportBinding,
) -> CliResult<()> {
    if binding.id.source != source_id {
        return Err(CliError::Diagnostics(format!(
            "bundler analyzer returned import identity from source {} for `{}` in `{path}`, expected source {}\n",
            binding.id.source.0, binding.local, source_id.0
        )));
    }
    let spans = [
        ("declaration", binding.declaration_span),
        ("item", binding.item_span),
        ("removal", binding.removal_span),
        ("module specifier", binding.module_specifier_span),
        ("imported token", binding.imported_span),
        ("local token", binding.local_span),
    ];
    for (name, span) in spans {
        if span.source != binding.id.source
            || span.start > span.end
            || span.end > source.len()
            || !source.is_char_boundary(span.start)
            || !source.is_char_boundary(span.end)
            || span.start < binding.declaration_span.start
            || span.end > binding.declaration_span.end
        {
            return Err(CliError::Diagnostics(format!(
                "bundler analyzer returned invalid {name} span for import `{}` in `{path}`: {}..{}\n",
                binding.local, span.start, span.end
            )));
        }
    }
    if binding.id.declaration_start != binding.declaration_span.start
        || binding.id.item_start != binding.item_span.start
    {
        return Err(CliError::Diagnostics(format!(
            "bundler analyzer returned inconsistent import identity for `{}` in `{path}`\n",
            binding.local
        )));
    }
    Ok(())
}

fn has_optional_invocation(source: &str, span_end: usize) -> bool {
    let Some(after_span) = source.get(span_end..) else {
        return false;
    };
    let Some(after_chain) = strip_js_trivia(after_span).strip_prefix("?.") else {
        return false;
    };
    strip_js_trivia(after_chain).starts_with('(')
}

fn strip_js_trivia(mut source: &str) -> &str {
    loop {
        let trimmed = source.trim_start();
        if let Some(comment) = trimmed.strip_prefix("//") {
            source = comment
                .find(['\r', '\n'])
                .map_or("", |newline| &comment[newline..]);
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix("/*") {
            let Some(end) = comment.find("*/") else {
                return trimmed;
            };
            source = &comment[end + 2..];
            continue;
        }
        return trimmed;
    }
}

fn lexical_relative_path(from_directory: &str, target: &str) -> Result<String, &'static str> {
    let from = portable_relative_components(from_directory)?;
    let target = portable_relative_components(target)?;
    let common = from
        .iter()
        .zip(&target)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = vec![".."; from.len() - common];
    relative.extend(target[common..].iter().copied());
    if relative.is_empty() {
        return Err("source path resolves to the map directory");
    }
    Ok(relative.join("/"))
}

fn portable_relative_components(value: &str) -> Result<Vec<&str>, &'static str> {
    let has_windows_prefix = value
        .as_bytes()
        .get(1)
        .is_some_and(|character| *character == b':');
    if value.is_empty() || value.starts_with('/') || value.contains('\\') || has_windows_prefix {
        return Err("path must be non-empty, project-relative, and use `/` separators");
    }
    let components = value
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect::<Vec<_>>();
    if components.is_empty() || components.contains(&"..") {
        return Err("path must not contain parent traversal or resolve to the project root");
    }
    Ok(components)
}

fn legacy_web_codegen_options(config: &LinguiniConfig) -> TypeScriptWebOptions {
    let cookie = config.web.cookie.as_ref();
    let local_storage = config.web.local_storage.as_ref();
    let sources = config
        .web
        .locale
        .sources
        .iter()
        .map(|source| match source {
            linguini_config::LocaleSource::Path => TypeScriptLocaleSource::Path,
            linguini_config::LocaleSource::Cookie => TypeScriptLocaleSource::Cookie,
            linguini_config::LocaleSource::LocalStorage => TypeScriptLocaleSource::LocalStorage,
            linguini_config::LocaleSource::AcceptLanguage => TypeScriptLocaleSource::AcceptLanguage,
        })
        .collect::<Vec<_>>();

    TypeScriptWebOptions {
        sources,
        cookie_name: cookie
            .map(|cookie| cookie.name.clone())
            .unwrap_or_else(|| "LINGUINI_LOCALE".to_owned()),
        cookie_path: cookie
            .map(|cookie| match &cookie.path {
                CookiePath::Auto => "/".to_owned(),
                CookiePath::Explicit(path) => path.clone(),
            })
            .unwrap_or_else(|| "/".to_owned()),
        cookie_domain: cookie.and_then(|cookie| cookie.domain.clone()),
        cookie_max_age: cookie
            .map(|cookie| cookie.max_age_seconds)
            .unwrap_or(365 * 24 * 60 * 60),
        cookie_same_site: cookie
            .map(|cookie| cookie.same_site.as_str().to_owned())
            .unwrap_or_else(|| "lax".to_owned()),
        cookie_secure: cookie
            .map(|cookie| cookie.secure == SecurePolicy::Always)
            .unwrap_or(false),
        cookie_http_only: cookie.map(|cookie| cookie.http_only).unwrap_or(false),
        local_storage_key: local_storage
            .map(|storage| storage.key.clone())
            .unwrap_or_else(|| "LINGUINI_LOCALE".to_owned()),
        prefix_default_locale: config.web.routing.locale_prefix == LocalePrefixMode::Always,
        base_path: String::new(),
        redirect: config.web.routing.canonical == CanonicalMode::Redirect,
        origin: None,
        exclude: config.web.routes.exclude.clone(),
        localize_links: config.web.links.mode == LinkMode::Runtime,
    }
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
            merge_module(
                &mut locale_ir,
                namespaced_module(lower_locale(&locale_file.ast), namespace),
            );
        }

        for fallback_locale in project_locale_fallbacks(
            &config.project.locales,
            locale,
            &config.project.default_locale,
        ) {
            let fallback_key = (namespace.clone(), fallback_locale.to_owned());
            if let Some(default_locale_file) = locale_index.get(&fallback_key) {
                merge_module_fallback(
                    &mut locale_ir,
                    namespaced_module(lower_locale(&default_locale_file.ast), namespace),
                );
            }
        }
    }

    Ok(locale_ir)
}

fn project_locale_fallbacks<'a>(
    locales: &'a [String],
    locale: &str,
    default_locale: &'a str,
) -> Vec<&'a str> {
    let tags = locale_fallback_chain(locale).unwrap_or_else(|_| vec![locale.to_owned()]);
    let mut fallbacks = Vec::new();
    for tag in tags {
        let Some(configured) = locales.iter().find(|candidate| {
            canonicalize_locale(candidate)
                .is_ok_and(|canonical| canonical.eq_ignore_ascii_case(&tag))
        }) else {
            continue;
        };
        if !configured.eq_ignore_ascii_case(locale)
            && !fallbacks
                .iter()
                .any(|existing: &&str| existing.eq_ignore_ascii_case(configured))
        {
            fallbacks.push(configured.as_str());
        }
    }
    if !default_locale.eq_ignore_ascii_case(locale)
        && !fallbacks
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(default_locale))
    {
        fallbacks.push(default_locale);
    }
    fallbacks
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
                    .is_some_and(|span| span.source == source.ast.span.source)
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

fn write_codegen_tree(
    root: &Path,
    out_dir: &SafeOutputRoot,
    files: &[TypeScriptGeneratedFile],
) -> CliResult<String> {
    let mut generated = Vec::with_capacity(files.len());
    let mut output = String::from("generated files:\n");
    for file in files {
        let relative_path = relative_codegen_path(&file.path)?;
        generated.push(GeneratedFile {
            path: relative_path.clone(),
            contents: &file.contents,
        });
        output.push_str(&format!(
            "- {}\n",
            path_for_output(root, &out_dir.path().join(relative_path))
        ));
    }

    replace_owned_files(out_dir, &generated)?;
    output.push_str(&format!(
        "replaced generated tree: {}\n",
        path_for_output(root, out_dir.path())
    ));
    Ok(output)
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

fn merge_schema_ir(schema_files: &[ParsedSchemaSource]) -> IrModule {
    let mut schema = IrModule::default();
    for file in schema_files {
        merge_module(
            &mut schema,
            namespaced_module(lower_schema(&file.ast), &file.file.namespace),
        );
    }
    schema
}

pub(super) fn namespaced_module(mut module: IrModule, namespace: &str) -> IrModule {
    qualify_module(&mut module, namespace);
    module
}

pub(super) fn merge_module(target: &mut IrModule, source: IrModule) {
    target.enums.extend(source.enums);
    target.type_aliases.extend(source.type_aliases);
    target.variables.extend(source.variables);
    target.messages.extend(source.messages);
    target.groups.extend(source.groups);
    target.forms.extend(source.forms);
    target.functions.extend(source.functions);
    target.origins.extend(source.origins);
}

pub(super) fn merge_module_fallback(target: &mut IrModule, source: IrModule) {
    let mut inserted = BTreeSet::new();
    for item in source.enums {
        if can_insert_fallback_symbol(target, &item.name) {
            inserted.insert((IrSymbolKind::Enum, item.name.clone()));
            target.enums.push(item);
        }
    }
    for item in source.type_aliases {
        if can_insert_fallback_symbol(target, &item.name) {
            inserted.insert((IrSymbolKind::TypeAlias, item.name.clone()));
            target.type_aliases.push(item);
        }
    }
    for item in source.messages {
        if can_insert_fallback_symbol(target, &item.name) {
            inserted.insert((IrSymbolKind::Message, item.name.clone()));
            target.messages.push(item);
        }
    }
    for item in source.groups {
        if can_insert_fallback_symbol(target, &item.name) {
            inserted.insert((IrSymbolKind::Group, item.name.clone()));
            target.groups.push(item);
        }
    }
    for item in source.variables {
        if can_insert_fallback_symbol(target, &item.name) {
            inserted.insert((IrSymbolKind::Variable, item.name.clone()));
            target.variables.push(item);
        }
    }
    for item in source.forms {
        if can_insert_fallback_symbol(target, &item.name) {
            inserted.insert((IrSymbolKind::Form, item.name.clone()));
            target.forms.push(item);
        }
    }
    for item in source.functions {
        if can_insert_fallback_symbol(target, &item.name) {
            inserted.insert((IrSymbolKind::Function, item.name.clone()));
            target.functions.push(item);
        }
    }
    for origin in source.origins {
        let keep = inserted.contains(&(origin.kind, origin.name.clone()));
        if keep {
            target.origins.push(origin);
        }
    }
}

fn contains_symbol_name(module: &IrModule, name: &str) -> bool {
    module.enums.iter().any(|item| item.name == name)
        || module.type_aliases.iter().any(|item| item.name == name)
        || module.variables.iter().any(|item| item.name == name)
        || module.messages.iter().any(|item| item.name == name)
        || module.groups.iter().any(|item| item.name == name)
        || module.forms.iter().any(|item| item.name == name)
        || module.functions.iter().any(|item| item.name == name)
}

fn can_insert_fallback_symbol(module: &IrModule, name: &str) -> bool {
    if contains_symbol_name(module, name) {
        return false;
    }

    let mut prefix = name;
    while let Some((ancestor, _)) = prefix.rsplit_once('.') {
        if contains_non_group_symbol_name(module, ancestor) {
            return false;
        }
        prefix = ancestor;
    }
    true
}

fn contains_non_group_symbol_name(module: &IrModule, name: &str) -> bool {
    module.enums.iter().any(|item| item.name == name)
        || module.type_aliases.iter().any(|item| item.name == name)
        || module.variables.iter().any(|item| item.name == name)
        || module.messages.iter().any(|item| item.name == name)
        || module.forms.iter().any(|item| item.name == name)
        || module.functions.iter().any(|item| item.name == name)
}

#[cfg(test)]
mod tests {
    use super::{
        immediate_dynamic_messages, merge_module, merge_module_fallback, namespaced_module,
        project_locale_fallbacks, project_relative_source_path,
    };
    use linguini_config::{TypeScriptBundlerDynamicConfig, TypeScriptBundlerDynamicMode};
    use linguini_ir::lower_locale;
    use linguini_syntax::parse_locale;
    use std::collections::BTreeMap;

    #[test]
    fn root_dynamic_prefix_accepts_exact_top_level_leaf() {
        let messages = BTreeMap::from([("title".to_owned(), 0), ("main.title".to_owned(), 1)]);
        let dynamic = TypeScriptBundlerDynamicConfig {
            mode: TypeScriptBundlerDynamicMode::Bundle,
            allow: vec!["title".to_owned(), "main.title".to_owned()],
        };
        assert_eq!(
            immediate_dynamic_messages("", &dynamic, &messages),
            vec![("title", "title", 0)]
        );
    }

    #[cfg(unix)]
    #[test]
    fn project_source_path_rejects_literal_backslash_without_normalizing_identity() {
        let error = project_relative_source_path(
            std::path::Path::new("/project"),
            std::path::Path::new("/project/schema/bad\\name.lgs"),
        )
        .expect_err("literal backslash must be rejected");

        assert!(error.to_string().contains("literal backslash"));
        assert!(error.to_string().contains("bad\\name.lgs"));
    }

    #[test]
    fn module_merge_preserves_all_declaration_origins() {
        let mut target = namespaced_module(
            lower_locale(&parse_locale("section { first = First }\n").expect("first locale")),
            "shop",
        );
        let source = namespaced_module(
            lower_locale(&parse_locale("section { second = Second }\n").expect("second locale")),
            "shop",
        );

        merge_module(&mut target, source);

        assert_eq!(
            target
                .origins
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            [
                "shop.section",
                "shop.section.first",
                "shop.section",
                "shop.section.second",
            ]
        );
        assert_eq!(
            target
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section", "shop.section"]
        );
    }

    #[test]
    fn fallback_merge_keeps_only_inserted_symbol_provenance() {
        let mut target = namespaced_module(
            lower_locale(&parse_locale("present = Primary\n").expect("primary locale")),
            "shop",
        );
        let fallback = namespaced_module(
            lower_locale(
                &parse_locale("present = Default\nextra { nested = Fallback }\n")
                    .expect("fallback locale"),
            ),
            "shop",
        );

        merge_module_fallback(&mut target, fallback);

        assert_eq!(
            target
                .origins
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.present", "shop.extra", "shop.extra.nested"]
        );
        assert_eq!(
            target
                .messages
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.present", "shop.extra.nested"]
        );
        assert_eq!(
            target
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.extra"]
        );
    }

    #[test]
    fn fallback_merge_keeps_new_nested_groups_without_duplicate_parent_provenance() {
        let mut target = namespaced_module(
            lower_locale(&parse_locale("section { primary = Primary }\n").expect("primary")),
            "shop",
        );
        let fallback = namespaced_module(
            lower_locale(
                &parse_locale("section { primary = Default\n secondary { item = Fallback } }\n")
                    .expect("fallback"),
            ),
            "shop",
        );

        merge_module_fallback(&mut target, fallback);

        assert_eq!(
            target
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section", "shop.section.secondary"]
        );
        assert_eq!(
            target
                .messages
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section.primary", "shop.section.secondary.item"]
        );
        assert_eq!(
            target
                .origins
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            [
                "shop.section",
                "shop.section.primary",
                "shop.section.secondary",
                "shop.section.secondary.item",
            ]
        );
    }

    #[test]
    fn fallback_merge_blocks_cross_kind_parent_collision() {
        let mut target = namespaced_module(
            lower_locale(&parse_locale("section { title = Primary }\n").expect("primary")),
            "shop",
        );
        let fallback = namespaced_module(
            lower_locale(&parse_locale("section = Fallback\n").expect("fallback")),
            "shop",
        );

        merge_module_fallback(&mut target, fallback);

        assert_eq!(target.groups.len(), 1);
        assert_eq!(target.messages.len(), 1);
        assert_eq!(target.messages[0].name, "shop.section.title");
        assert_eq!(
            target
                .origins
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section", "shop.section.title"]
        );
    }

    #[test]
    fn fallback_merge_blocks_reverse_cross_kind_parent_collision() {
        let mut target = namespaced_module(
            lower_locale(&parse_locale("section = Primary\n").expect("primary")),
            "shop",
        );
        let fallback = namespaced_module(
            lower_locale(&parse_locale("section { child = Fallback }\n").expect("fallback")),
            "shop",
        );

        merge_module_fallback(&mut target, fallback);

        assert!(target.groups.is_empty());
        assert_eq!(
            target
                .messages
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section"]
        );
        assert_eq!(
            target
                .origins
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section"]
        );
    }

    #[test]
    fn project_fallbacks_follow_cldr_parents_and_likely_scripts() {
        let locales = ["en", "en-001", "en-AU", "zh-Hant", "zh-TW"]
            .map(str::to_owned)
            .to_vec();

        assert_eq!(
            project_locale_fallbacks(&locales, "en-AU", "en"),
            ["en-001", "en"]
        );
        assert_eq!(
            project_locale_fallbacks(&locales, "zh-TW", "en"),
            ["zh-Hant", "en"]
        );
    }

    #[test]
    fn project_fallbacks_match_canonical_aliases() {
        let locales = ["en", "he", "iw-IL"].map(str::to_owned).to_vec();

        assert_eq!(
            project_locale_fallbacks(&locales, "iw-IL", "en"),
            ["he", "en"]
        );
    }
}
