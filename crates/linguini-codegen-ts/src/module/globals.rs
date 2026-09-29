//! Aggregate locale-global module emission for TypeScript and checked JavaScript.

use linguini_ir::{IrModule, IrModuleBuilder, IrSymbolKind};
use linguini_syntax::SourceId;

use crate::ecmascript::{
    EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaSource, EcmaStatement,
};

use super::emit::{
    emit_forms_for_target, emit_local_functions_for_target, emit_locale_enum_jsdoc_types,
    emit_locale_enum_types, emit_variables_for_target, module_imports,
};
use super::message::ordered_sources;
use super::names::{import_path_for_target, safe_identifier};
use super::{
    locale_global_value_names, locale_globals, locale_module_for_schema, locale_runtime_import,
    project_locale_options, visible_schema, TypeScriptCodegenError,
    TypeScriptLocaleGlobalsArtifact, TypeScriptOptions, ValidatedTypeScriptProject,
};

/// One source-mapped aggregate locale-global module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTypeScriptLocaleGlobalsModule {
    pub locale: String,
    pub code: String,
    pub source_map: String,
    pub source_ids: Vec<SourceId>,
}

impl CompiledTypeScriptLocaleGlobalsModule {
    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn source_map(&self) -> &str {
        &self.source_map
    }

    pub fn source_ids(&self) -> &[SourceId] {
        &self.source_ids
    }
}

pub type CompiledJavaScriptLocaleGlobalsModule = CompiledTypeScriptLocaleGlobalsModule;

pub fn compile_typescript_locale_globals_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleGlobalsArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptLocaleGlobalsModule, TypeScriptCodegenError> {
    compile_locale_globals_artifact_module(project, artifact, sources, EcmaScriptTarget::TypeScript)
}

pub fn compile_javascript_locale_globals_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleGlobalsArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledJavaScriptLocaleGlobalsModule, TypeScriptCodegenError> {
    compile_locale_globals_artifact_module(project, artifact, sources, EcmaScriptTarget::JavaScript)
}

fn compile_locale_globals_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleGlobalsArtifact,
    sources: &[EcmaSource],
    target: EcmaScriptTarget,
) -> Result<CompiledTypeScriptLocaleGlobalsModule, TypeScriptCodegenError> {
    let locale = project
        .locales
        .iter()
        .find(|locale| locale.locale == artifact.locale)
        .ok_or_else(|| TypeScriptCodegenError::UnknownLocale {
            locale: artifact.locale.clone(),
        })?;
    let options = project_locale_options(&locale.locale, &project.options)?;
    let schema = visible_schema(project.schema, &options);
    let locale_module = locale_module_for_schema(&locale.module, &schema);
    let output_path = match target {
        EcmaScriptTarget::TypeScript => artifact.module_path.clone(),
        EcmaScriptTarget::JavaScript => artifact
            .module_path
            .strip_suffix(".ts")
            .map_or_else(|| artifact.module_path.clone(), |path| format!("{path}.js")),
    };
    let module = locale_globals_module(
        &schema,
        &locale_module,
        &options,
        EcmaModuleOutput::new(target, output_path, None),
    );
    let source_records = ordered_sources(&artifact.source_ids, sources)?;
    let rendered = module.render(&source_records);
    Ok(CompiledTypeScriptLocaleGlobalsModule {
        locale: artifact.canonical_locale.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
        source_ids: artifact.source_ids.clone(),
    })
}

pub(super) fn locale_globals_module(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: EcmaModuleOutput,
) -> EcmaModule {
    let target = output.target();
    let globals_schema = IrModuleBuilder::seeded(schema)
        .retain_symbols(|kind, _| !matches!(kind, IrSymbolKind::Message | IrSymbolKind::Group))
        .build()
        .expect("globals schema projection preserves unique declaration names");
    let globals = locale_globals(locale);
    let shared_import_path = import_path_for_target("../../shared", target);
    let mut imports = module_imports(&globals_schema, &globals, options, &shared_import_path);
    let runtime_import_path = import_path_for_target("./_runtime", target);
    if let Some(runtime_import) =
        locale_runtime_import(&globals_schema, &globals, options, &runtime_import_path)
    {
        imports.push(runtime_import);
    }

    let mut body = String::new();
    if target.is_typescript() {
        emit_locale_enum_types(&globals_schema, &globals, &mut body);
    } else {
        emit_locale_enum_jsdoc_types(&globals_schema, &globals, &mut body);
    }
    emit_variables_for_target(&globals, options, target, false, &mut body);
    emit_forms_for_target(&globals, options, target, false, &mut body);
    emit_local_functions_for_target(&globals, options, target, false, &mut body);

    if target.is_typescript() {
        let local_enum_names = globals
            .enums()
            .iter()
            .filter(|item| {
                !globals_schema
                    .enums()
                    .iter()
                    .any(|schema| schema.name == item.name)
                    && !globals_schema
                        .type_aliases()
                        .iter()
                        .any(|schema| schema.name == item.name)
            })
            .map(|item| safe_identifier(&item.name))
            .collect::<Vec<_>>();
        if !local_enum_names.is_empty() {
            body.push_str(&format!(
                "export type {{ {} }};\n",
                local_enum_names.join(", ")
            ));
        }
    }
    let value_names = locale_global_value_names(&globals);
    if !value_names.is_empty() {
        body.push_str(&format!("export {{ {} }};\n", value_names.join(", ")));
    }

    let mut module = EcmaModule::new(output);
    module.extend_imports(imports);
    if !body.trim().is_empty() {
        let span = globals.origins().first().map(|origin| origin.span);
        module.push_statement(EcmaStatement::value(body, span));
    }
    module
}

#[cfg(test)]
mod tests {
    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    use crate::ecmascript::EcmaSource;

    use super::{
        compile_javascript_locale_globals_artifact_module,
        compile_typescript_locale_globals_artifact_module,
    };
    use crate::module::{
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions,
        ValidatedTypeScriptProject,
    };

    #[test]
    fn locale_globals_artifact_uses_one_target_aware_backend() {
        let schema_text = "enum Fruit { apple }\nsummary(fruit: Fruit, count: Number)\n";
        let locale_text = "enum Gender { male, other }\n\
let prefix = Total\n\
impl Fruit {\n\
  apple {\n\
    Gender = male\n\
    form label(Plural) {\n\
      one => One {prefix}\n\
      _ => Many {prefix}\n\
    }\n\
  }\n\
}\n\
fn Render(Gender, Plural) {\n\
  male {\n\
    one => One\n\
    _ => Many\n\
  }\n\
  other {\n\
    one => Other one\n\
    _ => Other many\n\
  }\n\
}\n\
summary = {Render(fruit.Gender, count)}: {fruit.label(count)}\n";
        let schema =
            lower_schema(&parse_schema_in(schema_text, SourceId(81)).expect("schema parses"));
        let locales = [TypeScriptLocaleModule {
            locale: "en-us".to_owned(),
            module: lower_locale(
                &parse_locale_in(locale_text, SourceId(82)).expect("locale parses"),
            ),
        }];
        let project_options = TypeScriptProjectOptions {
            base_locale: Some("en-us".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        let project = ValidatedTypeScriptProject::try_new(&schema, &locales, &project_options)
            .expect("validated project");
        let artifacts = project
            .locale_globals_artifacts()
            .expect("globals artifacts");
        assert_eq!(artifacts.len(), 1);
        let artifact = &artifacts[0];
        assert_eq!(artifact.locale, "en-us");
        assert_eq!(artifact.canonical_locale, "en-US");
        assert_eq!(artifact.module_path, "locales/en-us/_globals.ts");
        assert_eq!(artifact.source_ids, [SourceId(82)]);
        let sources = [
            EcmaSource::new(SourceId(81), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(82), "locale.lgl", locale_text),
        ];

        let typescript =
            compile_typescript_locale_globals_artifact_module(&project, artifact, &sources)
                .expect("TypeScript globals");
        let javascript =
            compile_javascript_locale_globals_artifact_module(&project, artifact, &sources)
                .expect("JavaScript globals");
        let project_globals = generate_typescript_project_files(&project)
            .expect("project files")
            .into_iter()
            .find(|file| file.path == artifact.module_path)
            .expect("project globals");
        assert_eq!(
            typescript
                .code()
                .strip_suffix("//# sourceMappingURL=_globals.ts.map\n")
                .expect("TypeScript source-map trailer"),
            project_globals.contents
        );

        assert!(javascript.code().contains("from \"../../shared.js\""));
        assert!(javascript.code().contains("from \"./_runtime.js\""));
        assert!(javascript
            .code()
            .contains("/** @typedef {\"male\" | \"other\"} Gender */"));
        assert!(javascript
            .code()
            .contains("@param {number | bigint | string} __lgl_p1"));
        assert!(javascript
            .code()
            .contains("export { prefix, __lgl_form_4672756974, Render };"));
        for forbidden in ["type Gender =", ": string", " as const", "import type"] {
            assert!(!javascript.code().contains(forbidden), "{forbidden}");
        }
        assert!(javascript
            .code()
            .ends_with("//# sourceMappingURL=_globals.js.map\n"));
        assert_eq!(javascript.locale(), "en-US");
        assert_eq!(javascript.source_ids(), [SourceId(82)]);
        assert!(javascript
            .source_map()
            .contains("\"file\":\"locales/en-us/_globals.js\""));
        assert!(javascript
            .source_map()
            .contains("\"sources\":[\"locale.lgl\"]"));
    }

    #[test]
    fn locale_globals_artifacts_omit_locales_without_globals() {
        let schema =
            lower_schema(&parse_schema_in("message\n", SourceId(91)).expect("schema parses"));
        let locales = [TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(
                &parse_locale_in("message = Message\n", SourceId(92)).expect("locale parses"),
            ),
        }];
        let project_options = TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        let project = ValidatedTypeScriptProject::try_new(&schema, &locales, &project_options)
            .expect("validated project");

        assert!(project
            .locale_globals_artifacts()
            .expect("globals artifacts")
            .is_empty());
    }
}
