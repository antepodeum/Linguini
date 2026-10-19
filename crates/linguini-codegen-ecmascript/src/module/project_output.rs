//! Project-wide ECMAScript artifact ownership and deterministic rendering.

use super::names::import_path_for_target;
use super::{
    artifacts, decl, entry, globals, locale, messages, runtime, shared, EcmaCodegenError,
    EcmaGeneratedFile, EcmaLocaleArtifactKind, EcmaOptions, ValidatedEcmaProject,
};
use crate::ecmascript::{
    EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaSource, RenderedEcmaModule,
};
use linguini_syntax::SourceId;

pub(super) enum ProjectFile {
    Module {
        module: EcmaModule,
        source_ids: Vec<SourceId>,
    },
    Rendered(RenderedEcmaModule),
    Data(EcmaGeneratedFile),
}

impl ProjectFile {
    fn source_only(self) -> EcmaGeneratedFile {
        match self {
            Self::Module { module, .. } => EcmaGeneratedFile {
                path: module.output().path().to_owned(),
                contents: module.render_code(),
            },
            Self::Rendered(module) => EcmaGeneratedFile {
                path: module.output.path().to_owned(),
                contents: module
                    .code
                    .strip_suffix(&format!(
                        "//# sourceMappingURL={}\n",
                        module.output.source_map_path().rsplit('/').next().unwrap()
                    ))
                    .expect("renderer owns map trailer")
                    .to_owned(),
            },
            Self::Data(file) => file,
        }
    }

    fn with_sources(
        self,
        sources: &[EcmaSource],
        files: &mut Vec<EcmaGeneratedFile>,
    ) -> Result<(), EcmaCodegenError> {
        match self {
            Self::Data(file) => files.push(file),
            Self::Rendered(module) => append_rendered(files, module),
            Self::Module { module, source_ids } => {
                let mut records = super::message::ordered_sources(&source_ids, sources)?;
                let prefix = "../".repeat(
                    module
                        .output()
                        .path()
                        .bytes()
                        .filter(|byte| *byte == b'/')
                        .count(),
                );
                for source in &mut records {
                    source.path = format!("{prefix}{}", source.path);
                }
                append_rendered(files, module.render(&records));
            }
        }
        Ok(())
    }
}

fn append_rendered(files: &mut Vec<EcmaGeneratedFile>, module: RenderedEcmaModule) {
    files.push(EcmaGeneratedFile {
        path: module.output.path().to_owned(),
        contents: module.code,
    });
    files.push(EcmaGeneratedFile {
        path: module.output.source_map_path(),
        contents: module.source_map,
    });
}

/// Generate the source-only ECMAScript project API from structured artifacts.
pub(super) fn generate_project_files(
    project: &ValidatedEcmaProject<'_>,
) -> Result<Vec<EcmaGeneratedFile>, EcmaCodegenError> {
    let files = project_files(project, EcmaModuleKind::JavaScript)?
        .into_iter()
        .map(ProjectFile::source_only)
        .collect::<Vec<_>>();
    super::validate_generated_output_collisions(&files)?;
    Ok(files)
}

/// Compile a complete native ECMAScript project, including owned source-map companions.
/// Every referenced source identity must be present exactly once in `sources`.
/// Source paths are relative to the output root; each map rebases them to its own directory.
pub fn compile_ecmascript_project_files(
    project: &ValidatedEcmaProject<'_>,
    sources: &[EcmaSource],
) -> Result<Vec<EcmaGeneratedFile>, EcmaCodegenError> {
    let mut files = Vec::new();
    for artifact in project_files(project, EcmaModuleKind::JavaScript)? {
        artifact.with_sources(sources, &mut files)?;
    }
    super::validate_generated_output_collisions(&files)?;
    Ok(files)
}

pub(super) fn data(files: &mut Vec<ProjectFile>, path: impl Into<String>, contents: String) {
    files.push(ProjectFile::Data(EcmaGeneratedFile {
        path: path.into(),
        contents,
    }));
}
pub(super) fn rendered(files: &mut Vec<ProjectFile>, module: RenderedEcmaModule) {
    files.push(ProjectFile::Rendered(module));
}
fn module(files: &mut Vec<ProjectFile>, module: EcmaModule, source_ids: Vec<SourceId>) {
    let source_ids = if source_ids.is_empty() {
        module.source_ids()
    } else {
        source_ids
    };
    files.push(ProjectFile::Module { module, source_ids });
}
fn output(stem: &str, target: EcmaModuleKind) -> EcmaModuleOutput {
    EcmaModuleOutput::new(
        target,
        format!(
            "{stem}.{}",
            if target.is_declaration() { "ts" } else { "js" }
        ),
        None,
    )
}

fn project_files(
    project: &ValidatedEcmaProject<'_>,
    target: EcmaModuleKind,
) -> Result<Vec<ProjectFile>, EcmaCodegenError> {
    let mut files = Vec::new();
    append_schema(project, target, &mut files);
    append_locales(project, target, &mut files)?;
    append_entries(project, target, &mut files);
    super::project_framework::append(project, &mut files);
    Ok(files)
}

fn append_schema(
    project: &ValidatedEcmaProject<'_>,
    target: EcmaModuleKind,
    files: &mut Vec<ProjectFile>,
) {
    let options = &project.options;
    let schema = project.schema.as_module();
    let selected_schema = super::visible_schema(
        schema,
        &EcmaOptions {
            included_messages: if options.tree_shaking {
                options.included_messages.clone()
            } else {
                vec![]
            },
            ..Default::default()
        },
    );
    module(
        files,
        shared::shared_module(schema, output("shared", target), false),
        vec![],
    );
    module(
        files,
        messages::messages_module(
            &selected_schema,
            output("messages", target),
            &import_path_for_target("./shared", target),
        ),
        vec![],
    );
    if options.gitignore {
        data(
            files,
            ".gitignore",
            "# Generated by Linguini. Do not edit.\n*\n".to_owned(),
        );
    }
    if options.declaration {
        module(
            files,
            shared::shared_module(
                schema,
                EcmaModuleOutput::new(EcmaModuleKind::Declaration, "shared.d.ts", None),
                true,
            ),
            vec![],
        );
        module(
            files,
            messages::messages_module(
                &selected_schema,
                EcmaModuleOutput::new(EcmaModuleKind::Declaration, "messages.d.ts", None),
                &import_path_for_target("./shared", EcmaModuleKind::JavaScript),
            ),
            vec![],
        );
    }
}

fn append_locales(
    project: &ValidatedEcmaProject<'_>,
    target: EcmaModuleKind,
    files: &mut Vec<ProjectFile>,
) -> Result<(), EcmaCodegenError> {
    let options = &project.options;
    let schema = project.schema.as_module();
    let runtimes = project.locale_runtime_artifacts()?;
    let globals = project.locale_globals_artifacts()?;
    let locales = project.locale_artifacts()?;
    for locale in &project.locales {
        let runtime_artifact = runtimes
            .iter()
            .find(|artifact| artifact.locale == locale.locale)
            .expect("effective locale runtime");
        module(
            files,
            runtime::locale_runtime_artifact_module(project, runtime_artifact, target)?,
            runtime_artifact.source_ids.clone(),
        );
        if let Some(artifact) = globals
            .iter()
            .find(|artifact| artifact.locale == locale.locale)
        {
            module(
                files,
                globals::locale_globals_artifact_module(project, artifact, target)?,
                artifact.source_ids.clone(),
            );
        }
        for artifact in locales
            .iter()
            .filter(|artifact| artifact.locale == locale.locale)
        {
            module(
                files,
                locale::locale_artifact_module(project, artifact, target)?,
                artifact.source_ids.clone(),
            );
            if options.declaration {
                let locale_options = super::project_locale_options(&locale.locale, options)?;
                let schema = super::visible_schema(schema, &locale_options);
                let (path, declaration) = match &artifact.kind {
                    EcmaLocaleArtifactKind::Namespace { namespace } => {
                        let schema = super::namespace_module(&schema, namespace);
                        let path =
                            artifact.module_path.trim_end_matches(".js").to_owned() + ".d.ts";
                        let declaration = decl::generate_locale_declaration_with_shared_import(
                            &schema,
                            &path,
                            &import_path_for_target("../../shared", target),
                            Some(namespace),
                        );
                        (path, declaration)
                    }
                    EcmaLocaleArtifactKind::Barrel => {
                        let namespaces = super::top_level_namespaces(&schema);
                        let root = if namespaces.is_empty() {
                            schema
                        } else {
                            super::root_module(&schema)
                        };
                        (
                            format!("locales/{}.d.ts", locale.locale),
                            decl::generate_locale_declaration_with_namespaces(
                                &root,
                                &locale.locale,
                                &namespaces,
                            ),
                        )
                    }
                };
                data(files, path, declaration);
            }
        }
    }
    Ok(())
}

fn append_entries(
    project: &ValidatedEcmaProject<'_>,
    target: EcmaModuleKind,
    files: &mut Vec<ProjectFile>,
) {
    let options = &project.options;
    for artifact in artifacts::project_artifacts() {
        let stem = artifact.module_path.trim_end_matches(".js");
        module(
            files,
            entry::project_module(
                &project.locales,
                options.base_locale.as_deref(),
                artifact.kind,
                output(stem, target),
            ),
            vec![],
        );
        if options.declaration {
            let declaration = match artifact.kind {
                super::EcmaProjectArtifactKind::Locale => {
                    entry::generate_project_locale_declaration(
                        &project.locales,
                        options.base_locale.as_deref(),
                    )
                }
                super::EcmaProjectArtifactKind::Index => {
                    entry::generate_project_index_declaration()
                }
            };
            data(files, format!("{stem}.d.ts"), declaration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{EcmaFramework, EcmaLocaleModule, EcmaProjectOptions, EcmaWebOptions};
    use super::*;
    use linguini_ir::{lower_locale_typed, lower_schema_typed};
    use linguini_syntax::{parse_locale_in, parse_schema_in};
    use std::path::Path;

    #[test]
    fn complete_project_targets_own_maps_declarations_fallbacks_and_capabilities() {
        let schema_source = "/// Greeting\nhello(name: String)\nsection { title }\n";
        let en_source = "hello = Hello {name}\nsection { title = English }\n";
        let fr_source = "hello = Bonjour {name}\n";
        let schema = lower_schema_typed(&parse_schema_in(schema_source, SourceId(7)).unwrap());
        let locales = [
            EcmaLocaleModule {
                locale: "en".to_owned(),
                module: lower_locale_typed(&parse_locale_in(en_source, SourceId(11)).unwrap()),
            },
            EcmaLocaleModule {
                locale: "fr".to_owned(),
                module: lower_locale_typed(&parse_locale_in(fr_source, SourceId(13)).unwrap()),
            },
        ];
        let sources = [
            EcmaSource::new(SourceId(7), "schema/main.lgs", schema_source),
            EcmaSource::new(SourceId(11), "locale/en/main.lgl", en_source),
            EcmaSource::new(SourceId(13), "locale/fr/main.lgl", fr_source),
        ];
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-complete-project");
        for (name, declaration, framework) in [
            ("plain", true, None),
            ("source-only", false, None),
            ("kit", true, Some(EcmaFramework::SvelteKit)),
        ] {
            let options = EcmaProjectOptions {
                base_locale: Some("en".to_owned()),
                declaration,
                gitignore: false,
                framework,
                web: framework.map(|_| EcmaWebOptions::default()),
                ..Default::default()
            };
            let project = ValidatedEcmaProject::try_new(&schema, &locales, &options).unwrap();
            assert!(compile_ecmascript_project_files(&project, &sources[..1]).is_err());
            assert!(compile_ecmascript_project_files(
                &project,
                &[
                    sources[0].clone(),
                    sources[0].clone(),
                    sources[1].clone(),
                    sources[2].clone()
                ]
            )
            .is_err());
            {
                let language = "javascript";
                assert!(matches!(
                    compile_ecmascript_project_files(&project, &sources[1..]),
                    Err(EcmaCodegenError::MissingMessageSource {
                        source_id: SourceId(7)
                    })
                ));
                let files = compile_ecmascript_project_files(&project, &sources).unwrap();
                for stem in ["shared", "messages"] {
                    assert_eq!(
                        files
                            .iter()
                            .any(|file| file.path == format!("{stem}.d.ts.map")),
                        declaration
                    );
                }
                for file in &files {
                    if file.path.ends_with(".d.ts") {
                        assert!(declaration);
                    }
                    if file.path.ends_with(".map") {
                        let expected = file.path.trim_end_matches(".map");
                        assert!(file.contents.contains(&format!("\"file\":\"{expected}\"")));
                        let implementation =
                            files.iter().find(|item| item.path == expected).unwrap();
                        assert!(implementation.contents.ends_with(&format!(
                            "//# sourceMappingURL={}\n",
                            file.path.rsplit('/').next().unwrap()
                        )));
                        if file.path == "locales/fr/section.js.map" {
                            assert!(file
                                .contents
                                .contains(en_source.replace('\n', "\\n").as_str()));
                            assert!(file.contents.contains("\"mappings\":\""));
                            assert!(!file.contents.contains("\"mappings\":\"\""));
                        }
                    } else if file.path != ".gitignore" && !file.path.ends_with(".d.ts") {
                        assert!(files
                            .iter()
                            .any(|item| item.path == format!("{}.map", file.path)));
                    }
                    let path = root.join(format!("{name}/{language}/{}", file.path));
                    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
                        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                        std::fs::write(&path, &file.contents).unwrap();
                    }
                    assert_eq!(file.contents, std::fs::read_to_string(path).unwrap());
                }
                assert_eq!(
                    files.iter().any(|file| file.path.starts_with("web/")),
                    framework.is_some()
                );
                if !declaration {
                    assert!(!files.iter().any(|file| file.path.ends_with(".d.ts")));
                }
            }
        }
    }
}
