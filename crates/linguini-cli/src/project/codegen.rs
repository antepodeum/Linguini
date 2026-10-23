use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use linguini_analyzer::{
    ApplicationBindingProvenance, ApplicationDynamicReferenceKind, ApplicationImportBinding,
    ApplicationImportBindingId, ApplicationReferenceKind, ApplicationUsage, DiagnosticSeverity,
};
use linguini_cldr::{canonicalize_locale, locale_fallback_chain};
use linguini_codegen_ecmascript::{
    compile_ecmascript_project_files, rebase_map_sources, EcmaGeneratedFile, EcmaLocaleModule,
    EcmaProjectOptions, EcmaSource, ValidatedEcmaProject,
};
use linguini_config::{
    discover_application_source_files_with_fields, LinguiniConfig, TypeScriptBundlerConfig,
    TypeScriptTargetConfig,
};
use linguini_ir::{
    ensure_no_unresolved_references, lower_locale, lower_schema, qualify_module, IrModule,
    IrModuleBuilder, IrSymbolKind, LocaleIr, SchemaIr,
};
use linguini_syntax::SourceId;
use sha2::{Digest, Sha256};

use super::modules::{Framework, ModuleCapabilities};
use crate::{CliError, CliResult, DiagnosticFormat};

use super::check::{check_project_with_options, reject_locale_files_without_schema_namespace};
use super::io::{
    canonical_project_root, path_for_output, read_file, read_project_config,
    render_file_diagnostics,
};
use super::output::{
    replace_owned_files, replace_owned_files_in_scope, GeneratedFile, SafeOutputRoot,
};
use super::sources::{
    coverage_options, expected_locale_path, load_locale_sources, load_schema_sources, locale_index,
    schema_project_diagnostics,
};
use super::{ParsedLocaleSource, ParsedSchemaSource};

pub fn build_project(root: &Path) -> CliResult<String> {
    let modules = super::modules::read_module_plan(root, None, false)?;
    build_project_with_emissions(root, false, DiagnosticFormat::Human, &[], &[], &modules)
}

pub(crate) fn build_project_with_emissions(
    root: &Path,
    deny_warnings: bool,
    format: DiagnosticFormat,
    emit_dirs: &[PathBuf],
    emit_declaration_dirs: &[PathBuf],
    modules: &linguini_codegen::ModulePlan,
) -> CliResult<String> {
    let canonical_root = canonical_project_root(root)?;
    let root = canonical_root.as_path();
    let check_output = check_project_with_options(root, deny_warnings, format)?;
    let config = read_project_config(root)?;
    let codegen_output =
        generate_project(root, &config, emit_dirs, emit_declaration_dirs, modules)?;

    if format == DiagnosticFormat::Human {
        Ok(format!("{check_output}{codegen_output}build: ok\n"))
    } else {
        Ok(check_output)
    }
}

fn generate_project(
    root: &Path,
    config: &LinguiniConfig,
    emit_dirs: &[PathBuf],
    emit_declaration_dirs: &[PathBuf],
    modules: &linguini_codegen::ModulePlan,
) -> CliResult<String> {
    if config.targets().typescript().is_none() && !modules.modules().is_empty() {
        return Err(CliError::Diagnostics(
            "module plan requires an ECMAScript codegen target\n".to_owned(),
        ));
    }
    let Some(target) = config.targets().typescript() else {
        if !emit_dirs.is_empty() || !emit_declaration_dirs.is_empty() {
            return Err(CliError::Diagnostics(
                "additional emission requires a configured codegen target\n".to_owned(),
            ));
        }
        return Ok("codegen targets: none\n".to_owned());
    };

    generate_target(
        root,
        config,
        target,
        emit_dirs,
        emit_declaration_dirs,
        modules,
    )
}

fn generate_target(
    root: &Path,
    config: &LinguiniConfig,
    target: &TypeScriptTargetConfig,
    emit_dirs: &[PathBuf],
    emit_declaration_dirs: &[PathBuf],
    modules: &linguini_codegen::ModulePlan,
) -> CliResult<String> {
    if !emit_declaration_dirs.is_empty() && !target.declaration() {
        return Err(CliError::Diagnostics(
            "--emit-declaration-dir requires declarations to be enabled\n".to_owned(),
        ));
    }
    let outputs = codegen_output_roots(root, config, target, emit_dirs, emit_declaration_dirs)?;
    let output_paths = outputs
        .iter()
        .map(|output| output.directory.clone())
        .collect::<Vec<_>>();
    let schema_files = load_schema_sources(root, config)?;
    let schema_namespaces: BTreeSet<_> = schema_files
        .iter()
        .map(|schema| schema.file.namespace.clone())
        .collect();
    ensure_schema_project_valid(root, &schema_files)?;
    let schema = merge_schema_ir(&schema_files)?;
    let locale_files = load_locale_sources(root, config)?;
    let locale_index = locale_index(&locale_files)?;

    reject_locale_files_without_schema_namespace(root, &schema_namespaces, &locale_files)?;

    let mut locales = Vec::new();
    for locale in config.project().locales() {
        let locale_ir = build_locale_ir(root, config, &schema_files, &locale_index, locale)?;
        ensure_locale_ir_resolves(&schema, &locale_ir, locale)?;
        locales.push(EcmaLocaleModule {
            locale: locale.to_owned(),
            module: locale_ir,
        });
    }

    let options = EcmaProjectOptions {
        declaration: target.declaration(),
        gitignore: target.gitignore(),
        tree_shaking: target.tree_shaking(),
        included_messages: target.messages().to_vec(),
        base_locale: Some(config.project().default_locale().to_owned()),
    };
    let capabilities = ModuleCapabilities::compile(modules, config.project().name())?;
    let web_options = &capabilities.web;
    let project = ValidatedEcmaProject::try_new(&schema, &locales, &options)
        .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
    let sources = ecma_sources(root, &schema_files, &locale_files)?;
    let mut plans = Vec::new();
    for destination in outputs {
        let CodegenOutput {
            directory,
            output,
            declarations_only,
        } = destination;
        let project_sources =
            rebase_map_sources(&directory, "index.js.map", &sources).map_err(|reason| {
                CliError::Diagnostics(format!(
                    "cannot rebase project source-map paths: {reason}\n"
                ))
            })?;
        let mut files = compile_ecmascript_project_files(&project, &project_sources)
            .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
        if let Some(web) = web_options {
            files.extend(
                linguini_codegen_ecmascript_web::generate_project_files(
                    &linguini_codegen_ecmascript_web::WebProjectOptions {
                        web: web.clone(),
                        declaration: options.declaration,
                    },
                    true,
                )
                .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?,
            );
        }
        match capabilities.framework {
            Some(Framework::Svelte) => {
                files.extend(linguini_codegen_svelte::generate_project_files(
                    &linguini_codegen_svelte::SvelteProjectOptions {
                        web: web_options.clone(),
                        declaration: options.declaration,
                        host: Default::default(),
                    },
                    true,
                ))
            }
            Some(Framework::SvelteKit) => {
                files.extend(linguini_codegen_sveltekit::generate_project_files(
                    &linguini_codegen_sveltekit::SvelteKitProjectOptions {
                        web: web_options.clone(),
                        declaration: options.declaration,
                    },
                    true,
                ))
            }
            _ => {}
        }
        if declarations_only {
            files.retain(|file| is_declaration_artifact(Path::new(&file.path)));
        } else {
            files.extend(generate_bundler_files(
                root,
                &project,
                &sources,
                CodegenDestination {
                    settings: target,
                    directory: &directory,
                    all_outputs: &output_paths,
                },
                config.project().locales(),
                config.project().default_locale(),
                &capabilities,
            )?);
        }
        plans.push((output, files, declarations_only));
    }
    let mut output = String::new();
    for (destination, files, declarations_only) in plans {
        output.push_str(&write_codegen_tree(
            root,
            &destination,
            &files,
            declarations_only,
        )?);
    }
    Ok(output)
}

/// Output paths are disjoint and use the same containment and symlink rules as the configured root.
fn codegen_output_roots(
    root: &Path,
    config: &LinguiniConfig,
    target: &TypeScriptTargetConfig,
    emit_dirs: &[PathBuf],
    emit_declaration_dirs: &[PathBuf],
) -> CliResult<Vec<CodegenOutput>> {
    let mut outputs: Vec<CodegenOutput> = Vec::new();
    let directories = std::iter::once((Path::new(target.out()), false))
        .chain(emit_dirs.iter().map(|path| (path.as_path(), false)))
        .chain(
            emit_declaration_dirs
                .iter()
                .map(|path| (path.as_path(), true)),
        );
    for (directory, declarations_only) in directories {
        let output = SafeOutputRoot::new(
            root,
            directory,
            &[
                Path::new(config.paths().schema()),
                Path::new(config.paths().locale()),
            ],
        )?;
        if outputs
            .iter()
            .any(|existing| existing.output.path() == output.path())
        {
            continue;
        }
        if outputs.iter().any(|existing| {
            existing.output.path().starts_with(output.path())
                || output.path().starts_with(existing.output.path())
        }) {
            return Err(CliError::Diagnostics(
                "codegen output directories must not overlap\n".to_owned(),
            ));
        }
        let relative = output
            .path()
            .strip_prefix(root)
            .expect("validated project output root")
            .to_str()
            .ok_or_else(|| {
                CliError::Diagnostics("codegen output path must be valid UTF-8\n".to_owned())
            })?
            .replace('\\', "/");
        outputs.push(CodegenOutput {
            directory: relative,
            output,
            declarations_only,
        });
    }
    Ok(outputs)
}

struct CodegenOutput {
    directory: String,
    output: SafeOutputRoot,
    declarations_only: bool,
}

fn is_declaration_artifact(path: &Path) -> bool {
    path.to_str()
        .is_some_and(|path| path.ends_with(".d.ts") || path.ends_with(".d.ts.map"))
}

struct CodegenDestination<'a> {
    settings: &'a TypeScriptTargetConfig,
    directory: &'a str,
    all_outputs: &'a [String],
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
                source.ast.span().source,
                project_relative_source_path(root, &source.file.path)?,
                source.source.clone(),
            ))
        })
        .chain(locale_files.iter().map(|source| {
            Ok(EcmaSource::new(
                source.ast.span().source,
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
    project: &ValidatedEcmaProject<'_>,
    sources: &[EcmaSource],
    destination: CodegenDestination<'_>,
    configured_locales: &[String],
    base_locale: &str,
    capabilities: &ModuleCapabilities,
) -> CliResult<Vec<EcmaGeneratedFile>> {
    use linguini_codegen_vite::{
        LocaleLoading, ViteApplicationPlan, ViteEmissionOptions, VitePlan,
    };
    let plan =
        VitePlan::new(project).map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
    let applications = if let Some(bundler) = destination.settings.bundler() {
        let applications = scan_bundler_applications(
            root,
            destination.all_outputs,
            bundler,
            &plan.message_arities(),
        )?;
        let mut runtime_helpers = BTreeMap::new();
        if capabilities.framework.is_some() {
            runtime_helpers =
                linguini_codegen_svelte::runtime_helper_descriptors(capabilities.web.as_ref());
        }
        Some(ViteApplicationPlan {
            locale_loading: match bundler.locale_loading().as_str() {
                "dynamic" => LocaleLoading::Dynamic,
                _ => LocaleLoading::Eager,
            },
            applications,
            runtime_helpers,
        })
    } else {
        None
    };
    plan.generate_files(&ViteEmissionOptions {
        output_root: destination.directory,
        sources,
        configured_locales,
        base_locale,
        applications: applications.as_ref(),
    })
    .map_err(|error| CliError::Diagnostics(format!("{error}\n")))
}

const APPLICATION_SOURCE_ID_BASE: u32 = 0x8000_0000;

fn scan_bundler_applications(
    root: &Path,
    output_roots: &[String],
    config: &TypeScriptBundlerConfig,
    messages: &BTreeMap<String, usize>,
) -> CliResult<BTreeMap<String, serde_json::Value>> {
    validate_dynamic_allow_entries(config.dynamic(), messages)?;
    let mut exclude = config.exclude().to_vec();
    exclude.extend_from_slice(output_roots);
    let paths = discover_application_source_files_with_fields(
        root,
        config.sources(),
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
                config.dynamic(),
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
    if dynamic.mode() != linguini_config::TypeScriptBundlerDynamicMode::Bundle {
        return Ok(());
    }
    let unknown = dynamic
        .allow()
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
        if dynamic.mode() == linguini_config::TypeScriptBundlerDynamicMode::Error {
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
            if !dynamic.allow().iter().any(|allowed| allowed == message) {
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

fn build_locale_ir(
    root: &Path,
    config: &LinguiniConfig,
    schema_files: &[ParsedSchemaSource],
    locale_index: &BTreeMap<(String, String), &ParsedLocaleSource>,
    locale: &str,
) -> CliResult<LocaleIr> {
    let mut locale_ir = IrModule::default();

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

        for fallback_locale in project_locale_fallbacks(
            config.project().locales(),
            locale,
            config.project().default_locale(),
        ) {
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

fn write_codegen_tree(
    root: &Path,
    out_dir: &SafeOutputRoot,
    files: &[EcmaGeneratedFile],
    declarations_only: bool,
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

    if declarations_only {
        replace_owned_files_in_scope(out_dir, &generated, is_declaration_artifact)?;
    } else {
        replace_owned_files(out_dir, &generated)?;
    }
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
        let dynamic = TypeScriptBundlerDynamicConfig::try_new(
            TypeScriptBundlerDynamicMode::Bundle,
            vec!["title".to_owned(), "main.title".to_owned()],
        )
        .expect("valid dynamic policy");
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
            lower_locale(&parse_locale("other { second = Second }\n").expect("second locale")),
            "shop",
        );

        merge_module(&mut target, source).expect("disjoint fixture merge");

        assert_eq!(
            target
                .origins()
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            [
                "shop.section",
                "shop.section.first",
                "shop.other",
                "shop.other.second",
            ]
        );
        assert_eq!(
            target
                .groups()
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section", "shop.other"]
        );
    }

    #[test]
    fn module_merge_rejects_duplicate_symbols_instead_of_duplicating_them() {
        let target = namespaced_module(
            lower_locale(&parse_locale("section { first = First }\n").expect("first locale")),
            "shop",
        );
        let conflicting = namespaced_module(
            lower_locale(&parse_locale("section { second = Second }\n").expect("second locale")),
            "shop",
        );

        let mut merged = target;
        let error = merge_module(&mut merged, conflicting)
            .expect_err("duplicate group symbols must fail visibly");
        assert!(error.to_string().contains("duplicate group symbol"));
        assert!(error.to_string().contains("shop.section"));
        // The rejected source contributes nothing to the target.
        assert_eq!(merged.groups().len(), 1);
        assert!(merged
            .messages()
            .iter()
            .all(|message| message.name != "shop.section.second"));
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

        merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

        assert_eq!(
            target
                .origins()
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.present", "shop.extra", "shop.extra.nested"]
        );
        assert_eq!(
            target
                .messages()
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.present", "shop.extra.nested"]
        );
        assert_eq!(
            target
                .groups()
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

        merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

        assert_eq!(
            target
                .groups()
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section", "shop.section.secondary"]
        );
        assert_eq!(
            target
                .messages()
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section.primary", "shop.section.secondary.item"]
        );
        assert_eq!(
            target
                .origins()
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

        merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

        assert_eq!(target.groups().len(), 1);
        assert_eq!(target.messages().len(), 1);
        assert_eq!(target.messages()[0].name, "shop.section.title");
        assert_eq!(
            target
                .origins()
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

        merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

        assert!(target.groups().is_empty());
        assert_eq!(
            target
                .messages()
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.section"]
        );
        assert_eq!(
            target
                .origins()
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
