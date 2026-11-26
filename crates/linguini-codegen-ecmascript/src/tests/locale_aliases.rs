use super::*;
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

#[test]
fn exact_locale_spelling_owns_ordinary_physical_and_standalone_message_sources() {
    let schema_source = "title\ncategory(count: Number)\n";
    let schema = lower_schema(&parse_schema_in(schema_source, SourceId(1)).unwrap());
    let inputs = [
        ("en", 4, "title = Base\ncategory = {Plural(count)}\n"),
        ("he-IL", 2, "title = Sibling\n"),
        ("he", 3, "title = Parent\n"),
        ("iw-IL", 5, ""),
    ];
    let locales = inputs
        .iter()
        .map(|&(locale, id, source)| EcmaLocaleModule {
            locale: locale.into(),
            module: lower_locale(&parse_locale_in(source, SourceId(id)).unwrap()),
        })
        .collect::<Vec<_>>();
    let mut sources = vec![crate::EcmaSource::new(
        SourceId(1),
        "schema.lgs",
        schema_source,
    )];
    sources.extend(inputs.iter().map(|&(locale, id, source)| {
        crate::EcmaSource::new(SourceId(id), format!("{locale}.lgl"), source)
    }));
    let options = project_options("en");
    let project = ValidatedEcmaProject::try_new(&schema, &locales, &options).unwrap();
    let mut files = crate::generate_project_files(&project).unwrap();
    let artifact = project
        .message_artifacts()
        .unwrap()
        .into_iter()
        .find(|artifact| artifact.locale == "iw-IL" && artifact.message == "title")
        .unwrap();
    assert_eq!(artifact.locale, "iw-IL");
    assert_eq!(
        project
            .message_dependency_closure("iw-IL", "title")
            .unwrap()
            .source_ids(),
        [SourceId(1), SourceId(3)]
    );
    let physical =
        crate::compile_bundler_message_artifact_module(&project, &artifact, &sources).unwrap();
    assert_eq!(physical.locale(), "he-IL");
    assert_eq!(physical.source_ids(), [SourceId(1), SourceId(3)]);
    assert!(physical.source_map().contains("he.lgl"));
    assert!(!physical.source_map().contains("he-IL.lgl"));
    files.push(EcmaGeneratedFile {
        path: "physical.js".into(),
        contents: physical.code().into(),
    });
    let standalone = crate::compile_message_module(
        &project,
        "iw-IL",
        "title",
        "standalone.js",
        "./shared.js",
        &sources,
    )
    .unwrap();
    assert_eq!(standalone.locale(), "he-IL");
    assert_eq!(standalone.source_ids(), [SourceId(1), SourceId(3)]);
    files.push(EcmaGeneratedFile {
        path: "standalone.js".into(),
        contents: standalone.code().into(),
    });
    let category_artifact = project
        .message_artifacts()
        .unwrap()
        .into_iter()
        .find(|artifact| artifact.locale == "iw-IL" && artifact.message == "category")
        .unwrap();
    let category =
        crate::compile_bundler_message_artifact_module(&project, &category_artifact, &sources)
            .unwrap();
    assert_eq!(category.source_ids(), [SourceId(1), SourceId(4)]);
    files.push(EcmaGeneratedFile {
        path: category_artifact.module_path.clone(),
        contents: category.code().into(),
    });
    files.push(EcmaGeneratedFile {
        path: "physical-category.js".into(),
        contents: format!(
            "export {{message}} from {:?};",
            format!("./{}", category_artifact.module_path)
        ),
    });
    let standalone_category = crate::compile_message_module(
        &project,
        "iw-IL",
        "category",
        "standalone-category.js",
        "./shared.js",
        &sources,
    )
    .unwrap();
    files.push(EcmaGeneratedFile {
        path: "standalone-category.js".into(),
        contents: standalone_category.code().into(),
    });
    assert_eq!(
        execute_project(
            &files,
            r#"
import assert from 'node:assert/strict';
import ordinary from './locales/iw-IL.js';
import {message as physical} from './physical.js';
import {message as standalone} from './standalone.js';
import {message as category} from './physical-category.js';
import {message as standaloneCategory} from './standalone-category.js';
assert.equal(ordinary.title,'Parent');
assert.equal(physical,'Parent');
assert.equal(standalone(),'Parent');
assert.equal(ordinary.category(1),'one');
assert.equal(category(1),'one');
assert.equal(standaloneCategory(1),'one');
console.log('exact-locale-ok');
"#
        ),
        "exact-locale-ok"
    );
    let mixed_case = crate::compile_message_module(
        &project,
        "IW-il",
        "title",
        "mixed.js",
        "./shared.js",
        &sources,
    )
    .unwrap();
    assert_eq!(mixed_case.locale(), "he-IL");
    assert_eq!(mixed_case.source_ids(), [SourceId(1), SourceId(3)]);
    let canonical_only = ValidatedEcmaProject::try_new(&schema, &locales[..3], &options).unwrap();
    let aliased = crate::compile_message_module(
        &canonical_only,
        "iw-IL",
        "title",
        "alias.js",
        "./shared.js",
        &sources,
    )
    .unwrap();
    assert_eq!(aliased.locale(), "he-IL");
    assert_eq!(aliased.source_ids(), [SourceId(1), SourceId(2)]);
    assert!(matches!(
        crate::compile_message_module(
            &project,
            "he-XX",
            "title",
            "unknown.js",
            "./shared.js",
            &sources
        ),
        Err(EcmaCodegenError::UnknownLocale { .. })
    ));
}
