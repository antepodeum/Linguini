use crate::{generate_plural_function, generate_typescript_files, TypeScriptOptions};
use linguini_cldr::load_plural_rules;
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

    let files = generate_typescript_files(
        &lower_schema(&schema),
        &lower_locale(&locale),
        &TypeScriptOptions {
            locale: "ru".to_owned(),
            plural_function: "pluralRu".to_owned(),
            plural_import: None,
            plural_source: Some(generate_plural_function(
                "pluralRu",
                &load_plural_rules(PLURALS, "ru").expect("plural rules"),
            )),
        },
    );

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

const PLURALS: &str = r#"
{
  "supplemental": {
    "plurals-type-cardinal": {
      "ru": {
        "pluralRule-count-one": "v = 0 and i % 10 = 1 and i % 100 != 11",
        "pluralRule-count-few": "v = 0 and i % 10 = 2..4 and i % 100 != 12..14",
        "pluralRule-count-many": "v = 0 and i % 10 = 0 or v = 0 and i % 10 = 5..9 or v = 0 and i % 100 = 11..14",
        "pluralRule-count-other": ""
      }
    }
  }
}
"#;
