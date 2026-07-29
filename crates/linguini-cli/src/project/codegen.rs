use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use linguini_analyzer::DiagnosticSeverity;
use linguini_codegen_ts::{
    generate_typescript_project_files, TypeScriptFramework, TypeScriptGeneratedFile,
    TypeScriptLocaleModule, TypeScriptProjectOptions, TypeScriptWebOptions,
    ValidatedTypeScriptProject,
};
use linguini_config::{
    CanonicalMode, CookiePath, LinguiniConfig, LinkMode, LocalePrefixMode, SecurePolicy,
    TypeScriptTargetConfig,
};
use linguini_ir::{
    ensure_no_unresolved_references, lower_locale, lower_schema, qualify_module, IrModule,
    IrSymbolKind,
};

use crate::{CliError, CliResult};

use super::check::{check_project, reject_locale_files_without_schema_namespace};
use super::io::{path_for_output, read_project_config, render_file_diagnostics};
use super::output::{replace_owned_files, GeneratedFile, SafeOutputRoot};
use super::sources::{
    coverage_options, expected_locale_path, load_locale_sources, load_schema_sources, locale_index,
    schema_project_diagnostics,
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
    let files = generate_typescript_project_files(&project)
        .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;

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

fn legacy_web_codegen_options(config: &LinguiniConfig) -> TypeScriptWebOptions {
    let cookie = config.web.cookie.as_ref();
    let local_storage = config.web.local_storage.as_ref();
    let mut strategy = config
        .web
        .locale
        .sources
        .iter()
        .map(|source| match source {
            linguini_config::LocaleSource::Path => "url",
            linguini_config::LocaleSource::Cookie => "cookie",
            linguini_config::LocaleSource::LocalStorage => "localStorage",
            linguini_config::LocaleSource::AcceptLanguage => "header",
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    strategy.push("baseLocale".to_owned());

    TypeScriptWebOptions {
        strategy,
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
        global_variable_name: None,
        prefix_default_locale: config.web.routing.locale_prefix == LocalePrefixMode::Always,
        base_path: String::new(),
        trailing_slash: "ignore".to_owned(),
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
            merge_module(
                &mut locale_ir,
                namespaced_module(lower_locale(&locale_file.ast), namespace),
            );
        }

        if locale != config.project.default_locale.as_str() {
            if let Some(default_locale_file) = locale_index.get(&default_key) {
                merge_module_fallback(
                    &mut locale_ir,
                    namespaced_module(lower_locale(&default_locale_file.ast), namespace),
                );
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
            .filter(|diagnostic| diagnostic.span.source == source.ast.span.source)
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
    target.forms.extend(source.forms);
    target.functions.extend(source.functions);
    target.origins.extend(source.origins);
}

pub(super) fn merge_module_fallback(target: &mut IrModule, source: IrModule) {
    let mut inserted = BTreeSet::new();
    for item in source.enums {
        if !target
            .enums
            .iter()
            .any(|existing| existing.name == item.name)
        {
            inserted.insert((IrSymbolKind::Enum, item.name.clone()));
            target.enums.push(item);
        }
    }
    for item in source.type_aliases {
        if !target
            .type_aliases
            .iter()
            .any(|existing| existing.name == item.name)
        {
            inserted.insert((IrSymbolKind::TypeAlias, item.name.clone()));
            target.type_aliases.push(item);
        }
    }
    for item in source.messages {
        if !target
            .messages
            .iter()
            .any(|existing| existing.name == item.name)
        {
            inserted.insert((IrSymbolKind::Message, item.name.clone()));
            target.messages.push(item);
        }
    }
    for item in source.variables {
        if !target
            .variables
            .iter()
            .any(|existing| existing.name == item.name)
        {
            inserted.insert((IrSymbolKind::Variable, item.name.clone()));
            target.variables.push(item);
        }
    }
    for item in source.forms {
        if !target
            .forms
            .iter()
            .any(|existing| existing.name == item.name)
        {
            inserted.insert((IrSymbolKind::Form, item.name.clone()));
            target.forms.push(item);
        }
    }
    for item in source.functions {
        if !target
            .functions
            .iter()
            .any(|existing| existing.name == item.name)
        {
            inserted.insert((IrSymbolKind::Function, item.name.clone()));
            target.functions.push(item);
        }
    }
    for origin in source.origins {
        let keep = inserted.contains(&(origin.kind, origin.name.clone()))
            || (origin.kind == IrSymbolKind::Group
                && inserted
                    .iter()
                    .any(|(_, name)| is_descendant(name, &origin.name)));
        if keep {
            target.origins.push(origin);
        }
    }
}

fn is_descendant(name: &str, parent: &str) -> bool {
    name.strip_prefix(parent)
        .is_some_and(|suffix| suffix.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::{merge_module, merge_module_fallback, namespaced_module};
    use linguini_ir::lower_locale;
    use linguini_syntax::parse_locale;

    #[test]
    fn module_merge_preserves_all_declaration_origins() {
        let mut target = namespaced_module(
            lower_locale(&parse_locale("first = First\n").expect("first locale")),
            "shop",
        );
        let source = namespaced_module(
            lower_locale(&parse_locale("second = Second\n").expect("second locale")),
            "shop",
        );

        merge_module(&mut target, source);

        assert_eq!(
            target
                .origins
                .iter()
                .map(|origin| origin.name.as_str())
                .collect::<Vec<_>>(),
            ["shop.first", "shop.second"]
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
    }
}
