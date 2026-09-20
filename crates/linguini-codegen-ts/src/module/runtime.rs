//! Shared locale runtime emission for TypeScript and checked JavaScript targets.

use linguini_ir::IrModule;
use linguini_syntax::SourceId;

use crate::ecmascript::{
    EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaSource, EcmaStatement,
};
use crate::plural::generate_plural_function_for_target;

use super::expr::formatter_data_declaration_for_target;
use super::formatters::{formatter_requirements, plural_required};
use super::message::ordered_sources;
use super::{
    locale_module_for_schema, project_locale_options, visible_schema, TypeScriptCodegenError,
    TypeScriptLocaleRuntimeArtifact, TypeScriptOptions, ValidatedTypeScriptProject,
};

/// One source-mapped locale helper runtime emitted from the common ECMAScript model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTypeScriptLocaleRuntimeModule {
    pub locale: String,
    pub code: String,
    pub source_map: String,
    pub source_ids: Vec<SourceId>,
}

impl CompiledTypeScriptLocaleRuntimeModule {
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

/// Checked-JavaScript locale runtime emitted from the common ECMAScript model.
pub type CompiledJavaScriptLocaleRuntimeModule = CompiledTypeScriptLocaleRuntimeModule;

/// Compiles one canonical TypeScript locale runtime artifact without recomputing its path.
pub fn compile_typescript_locale_runtime_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleRuntimeArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptLocaleRuntimeModule, TypeScriptCodegenError> {
    compile_locale_runtime_artifact_module(project, artifact, sources, EcmaScriptTarget::TypeScript)
}

/// Compiles one canonical checked-JavaScript locale runtime artifact.
pub fn compile_javascript_locale_runtime_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleRuntimeArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledJavaScriptLocaleRuntimeModule, TypeScriptCodegenError> {
    compile_locale_runtime_artifact_module(project, artifact, sources, EcmaScriptTarget::JavaScript)
}

fn compile_locale_runtime_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptLocaleRuntimeArtifact,
    sources: &[EcmaSource],
    target: EcmaScriptTarget,
) -> Result<CompiledTypeScriptLocaleRuntimeModule, TypeScriptCodegenError> {
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
    let module = locale_runtime_module(
        &schema,
        &locale_module,
        &options,
        EcmaModuleOutput::new(target, output_path, None),
    );
    let source_records = ordered_sources(&artifact.source_ids, sources)?;
    let rendered = module.render(&source_records);

    Ok(CompiledTypeScriptLocaleRuntimeModule {
        locale: artifact.canonical_locale.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
        source_ids: artifact.source_ids.clone(),
    })
}

pub(super) fn locale_runtime_module(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: EcmaModuleOutput,
) -> EcmaModule {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    let requirements = formatter_requirements(schema, locale);
    if requirements.any() {
        module.push_statement(EcmaStatement::generated(
            formatter_data_declaration_for_target(&options.locale, requirements, target, true),
        ));
    }
    if plural_required(schema, locale) {
        let rules = options
            .plural_rules
            .as_ref()
            .expect("project locale options always include plural rules");
        let source =
            generate_plural_function_for_target(&options.plural_function, rules, target, true);
        module.push_statement(EcmaStatement::generated(if requirements.any() {
            format!("\n\n{source}")
        } else {
            source
        }));
    }
    module
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    use crate::ecmascript::EcmaSource;

    use super::{
        compile_javascript_locale_runtime_artifact_module,
        compile_typescript_locale_runtime_artifact_module,
    };
    use crate::module::{
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions,
        ValidatedTypeScriptProject,
    };

    #[test]
    fn locale_runtime_artifact_uses_one_target_aware_backend() {
        let schema_text = "summary(count: Number)\n";
        let locale_text =
            "summary = {fn(Plural(count)) {\n  one => {count @number}\n  other => Many\n}}\n";
        let schema =
            lower_schema(&parse_schema_in(schema_text, SourceId(71)).expect("schema parses"));
        let locales = [TypeScriptLocaleModule {
            locale: "en-us".to_owned(),
            module: lower_locale(
                &parse_locale_in(locale_text, SourceId(72)).expect("locale parses"),
            ),
        }];
        let project_options = TypeScriptProjectOptions {
            base_locale: Some("en-us".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        let project = ValidatedTypeScriptProject::try_new(&schema, &locales, &project_options)
            .expect("validated project");
        let artifact = project
            .locale_runtime_artifacts()
            .expect("runtime artifacts")
            .into_iter()
            .next()
            .expect("English runtime artifact");
        let sources = [
            EcmaSource::new(SourceId(71), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(72), "locale.lgl", locale_text),
        ];

        let typescript =
            compile_typescript_locale_runtime_artifact_module(&project, &artifact, &sources)
                .expect("TypeScript runtime");
        let javascript =
            compile_javascript_locale_runtime_artifact_module(&project, &artifact, &sources)
                .expect("JavaScript runtime");
        let project_runtime = generate_typescript_project_files(&project)
            .expect("project files")
            .into_iter()
            .find(|file| file.path == artifact.module_path)
            .expect("project runtime");
        let source_map_trailer = "//# sourceMappingURL=_runtime.ts.map\n";
        assert_eq!(
            typescript
                .code
                .strip_suffix(source_map_trailer)
                .expect("TypeScript source-map trailer"),
            project_runtime.contents
        );

        assert!(javascript
            .code
            .contains("export function formatNumber(value)"));
        assert!(javascript
            .code
            .contains("export function pluralEnUs(value)"));
        assert!(!javascript.code.contains("formatCurrency"));
        assert!(!javascript.code.contains("formatDate"));
        assert!(!javascript.code.contains("type GeneratedNumeric ="));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=_runtime.js.map\n"));
        assert_eq!(javascript.source_ids, [SourceId(71), SourceId(72)]);
        assert!(javascript
            .source_map
            .contains("\"file\":\"locales/en-us/_runtime.js\""));
        assert!(javascript
            .source_map
            .contains("\"sources\":[\"schema.lgs\",\"locale.lgl\"]"));

        let snapshot_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-runtime");
        let code_snapshot = snapshot_root.join("_runtime.js");
        let map_snapshot = snapshot_root.join("_runtime.js.map");
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::create_dir_all(&snapshot_root).expect("create runtime snapshot directory");
            std::fs::write(&code_snapshot, &javascript.code)
                .expect("write JavaScript runtime snapshot");
            std::fs::write(&map_snapshot, &javascript.source_map)
                .expect("write JavaScript runtime source-map snapshot");
        }
        assert_eq!(
            javascript.code,
            std::fs::read_to_string(code_snapshot).expect("read JavaScript runtime snapshot")
        );
        assert_eq!(
            javascript.source_map,
            std::fs::read_to_string(map_snapshot)
                .expect("read JavaScript runtime source-map snapshot")
        );
    }
}
