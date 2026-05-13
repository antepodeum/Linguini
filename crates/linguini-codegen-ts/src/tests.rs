use crate::{generate_typescript_module, TypeScriptOptions};
use linguini_ir::{lower_locale, lower_schema};
use linguini_syntax::{parse_locale, parse_schema};
use std::fs;
use std::path::Path;

#[test]
fn generated_module_snapshot_is_stable() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lqs"
    ))
    .expect("schema");
    let locale =
        parse_locale(include_str!("../../../tests/fixtures/golden/locale/ru.lgl")).expect("locale");

    let output = generate_typescript_module(
        &lower_schema(&schema),
        &lower_locale(&locale),
        &TypeScriptOptions {
            plural_function: "pluralRu".to_owned(),
        },
    );

    assert_snapshot(
        "tests/fixtures/golden/snapshots/codegen-ts-module-ru.ts",
        &output,
    );
}

fn assert_snapshot(path: &str, snapshot: &str) {
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        fs::write(repo_root().join(path), snapshot).expect("write snapshot");
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
