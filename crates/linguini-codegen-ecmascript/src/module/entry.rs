//! Project locale metadata and provider entry modules.

use linguini_cldr::built_in_text_direction;

use crate::ecmascript::{EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaStatement};

mod index;
mod locale;

use super::names::safe_identifier;
use super::{EcmaLocaleModule, EcmaProjectArtifact, EcmaProjectArtifactKind, ValidatedEcmaProject};

/// One source-mapped project entry module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledEcmaProjectModule {
    pub artifact: EcmaProjectArtifact,
    pub code: String,
    pub source_map: String,
}

pub fn compile_project_artifact_module(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaProjectArtifact,
) -> CompiledEcmaProjectModule {
    compile_project_artifact_module_impl(project, artifact, EcmaModuleKind::JavaScript)
}

fn compile_project_artifact_module_impl(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaProjectArtifact,
    target: EcmaModuleKind,
) -> CompiledEcmaProjectModule {
    let output_path = artifact.module_path.clone();
    let module = project_module(
        &project.locales,
        project.options.base_locale.as_deref(),
        artifact.kind,
        EcmaModuleOutput::new(target, output_path, None),
    );
    let rendered = module.render(&[]);
    CompiledEcmaProjectModule {
        artifact: artifact.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
    }
}

pub(super) fn generate_project_locale_declaration(
    locales: &[EcmaLocaleModule],
    base_locale: Option<&str>,
) -> String {
    locale::declaration(locales, base_locale)
}

pub(super) fn generate_project_index_declaration() -> String {
    index_declaration()
}

pub(super) fn project_module(
    locales: &[EcmaLocaleModule],
    base_locale: Option<&str>,
    kind: EcmaProjectArtifactKind,
    output: EcmaModuleOutput,
) -> EcmaModule {
    let code = match kind {
        EcmaProjectArtifactKind::Locale => locale::render(locales, base_locale),
        EcmaProjectArtifactKind::Index => index::render(locales, base_locale),
    };
    let mut module = EcmaModule::new(output);
    module.push_statement(EcmaStatement::generated(code));
    module
}

fn index_declaration() -> String {
    crate::templates::render("index_declarations", minijinja::context! {})
}

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}

fn locale_direction(locale: &str) -> &'static str {
    built_in_text_direction(locale).unwrap_or("ltr")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    use super::compile_project_artifact_module;
    use crate::ecmascript::EcmaSource;
    use crate::module::{
        compile_locale_artifact_module, generate_project_files, generate_schema_files,
        EcmaLocaleModule, EcmaProjectArtifactKind, EcmaProjectOptions, ValidatedEcmaProject,
    };

    #[test]
    fn project_artifacts_share_target_aware_entry_emission() {
        let schema_text = "hello(name: String)\n";
        let english_text = "hello = Hello {name}\n";
        let french_text = "hello = Salut {name}\n";
        let schema =
            lower_schema(&parse_schema_in(schema_text, SourceId(201)).expect("schema parses"));
        let locales = [
            EcmaLocaleModule {
                locale: "en".into(),
                module: lower_locale(
                    &parse_locale_in(english_text, SourceId(202)).expect("English"),
                ),
            },
            EcmaLocaleModule {
                locale: "fr".into(),
                module: lower_locale(&parse_locale_in(french_text, SourceId(203)).expect("French")),
            },
        ];
        let options = EcmaProjectOptions {
            base_locale: Some("en".into()),
            ..EcmaProjectOptions::default()
        };
        let project = ValidatedEcmaProject::try_new(&schema, &locales, &options).expect("project");
        let artifacts = project.project_artifacts();
        assert_eq!(
            artifacts
                .iter()
                .map(|artifact| artifact.module_path.as_str())
                .collect::<Vec<_>>(),
            ["locale.js", "index.js"]
        );
        assert_eq!(artifacts, project.project_artifacts());
        let project_files = generate_project_files(&project).expect("project files");

        for artifact in &artifacts {
            let javascript = compile_project_artifact_module(&project, artifact);
            let project_file = project_files
                .iter()
                .find(|file| file.path == artifact.module_path)
                .expect("project module");
            let map_name = format!("{}.map", artifact.module_path);
            assert_eq!(
                javascript
                    .code
                    .strip_suffix(&format!("//# sourceMappingURL={map_name}\n"))
                    .expect("TypeScript source-map trailer"),
                project_file.contents
            );
            assert!(javascript
                .code
                .ends_with(&format!("//# sourceMappingURL={}\n", map_name)));
            assert!(javascript
                .source_map
                .contains(&format!("\"file\":\"{}\"", artifact.module_path)));
            assert!(!javascript.code.contains(" as const"));
            assert!(!javascript.code.contains("import type"));
            assert!(!javascript.code.contains("export type"));
        }

        let locale = compile_project_artifact_module(&project, &artifacts[0]);
        assert_eq!(artifacts[0].kind, EcmaProjectArtifactKind::Locale);
        assert!(locale.code.contains("@returns {locale is Locale}"));
        assert!(locale.code.contains("\"fr\": \"fr\""));

        let index = compile_project_artifact_module(&project, &artifacts[1]);
        assert_eq!(artifacts[1].kind, EcmaProjectArtifactKind::Index);
        assert!(index.code.contains("import(\"./locales/fr.js\")"));
        assert!(index.code.contains("const pendingLocales = new Map();"));
        assert!(index
            .code
            .contains("export async function prepareLinguini(language)"));

        let snapshot_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-project");
        let sources = [
            EcmaSource::new(SourceId(201), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(202), "en.lgl", english_text),
            EcmaSource::new(SourceId(203), "fr.lgl", french_text),
        ];
        let mut snapshots = generate_schema_files(&schema)
            .into_iter()
            .filter(|file| file.path.ends_with(".js"))
            .map(|file| (snapshot_root.join(file.path), file.contents))
            .collect::<Vec<_>>();
        for artifact in project.locale_artifacts().expect("locale artifacts") {
            let compiled = compile_locale_artifact_module(&project, &artifact, &sources)
                .expect("JavaScript locale");
            snapshots.push((snapshot_root.join(&artifact.module_path), compiled.code));
            snapshots.push((
                snapshot_root.join(&artifact.source_map_path),
                compiled.source_map,
            ));
        }
        for artifact in &artifacts {
            let compiled = compile_project_artifact_module(&project, artifact);
            snapshots.push((snapshot_root.join(&artifact.module_path), compiled.code));
            snapshots.push((
                snapshot_root.join(&artifact.source_map_path),
                compiled.source_map,
            ));
        }
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write project snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read project snapshot")
            );
        }
    }
}
