use crate::{lower_locale, lower_schema};
use linguini_syntax::{parse_locale, parse_schema};
use std::fs;
use std::path::Path;

#[test]
fn schema_ir_snapshot_is_stable() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lqs"
    ))
    .expect("schema");
    let snapshot = format!("{:#?}", lower_schema(&schema));

    assert_snapshot(
        "tests/fixtures/golden/snapshots/ir-schema-shop.txt",
        &snapshot,
    );
}

#[test]
fn locale_ir_snapshot_is_stable() {
    let locale =
        parse_locale(include_str!("../../../tests/fixtures/golden/locale/ru.lgl")).expect("locale");
    let snapshot = format!("{:#?}", lower_locale(&locale));

    assert_snapshot(
        "tests/fixtures/golden/snapshots/ir-locale-ru.txt",
        &snapshot,
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
