use crate::{ensure_no_unresolved_references, lower_locale, lower_schema};
use linguini_syntax::{parse_locale, parse_schema};
use std::fs;
use std::path::Path;

#[test]
fn schema_ir_snapshot_is_stable() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lgs"
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

#[test]
fn ir_reference_validation_accepts_golden_delivery_fixture() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lgs"
    ))
    .expect("schema");
    let locale =
        parse_locale(include_str!("../../../tests/fixtures/golden/locale/ru.lgl")).expect("locale");

    ensure_no_unresolved_references(&lower_schema(&schema), &lower_locale(&locale))
        .expect("references resolved");
}

#[test]
fn ir_reference_validation_reports_unknown_message_and_placeholder() {
    let schema = parse_schema("known(name: String)\n").expect("schema");
    let locale = parse_locale("missing = {unknown}\n").expect("locale");
    let errors = ensure_no_unresolved_references(&lower_schema(&schema), &lower_locale(&locale))
        .expect_err("unresolved references");

    assert!(errors
        .iter()
        .any(|error| error.message == "unresolved message `missing`"));
}

#[test]
fn ir_reference_validation_reports_unknown_placeholder_root() {
    let schema = parse_schema("known(name: String)\n").expect("schema");
    let locale = parse_locale("known = {unknown}\n").expect("locale");
    let errors = ensure_no_unresolved_references(&lower_schema(&schema), &lower_locale(&locale))
        .expect_err("unresolved references");

    assert!(errors
        .iter()
        .any(|error| error.message == "unresolved reference `unknown`"));
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
