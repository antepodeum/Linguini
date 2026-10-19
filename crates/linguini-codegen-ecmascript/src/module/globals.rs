//! Aggregate locale-global module emission for canonical JavaScript.

use linguini_ir::{IrModule, IrModuleBuilder, IrSymbolKind};
use linguini_syntax::SourceId;

use crate::ecmascript::{EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaSource, EcmaStatement};

use super::emit::{
    emit_forms, emit_local_functions, emit_locale_enum_jsdoc_types, emit_variables, module_imports,
};
use super::message::ordered_sources;
use super::names::import_path_for_target;
use super::{
    locale_global_value_names, locale_globals, locale_module_for_schema, locale_runtime_import,
    project_locale_options, visible_schema, EcmaCodegenError, EcmaLocaleGlobalsArtifact,
    EcmaOptions, ValidatedEcmaProject,
};

/// One source-mapped aggregate locale-global module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledEcmaLocaleGlobalsModule {
    pub locale: String,
    pub code: String,
    pub source_map: String,
    pub source_ids: Vec<SourceId>,
}

impl CompiledEcmaLocaleGlobalsModule {
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

pub fn compile_locale_globals_artifact_module(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaLocaleGlobalsArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledEcmaLocaleGlobalsModule, EcmaCodegenError> {
    compile_locale_globals_artifact_module_impl(
        project,
        artifact,
        sources,
        EcmaModuleKind::JavaScript,
    )
}

fn compile_locale_globals_artifact_module_impl(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaLocaleGlobalsArtifact,
    sources: &[EcmaSource],
    target: EcmaModuleKind,
) -> Result<CompiledEcmaLocaleGlobalsModule, EcmaCodegenError> {
    let module = locale_globals_artifact_module(project, artifact, target)?;
    let source_records = ordered_sources(&artifact.source_ids, sources)?;
    let rendered = module.render(&source_records);
    Ok(CompiledEcmaLocaleGlobalsModule {
        locale: artifact.canonical_locale.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
        source_ids: artifact.source_ids.clone(),
    })
}

pub(super) fn locale_globals_artifact_module(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaLocaleGlobalsArtifact,
    target: EcmaModuleKind,
) -> Result<EcmaModule, EcmaCodegenError> {
    let locale = project
        .locales
        .iter()
        .find(|locale| locale.locale == artifact.locale)
        .ok_or_else(|| EcmaCodegenError::UnknownLocale {
            locale: artifact.locale.clone(),
        })?;
    let options = project_locale_options(&locale.locale, &project.options)?;
    let schema = visible_schema(project.schema, &options);
    let locale_module = locale_module_for_schema(&locale.module, &schema);
    let output_path = artifact
        .module_path
        .strip_suffix(".ts")
        .map_or_else(|| artifact.module_path.clone(), |path| format!("{path}.js"));
    let module = locale_globals_module(
        &schema,
        &locale_module,
        &options,
        EcmaModuleOutput::new(target, output_path, None),
    );
    Ok(module)
}

pub(super) fn locale_globals_module(
    schema: &IrModule,
    locale: &IrModule,
    options: &EcmaOptions,
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
    {
        emit_locale_enum_jsdoc_types(&globals_schema, &globals, &mut body);
    }
    emit_variables(&globals, options, false, &mut body);
    emit_forms(&globals, options, false, &mut body);
    emit_local_functions(&globals, options, false, &mut body);

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
    use std::path::Path;

    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    use crate::ecmascript::EcmaSource;
    use crate::{compile_locale_runtime_artifact_module, generate_schema_files};

    use super::compile_locale_globals_artifact_module;
    use crate::module::{
        generate_project_files, EcmaLocaleModule, EcmaProjectOptions, ValidatedEcmaProject,
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
        let locales = [EcmaLocaleModule {
            locale: "en-us".to_owned(),
            module: lower_locale(
                &parse_locale_in(locale_text, SourceId(82)).expect("locale parses"),
            ),
        }];
        let project_options = EcmaProjectOptions {
            base_locale: Some("en-us".to_owned()),
            ..EcmaProjectOptions::default()
        };
        let project = ValidatedEcmaProject::try_new(&schema, &locales, &project_options)
            .expect("validated project");
        let artifacts = project
            .locale_globals_artifacts()
            .expect("globals artifacts");
        assert_eq!(artifacts.len(), 1);
        let artifact = &artifacts[0];
        assert_eq!(artifact.locale, "en-us");
        assert_eq!(artifact.canonical_locale, "en-US");
        assert_eq!(artifact.module_path, "locales/en-us/_globals.js");
        assert_eq!(artifact.source_ids, [SourceId(82)]);
        let sources = [
            EcmaSource::new(SourceId(81), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(82), "locale.lgl", locale_text),
        ];
        let javascript = compile_locale_globals_artifact_module(&project, artifact, &sources)
            .expect("JavaScript globals");
        let project_files = generate_project_files(&project).expect("project files");
        let project_globals = project_files
            .iter()
            .find(|file| file.path == artifact.module_path)
            .expect("project globals");
        assert_eq!(
            javascript
                .code()
                .strip_suffix("//# sourceMappingURL=_globals.js.map\n")
                .expect("TypeScript source-map trailer"),
            project_globals.contents
        );

        assert!(!javascript.code().contains("from \"../../shared.js\""));
        assert!(javascript.code().contains("from \"./_runtime.js\""));
        assert!(javascript
            .code()
            .contains("/** @typedef {\"male\" | \"other\"} Gender */"));
        assert!(javascript
            .code()
            .contains("@param {number | bigint | string} plural"));
        assert!(javascript
            .code()
            .contains("/** @type {(value: number | bigint | string) => string} */ ((value) =>"));
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

        let runtime_artifact = project
            .locale_runtime_artifacts()
            .expect("runtime artifacts")
            .into_iter()
            .next()
            .expect("English runtime artifact");
        let runtime = compile_locale_runtime_artifact_module(&project, &runtime_artifact, &sources)
            .expect("JavaScript runtime");
        let shared = generate_schema_files(&schema)
            .into_iter()
            .find(|file| file.path == "shared.js")
            .expect("JavaScript shared module");
        let snapshot_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-globals");
        let locale_snapshot_root = snapshot_root.join("locales/en-us");
        let snapshots = [
            (snapshot_root.join("shared.js"), shared.contents.as_str()),
            (
                locale_snapshot_root.join("_runtime.js"),
                runtime.code.as_str(),
            ),
            (locale_snapshot_root.join("_globals.js"), javascript.code()),
            (
                locale_snapshot_root.join("_globals.js.map"),
                javascript.source_map(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::create_dir_all(&locale_snapshot_root)
                .expect("create globals snapshot directory");
            for (path, contents) in &snapshots {
                std::fs::write(path, contents).expect("write globals snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read globals snapshot")
            );
        }
    }

    #[test]
    fn locale_globals_artifacts_omit_locales_without_globals() {
        let schema =
            lower_schema(&parse_schema_in("message\n", SourceId(91)).expect("schema parses"));
        let locales = [EcmaLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(
                &parse_locale_in("message = Message\n", SourceId(92)).expect("locale parses"),
            ),
        }];
        let project_options = EcmaProjectOptions {
            base_locale: Some("en".to_owned()),
            ..EcmaProjectOptions::default()
        };
        let project = ValidatedEcmaProject::try_new(&schema, &locales, &project_options)
            .expect("validated project");

        assert!(project
            .locale_globals_artifacts()
            .expect("globals artifacts")
            .is_empty());
    }
}
