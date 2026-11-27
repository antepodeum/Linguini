//! Execute generated projects as native ESM, including their actual module graph.

use crate::{EcmaGeneratedFile, EcmaLocaleModule, EcmaProjectOptions, ValidatedEcmaProject};
use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) fn execute_project(files: &[EcmaGeneratedFile], script: &str) -> String {
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/codegen-execution")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&directory).expect("create execution directory");
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    fs::write(directory.join("package.json"), "{\"type\":\"module\"}").expect("ESM package");
    for file in files {
        let path = directory.join(&file.path);
        fs::create_dir_all(path.parent().expect("file parent")).expect("module directory");
        fs::write(path, &file.contents).expect("generated module");
    }
    let output = Command::new("node")
        .args(["--input-type=module", "--eval", script])
        .current_dir(&directory)
        .output()
        .expect("Node.js must be available");
    assert!(
        output.status.success(),
        "generated project failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("UTF-8 output")
        .trim()
        .to_owned()
}

/// Compile the `root` message and its actual dependencies in every module shape.
pub(crate) fn root_project_files(
    locale_name: &str,
    schema_source: &str,
    locale_source: &str,
) -> (Vec<EcmaGeneratedFile>, String) {
    let schema = lower_schema(&parse_schema_in(schema_source, SourceId(1)).unwrap());
    let locales = [EcmaLocaleModule {
        locale: locale_name.into(),
        module: lower_locale(&parse_locale_in(locale_source, SourceId(2)).unwrap()),
    }];
    let project = ValidatedEcmaProject::try_new(
        &schema,
        &locales,
        &EcmaProjectOptions {
            base_locale: Some(locale_name.to_owned()),
            ..EcmaProjectOptions::default()
        },
    )
    .unwrap();
    let sources = [
        crate::EcmaSource::new(SourceId(1), "schema.lgs", schema_source),
        crate::EcmaSource::new(SourceId(2), format!("{locale_name}.lgl"), locale_source),
    ];
    let mut files = crate::generate_project_files(&project).unwrap();
    for artifact in project.semantic_artifacts().unwrap() {
        let compiled =
            crate::compile_bundler_semantic_module(&project, &artifact, &sources).unwrap();
        files.push(EcmaGeneratedFile {
            path: artifact.module_path,
            contents: compiled.code,
        });
    }
    let artifact = project.message_artifacts().unwrap().remove(0);
    let physical =
        crate::compile_bundler_message_artifact_module(&project, &artifact, &sources).unwrap();
    files.push(EcmaGeneratedFile {
        path: artifact.module_path.clone(),
        contents: physical.code,
    });
    let standalone = crate::compile_message_module(
        &project,
        locale_name,
        "root",
        "standalone.js",
        "./shared.js",
        &sources,
    )
    .unwrap();
    files.push(EcmaGeneratedFile {
        path: "standalone.js".into(),
        contents: standalone.code,
    });
    (files, format!("./{}", artifact.module_path))
}
