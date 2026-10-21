//! Locale namespace and barrel emission for canonical JavaScript.

use crate::ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport, EcmaSource,
    EcmaStatement,
};
use linguini_ir::IrModule;

use super::emit::{
    emit_forms, emit_local_functions, emit_locale_enum_jsdoc_types, emit_messages,
    emit_schema_jsdoc_types, emit_variables, module_imports, ModuleExports,
};
use super::message::ordered_sources;
use super::names::{import_path_for_target, safe_file_stem, safe_identifier};
use super::{
    locale_global_import, locale_has_globals, locale_module_for_schema, locale_runtime_import,
    locale_without_globals, namespace_module, project_locale_options, root_module,
    root_module_with_locale_items, top_level_namespaces, visible_schema, EcmaCodegenError,
    EcmaLocaleArtifact, EcmaLocaleArtifactKind, EcmaOptions, ValidatedEcmaProject,
};

/// One source-mapped locale namespace or barrel module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledEcmaLocaleModule {
    pub artifact: EcmaLocaleArtifact,
    pub code: String,
    pub source_map: String,
}

pub fn compile_locale_artifact_module(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaLocaleArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledEcmaLocaleModule, EcmaCodegenError> {
    compile_locale_artifact_module_impl(project, artifact, sources, EcmaModuleKind::JavaScript)
}

fn compile_locale_artifact_module_impl(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaLocaleArtifact,
    sources: &[EcmaSource],
    target: EcmaModuleKind,
) -> Result<CompiledEcmaLocaleModule, EcmaCodegenError> {
    let module = locale_artifact_module(project, artifact, target)?;
    let source_records = ordered_sources(&artifact.source_ids, sources)?;
    let rendered = module.render(&source_records);
    Ok(CompiledEcmaLocaleModule {
        artifact: artifact.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
    })
}

pub(super) struct LocaleModuleRequest<'a> {
    pub schema: &'a IrModule,
    pub import_locale: &'a IrModule,
    pub locale: &'a IrModule,
    pub options: &'a EcmaOptions,
    pub namespaces: &'a [String],
    pub shared_import_path: &'a str,
    pub runtime_import_path: &'a str,
    pub global_import_path: Option<&'a str>,
    pub namespace_alias: Option<&'a str>,
    pub output: EcmaModuleOutput,
}

pub(super) fn locale_artifact_module(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaLocaleArtifact,
    target: EcmaModuleKind,
) -> Result<EcmaModule, EcmaCodegenError> {
    let configured_locale = project
        .locales
        .iter()
        .find(|locale| locale.locale == artifact.locale)
        .ok_or_else(|| EcmaCodegenError::UnknownLocale {
            locale: artifact.locale.clone(),
        })?;
    let options = project_locale_options(&configured_locale.locale, &project.options)?;
    let schema = visible_schema(project.schema, &options);
    let visible_locale = locale_module_for_schema(&configured_locale.module, &schema);
    let namespaces = top_level_namespaces(&schema);
    let has_globals = locale_has_globals(&visible_locale);
    let (
        schema,
        import_locale,
        locale,
        request_namespaces,
        shared_path,
        runtime_path,
        globals_path,
        alias,
    ) = match &artifact.kind {
        EcmaLocaleArtifactKind::Namespace { namespace } => {
            if !namespaces.contains(namespace) {
                return Err(EcmaCodegenError::UnknownLocaleNamespace {
                    locale: artifact.locale.clone(),
                    namespace: namespace.clone(),
                });
            }
            let schema = namespace_module(&schema, namespace);
            let import_locale = namespace_module(&visible_locale, namespace);
            let locale = locale_without_globals(&import_locale);
            (
                schema,
                import_locale,
                locale,
                Vec::new(),
                "../../shared".to_owned(),
                "./_runtime".to_owned(),
                has_globals.then(|| "./_globals".to_owned()),
                Some(namespace.as_str()),
            )
        }
        EcmaLocaleArtifactKind::Barrel => {
            let (schema, import_locale) = if namespaces.is_empty() {
                (schema, visible_locale)
            } else {
                (
                    root_module(&schema),
                    root_module_with_locale_items(&visible_locale),
                )
            };
            let locale = locale_without_globals(&import_locale);
            (
                schema,
                import_locale,
                locale,
                namespaces,
                "../shared".to_owned(),
                format!("./{}/_runtime", artifact.locale),
                has_globals.then(|| format!("./{}/_globals", artifact.locale)),
                None,
            )
        }
    };
    let output_path = artifact
        .module_path
        .strip_suffix(".ts")
        .map_or_else(|| artifact.module_path.clone(), |path| format!("{path}.js"));
    let module = locale_module(LocaleModuleRequest {
        schema: &schema,
        import_locale: &import_locale,
        locale: &locale,
        options: &options,
        namespaces: &request_namespaces,
        shared_import_path: &shared_path,
        runtime_import_path: &runtime_path,
        global_import_path: globals_path.as_deref(),
        namespace_alias: alias,
        output: EcmaModuleOutput::new(target, output_path, None),
    });
    Ok(module)
}

pub(super) fn locale_module(request: LocaleModuleRequest<'_>) -> EcmaModule {
    let target = request.output.target();
    let shared_import_path = import_path_for_target(request.shared_import_path, target);
    let runtime_import_path = import_path_for_target(request.runtime_import_path, target);
    let mut imports = Vec::new();
    for namespace in request.namespaces {
        let identifier = safe_identifier(namespace);
        let file_stem = safe_file_stem(namespace);
        imports.push(EcmaImport::named(
            import_path_for_target(&format!("./{}/{file_stem}", request.options.locale), target),
            vec![EcmaNamedImport::new(&identifier, &identifier)],
        ));
    }
    imports.extend(module_imports(
        request.schema,
        request.locale,
        request.options,
        &shared_import_path,
    ));
    if let Some(runtime_import) = locale_runtime_import(
        request.schema,
        request.locale,
        request.options,
        &runtime_import_path,
    ) {
        imports.push(runtime_import);
    }
    if let Some(global_import_path) = request.global_import_path {
        if let Some(global_import) = locale_global_import(
            request.schema,
            request.locale,
            request.import_locale,
            &import_path_for_target(global_import_path, target),
        ) {
            imports.push(global_import);
        }
    }

    let mut body = String::new();
    emit_schema_jsdoc_types(request.schema, &shared_import_path, &mut body);
    {
        emit_locale_enum_jsdoc_types(request.schema, request.locale, &mut body);
    }
    for namespace in request.namespaces {
        body.push_str(&crate::templates::render(
            "module/exports",
            minijinja::context! { names => [safe_identifier(namespace)], blank_line => true },
        ));
    }
    emit_variables(request.locale, request.options, false, &mut body);
    emit_forms(
        request.schema,
        request.locale,
        request.options,
        false,
        &mut body,
    );
    emit_local_functions(
        request.schema,
        request.locale,
        request.options,
        false,
        &mut body,
    );
    let exports = emit_messages(request.schema, request.locale, request.options, &mut body);
    emit_locale_default(&exports, request.namespaces, &mut body);
    if let Some(namespace_alias) = request.namespace_alias {
        let identifier = safe_identifier(namespace_alias);
        let alias_is_exported = exports
            .top_level
            .iter()
            .chain(exports.groups.iter())
            .any(|export| export == &identifier);
        if !alias_is_exported {
            body.push('\n');
            body.push_str(&crate::templates::render("module/constant", minijinja::context! { exported => true, name => identifier, value => "lgl", blank_line => false }));
        }
    }

    let mut module = EcmaModule::new(request.output);
    module.extend_imports(imports);
    if !body.trim().is_empty() {
        let span = request.locale.origins().first().map(|origin| origin.span);
        module.push_statement(EcmaStatement::value(body, span));
    }
    module
}

fn emit_locale_default(exports: &ModuleExports, namespaces: &[String], output: &mut String) {
    let names = exports
        .top_level
        .iter()
        .chain(&exports.groups)
        .cloned()
        .chain(namespaces.iter().map(|name| safe_identifier(name)))
        .collect::<Vec<_>>();
    output.push_str(&crate::templates::render(
        "module/locale_default",
        minijinja::context! { names },
    ));
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    use crate::ecmascript::EcmaSource;
    use crate::generate_schema_files;

    use super::compile_locale_artifact_module;
    use crate::module::{
        generate_project_files, EcmaLocaleArtifactKind, EcmaLocaleModule, EcmaProjectOptions,
        ValidatedEcmaProject,
    };

    #[test]
    fn locale_artifacts_share_target_aware_namespace_and_barrel_emission() {
        let schema_text = "root(name: String)\naccount {\n  label\n  personalized(name: String)\n  nested {\n    status\n  }\n}\n";
        let base_text = "root = Hello {name}\naccount {\n  label = Account\n  personalized = Welcome {name}\n  nested {\n    status = Ready\n  }\n}\n";
        let regional_text = "root = Howdy {name}\n";
        let schema =
            lower_schema(&parse_schema_in(schema_text, SourceId(101)).expect("schema parses"));
        let locales = [
            EcmaLocaleModule {
                locale: "en".to_owned(),
                module: lower_locale(
                    &parse_locale_in(base_text, SourceId(102)).expect("base locale parses"),
                ),
            },
            EcmaLocaleModule {
                locale: "en-US".to_owned(),
                module: lower_locale(
                    &parse_locale_in(regional_text, SourceId(103)).expect("regional locale parses"),
                ),
            },
        ];
        let project_options = EcmaProjectOptions {
            base_locale: Some("en".to_owned()),
            ..EcmaProjectOptions::default()
        };
        let project = ValidatedEcmaProject::try_new(&schema, &locales, &project_options)
            .expect("validated project");
        let artifacts = project.locale_artifacts().expect("locale artifacts");
        assert_eq!(artifacts.len(), 4);
        assert_eq!(
            artifacts,
            project.locale_artifacts().expect("repeat artifacts")
        );
        let namespace = artifacts
            .iter()
            .find(|artifact| {
                artifact.locale == "en-US"
                    && matches!(
                        &artifact.kind,
                        EcmaLocaleArtifactKind::Namespace { namespace }
                            if namespace == "account"
                    )
            })
            .expect("regional account artifact");
        let barrel = artifacts
            .iter()
            .find(|artifact| {
                artifact.locale == "en-US" && artifact.kind == EcmaLocaleArtifactKind::Barrel
            })
            .expect("regional barrel artifact");
        assert_eq!(namespace.module_path, "locales/en-US/account.js");
        assert_eq!(
            namespace.source_ids,
            [SourceId(101), SourceId(102), SourceId(103)]
        );
        assert_eq!(barrel.module_path, "locales/en-US.js");
        assert_eq!(
            barrel.source_ids,
            [SourceId(101), SourceId(102), SourceId(103)]
        );
        let sources = [
            EcmaSource::new(SourceId(101), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(102), "en.lgl", base_text),
            EcmaSource::new(SourceId(103), "en-US.lgl", regional_text),
        ];
        let project_files = generate_project_files(&project).expect("project files");

        for artifact in [namespace, barrel] {
            let javascript = compile_locale_artifact_module(&project, artifact, &sources)
                .expect("JavaScript locale artifact");
            let project_file = project_files
                .iter()
                .find(|file| file.path == artifact.module_path)
                .expect("project locale module");
            let map_name = artifact
                .module_path
                .rsplit_once('/')
                .map_or(artifact.module_path.as_str(), |(_, name)| name);
            assert_eq!(
                javascript
                    .code
                    .strip_suffix(&format!("//# sourceMappingURL={map_name}.map\n"))
                    .expect("TypeScript source-map trailer"),
                project_file.contents
            );
            assert!(!javascript.code.contains("import type"));
            assert!(!javascript.code.contains("...__lgl_args:"));
            assert!(!javascript.code.contains("): string {"));
            assert!(!javascript.code.contains(" as const"));
            assert!(javascript.code.contains(".js\";"));
            assert!(javascript.source_map.contains("schema.lgs"));
        }
        let namespace_javascript = compile_locale_artifact_module(&project, namespace, &sources)
            .expect("JavaScript namespace");
        assert!(namespace_javascript.code.contains(
            "/** @type {((name: string) => string) & ((args: { name: string }) => string)} */"
        ));
        assert!(namespace_javascript
            .code
            .contains("export const account = {"));
        let barrel_javascript =
            compile_locale_artifact_module(&project, barrel, &sources).expect("JavaScript barrel");
        assert!(barrel_javascript
            .code
            .contains("import { account } from \"./en-US/account.js\";"));
        assert!(barrel_javascript.code.contains("export default lgl;"));

        let shared_javascript = generate_schema_files(&schema)
            .into_iter()
            .find(|file| file.path == "shared.js")
            .expect("JavaScript shared module");
        let snapshot_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-locale");
        let javascript_locale_root = snapshot_root.join("locales/en-US");
        let snapshots = [
            (
                snapshot_root.join("shared.js"),
                shared_javascript.contents.as_str(),
            ),
            (
                javascript_locale_root.join("account.js"),
                namespace_javascript.code.as_str(),
            ),
            (
                javascript_locale_root.join("account.js.map"),
                namespace_javascript.source_map.as_str(),
            ),
            (
                snapshot_root.join("locales/en-US.js"),
                barrel_javascript.code.as_str(),
            ),
            (
                snapshot_root.join("locales/en-US.js.map"),
                barrel_javascript.source_map.as_str(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::create_dir_all(&javascript_locale_root)
                .expect("create JavaScript locale snapshot directory");
            for (path, contents) in &snapshots {
                std::fs::write(path, contents).expect("write locale snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read locale snapshot")
            );
        }
    }
}
