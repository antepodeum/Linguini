use crate::{generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions};
use linguini_ir::{lower_locale, lower_schema};
use linguini_syntax::{parse_locale, parse_schema};
use std::fs;
use std::path::Path;

#[test]
fn generated_module_snapshot_is_stable() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lgs"
    ))
    .expect("schema");
    let locale =
        parse_locale(include_str!("../../../tests/fixtures/golden/locale/ru.lgl")).expect("locale");

    let files = generate_typescript_project_files(
        &lower_schema(&schema),
        &[TypeScriptLocaleModule {
            locale: "ru".to_owned(),
            module: lower_locale(&locale),
        }],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: Some("ru".to_owned()),
        },
    )
    .expect("project files");

    for file in files {
        let snapshot_path = format!("tests/fixtures/golden/snapshots/ts/{}", file.path);
        assert_snapshot(&snapshot_path, &file.contents);
    }
}

fn assert_snapshot(path: &str, snapshot: &str) {
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        let path = repo_root().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create snapshot dir");
        }
        fs::write(path, snapshot).expect("write snapshot");
    }

    let expected = fs::read_to_string(repo_root().join(path)).expect("read snapshot");
    assert_eq!(snapshot, expected);
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn project_codegen_owns_multilocale_index_files() {
    use crate::{
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions,
    };
    use linguini_ir::IrModule;

    let files = generate_typescript_project_files(
        &IrModule::default(),
        &[
            TypeScriptLocaleModule {
                locale: "en".to_owned(),
                module: IrModule::default(),
            },
            TypeScriptLocaleModule {
                locale: "ru".to_owned(),
                module: IrModule::default(),
            },
        ],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: Some("en".to_owned()),
        },
    )
    .expect("project codegen");

    let index = files
        .iter()
        .find(|file| file.path == "index.ts")
        .expect("index.ts");
    assert!(index
        .contents
        .contains("import locale_en from \"./locales/en\";"));
    assert!(index
        .contents
        .contains("import locale_ru from \"./locales/ru\";"));
    assert!(index.contents.contains("ru: locale_ru"));
    assert_eq!(
        files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        [
            "shared.ts",
            "shared.d.ts",
            "locales/en.ts",
            "locales/en.d.ts",
            "locales/ru.ts",
            "locales/ru.d.ts",
            "index.ts",
            "index.d.ts",
        ]
    );
    for forbidden in [
        ["coo", "kie"].concat(),
        ["localize", "Href"].concat(),
        ["Middle", "ware"].concat(),
    ] {
        assert!(!index.contents.contains(&forbidden));
    }
}

#[test]
fn project_runtime_index_snapshot_is_stable() {
    use linguini_ir::IrModule;

    let files = generate_typescript_project_files(
        &IrModule::default(),
        &[
            TypeScriptLocaleModule {
                locale: "en".to_owned(),
                module: IrModule::default(),
            },
            TypeScriptLocaleModule {
                locale: "ru".to_owned(),
                module: IrModule::default(),
            },
        ],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: Some("en".to_owned()),
        },
    )
    .expect("project codegen");

    let index = files
        .iter()
        .find(|file| file.path == "index.ts")
        .expect("index.ts");
    assert_snapshot(
        "tests/fixtures/golden/snapshots/ts-runtime/index.ts",
        &index.contents,
    );
}

#[test]
fn project_codegen_filters_messages_in_tree_shaking_mode() {
    use crate::{
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions,
    };

    let schema =
        lower_schema(&parse_schema("keep()\ndrop()\ngroup { label() help() }\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale("keep = Keep\ndrop = Drop\ngroup {\n  label = Label\n  help = Help\n}\n")
            .expect("locale"),
    );

    let files = generate_typescript_project_files(
        &schema,
        &[TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: true,
            included_messages: vec!["keep".to_owned(), "group.label".to_owned()],
            base_locale: Some("en".to_owned()),
        },
    )
    .expect("project codegen");

    let locale_module = files
        .iter()
        .find(|file| file.path == "locales/en.ts")
        .expect("locale module");
    assert!(locale_module.contents.contains("export function keep()"));
    assert!(!locale_module.contents.contains("export function drop()"));
    assert!(locale_module.contents.contains("label: \"Label\""));
    assert!(!locale_module.contents.contains("help: \"Help\""));
}
