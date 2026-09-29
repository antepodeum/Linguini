//! Locale namespace and barrel emission for TypeScript and checked JavaScript.

use crate::ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaScriptTarget, EcmaSource,
    EcmaStatement,
};
use linguini_ir::IrModule;

use super::emit::{
    emit_forms_for_target, emit_local_functions_for_target, emit_locale_enum_jsdoc_types,
    emit_locale_enum_types, emit_messages_for_target, emit_schema_types_for_target,
    emit_variables_for_target, module_imports, ModuleExports,
};
use super::message::ordered_sources;
use super::names::{import_path_for_target, safe_file_stem, safe_identifier};
use super::{
    locale_global_import, locale_has_globals, locale_module_for_schema, locale_runtime_import,
    locale_without_globals, namespace_module, project_locale_options, root_module,
    root_module_with_locale_items, top_level_namespaces, visible_schema, TypeScriptCodegenError,
    TypeScriptLocaleArtifact, TypeScriptLocaleArtifactKind, TypeScriptOptions,
    ValidatedTypeScriptProject,
};

/// One source-mapped locale namespace or barrel module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTypeScriptLocaleModule {
    pub artifact: TypeScriptLocaleArtifact,
    pub code: String,
    pub source_map: String,
}

pub type CompiledJavaScriptLocaleModule = CompiledTypeScriptLocaleModule;

pub fn compile_typescript_locale_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptLocaleModule, TypeScriptCodegenError> {
    compile_locale_artifact_module(project, artifact, sources, EcmaScriptTarget::TypeScript)
}

pub fn compile_javascript_locale_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledJavaScriptLocaleModule, TypeScriptCodegenError> {
    compile_locale_artifact_module(project, artifact, sources, EcmaScriptTarget::JavaScript)
}

fn compile_locale_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleArtifact,
    sources: &[EcmaSource],
    target: EcmaScriptTarget,
) -> Result<CompiledTypeScriptLocaleModule, TypeScriptCodegenError> {
    let configured_locale = project
        .locales
        .iter()
        .find(|locale| locale.locale == artifact.locale)
        .ok_or_else(|| TypeScriptCodegenError::UnknownLocale {
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
        TypeScriptLocaleArtifactKind::Namespace { namespace } => {
            if !namespaces.contains(namespace) {
                return Err(TypeScriptCodegenError::UnknownLocaleNamespace {
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
        TypeScriptLocaleArtifactKind::Barrel => {
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
    let output_path = match target {
        EcmaScriptTarget::TypeScript => artifact.module_path.clone(),
        EcmaScriptTarget::JavaScript => artifact
            .module_path
            .strip_suffix(".ts")
            .map_or_else(|| artifact.module_path.clone(), |path| format!("{path}.js")),
    };
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
    let source_records = ordered_sources(&artifact.source_ids, sources)?;
    let rendered = module.render(&source_records);
    Ok(CompiledTypeScriptLocaleModule {
        artifact: artifact.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
    })
}

pub(super) struct LocaleModuleRequest<'a> {
    pub schema: &'a IrModule,
    pub import_locale: &'a IrModule,
    pub locale: &'a IrModule,
    pub options: &'a TypeScriptOptions,
    pub namespaces: &'a [String],
    pub shared_import_path: &'a str,
    pub runtime_import_path: &'a str,
    pub global_import_path: Option<&'a str>,
    pub namespace_alias: Option<&'a str>,
    pub output: EcmaModuleOutput,
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
            request.import_locale,
            &import_path_for_target(global_import_path, target),
        ) {
            imports.push(global_import);
        }
    }

    let mut body = String::new();
    emit_schema_types_for_target(request.schema, &shared_import_path, target, &mut body);
    if target.is_typescript() {
        emit_locale_enum_types(request.schema, request.locale, &mut body);
    } else {
        emit_locale_enum_jsdoc_types(request.schema, request.locale, &mut body);
    }
    for namespace in request.namespaces {
        body.push_str(&format!("export {{ {} }};\n\n", safe_identifier(namespace)));
    }
    emit_variables_for_target(request.locale, request.options, target, false, &mut body);
    emit_forms_for_target(request.locale, request.options, target, false, &mut body);
    emit_local_functions_for_target(request.locale, request.options, target, false, &mut body);
    let exports = emit_messages_for_target(
        request.schema,
        request.locale,
        request.options,
        target,
        &mut body,
    );
    emit_locale_default(
        &exports,
        request.namespaces,
        target.is_typescript(),
        &mut body,
    );
    if let Some(namespace_alias) = request.namespace_alias {
        let identifier = safe_identifier(namespace_alias);
        let alias_is_exported = exports
            .top_level
            .iter()
            .chain(exports.groups.iter())
            .any(|export| export == &identifier);
        if !alias_is_exported {
            body.push_str(&format!("\nexport const {identifier} = lgl;\n"));
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

fn emit_locale_default(
    exports: &ModuleExports,
    namespaces: &[String],
    typescript: bool,
    output: &mut String,
) {
    output.push_str("const lgl = {\n");
    for name in exports.top_level.iter().chain(exports.groups.iter()) {
        output.push_str(&format!("  {name},\n"));
    }
    for namespace in namespaces {
        output.push_str(&format!("  {},\n", safe_identifier(namespace)));
    }
    if typescript {
        output.push_str("} as const;\n\nexport default lgl;\n");
    } else {
        output.push_str("};\n\nexport default lgl;\n");
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    use crate::ecmascript::EcmaSource;
    use crate::generate_javascript_schema_files;

    use super::{
        compile_javascript_locale_artifact_module, compile_typescript_locale_artifact_module,
    };
    use crate::module::{
        generate_typescript_project_files, TypeScriptLocaleArtifactKind, TypeScriptLocaleModule,
        TypeScriptProjectOptions, ValidatedTypeScriptProject,
    };

    #[test]
    fn locale_artifacts_share_target_aware_namespace_and_barrel_emission() {
        let schema_text = "root(name: String)\naccount {\n  label\n  personalized(name: String)\n  nested {\n    status\n  }\n}\n";
        let base_text = "root = Hello {name}\naccount {\n  label = Account\n  personalized = Welcome {name}\n  nested {\n    status = Ready\n  }\n}\n";
        let regional_text = "root = Howdy {name}\n";
        let schema =
            lower_schema(&parse_schema_in(schema_text, SourceId(101)).expect("schema parses"));
        let locales = [
            TypeScriptLocaleModule {
                locale: "en".to_owned(),
                module: lower_locale(
                    &parse_locale_in(base_text, SourceId(102)).expect("base locale parses"),
                ),
            },
            TypeScriptLocaleModule {
                locale: "en-US".to_owned(),
                module: lower_locale(
                    &parse_locale_in(regional_text, SourceId(103)).expect("regional locale parses"),
                ),
            },
        ];
        let project_options = TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        let project = ValidatedTypeScriptProject::try_new(&schema, &locales, &project_options)
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
                        TypeScriptLocaleArtifactKind::Namespace { namespace }
                            if namespace == "account"
                    )
            })
            .expect("regional account artifact");
        let barrel = artifacts
            .iter()
            .find(|artifact| {
                artifact.locale == "en-US" && artifact.kind == TypeScriptLocaleArtifactKind::Barrel
            })
            .expect("regional barrel artifact");
        assert_eq!(namespace.module_path, "locales/en-US/account.ts");
        assert_eq!(
            namespace.source_ids,
            [SourceId(101), SourceId(102), SourceId(103)]
        );
        assert_eq!(barrel.module_path, "locales/en-US.ts");
        assert_eq!(
            barrel.source_ids,
            [SourceId(101), SourceId(102), SourceId(103)]
        );
        let sources = [
            EcmaSource::new(SourceId(101), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(102), "en.lgl", base_text),
            EcmaSource::new(SourceId(103), "en-US.lgl", regional_text),
        ];
        let project_files = generate_typescript_project_files(&project).expect("project files");

        for artifact in [namespace, barrel] {
            let typescript =
                compile_typescript_locale_artifact_module(&project, artifact, &sources)
                    .expect("TypeScript locale artifact");
            let javascript =
                compile_javascript_locale_artifact_module(&project, artifact, &sources)
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
                typescript
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
        let namespace_javascript =
            compile_javascript_locale_artifact_module(&project, namespace, &sources)
                .expect("JavaScript namespace");
        assert!(namespace_javascript.code.contains(
            "/** @type {((name: string) => string) & ((args: { name: string }) => string)} */"
        ));
        assert!(namespace_javascript
            .code
            .contains("export const account = {"));
        let barrel_javascript =
            compile_javascript_locale_artifact_module(&project, barrel, &sources)
                .expect("JavaScript barrel");
        assert!(barrel_javascript
            .code
            .contains("import { account } from \"./en-US/account.js\";"));
        assert!(barrel_javascript.code.contains("export default lgl;"));

        let namespace_typescript =
            compile_typescript_locale_artifact_module(&project, namespace, &sources)
                .expect("TypeScript namespace");
        let barrel_typescript =
            compile_typescript_locale_artifact_module(&project, barrel, &sources)
                .expect("TypeScript barrel");
        let shared_javascript = generate_javascript_schema_files(&schema)
            .into_iter()
            .find(|file| file.path == "shared.js")
            .expect("JavaScript shared module");
        let shared_typescript = project_files
            .iter()
            .find(|file| file.path == "shared.ts")
            .expect("TypeScript shared module");
        let snapshot_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-locale");
        let javascript_locale_root = snapshot_root.join("locales/en-US");
        let typescript_root = snapshot_root.join("typescript");
        let typescript_locale_root = typescript_root.join("locales/en-US");
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
            (
                typescript_root.join("shared.ts"),
                shared_typescript.contents.as_str(),
            ),
            (
                typescript_locale_root.join("account.ts"),
                namespace_typescript
                    .code
                    .strip_suffix("//# sourceMappingURL=account.ts.map\n")
                    .expect("namespace TypeScript source-map trailer"),
            ),
            (
                typescript_root.join("locales/en-US.ts"),
                barrel_typescript
                    .code
                    .strip_suffix("//# sourceMappingURL=en-US.ts.map\n")
                    .expect("barrel TypeScript source-map trailer"),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::create_dir_all(&javascript_locale_root)
                .expect("create JavaScript locale snapshot directory");
            std::fs::create_dir_all(&typescript_locale_root)
                .expect("create TypeScript locale snapshot directory");
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
