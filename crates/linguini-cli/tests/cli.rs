use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

fn linguini() -> Command {
    Command::cargo_bin("linguini").expect("linguini binary")
}

#[test]
fn help_is_generated_by_cli_argument_parser() {
    linguini()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Experimental localization toolkit CLI"))
        .stdout(contains("Usage: linguini <COMMAND>"))
        .stdout(contains("init"))
        .stdout(contains("check"))
        .stdout(contains("fix"))
        .stdout(contains("build"))
        .stdout(contains("generate"))
        .stderr("");
}

#[test]
fn no_public_cldr_subcommand_exists() {
    linguini()
        .args(["cldr", "status"])
        .assert()
        .failure()
        .stderr(contains("cldr"));
}

#[test]
fn init_command_creates_project_files_without_cache_config() {
    let project = TempDir::new().expect("temp project");

    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success()
        .stdout(contains("created linguini.toml"))
        .stdout(contains("created schema"))
        .stdout(contains("created locales"));

    let config = fs::read_to_string(project.path().join("linguini.toml")).expect("config");
    assert!(project.path().join("linguini.toml").exists());
    assert!(project.path().join("schema").is_dir());
    assert!(project.path().join("locales").is_dir());
    assert!(!config.contains("cache"));
    assert!(config.contains("[targets.ts]"));
    assert!(config.contains("out = \"src/generated/linguini\""));
}

#[test]
fn check_command_lists_schema_and_locale_files() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .success()
        .stdout(contains("schema files:"))
        .stdout(contains("schema/shop/delivery.lqs [shop.delivery]"))
        .stdout(contains("locale files:"))
        .stdout(contains("locales/shop/delivery/en.lgl [en:shop.delivery]"));
}

#[test]
fn check_command_reports_syntax_diagnostics_on_stderr() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("schema/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::write(schema_dir.join("broken.lqs"), "delivery(fruit: Fruit\n").expect("schema file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(contains("Error:"))
        .stderr(contains("schema/shop/broken.lqs"))
        .stderr(contains("schema syntax error"));
}

#[test]
fn check_command_reports_missing_messages_for_empty_locale_file() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "").expect("locale file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(contains(
            "locale `en` for schema namespace `shop.delivery` is missing 1 schema message: `delivery`",
        ))
        .stderr(contains("locales/shop/delivery/en.lgl"))
        .stderr(contains("Fix: add missing locale message stubs"))
        .stderr(contains("linguini fix missing-messages:shop.delivery:en"))
        .stderr(predicates::str::is_match("locale file contains no declarations").unwrap().not());
}

#[test]
fn check_reports_missing_schema_messages_in_matching_locale_namespace() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lqs"), "delivery()\ncounted()\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(contains(
            "locale `en` for schema namespace `shop` is missing 1 schema message: `counted`",
        ))
        .stderr(contains("locales/shop/en.lgl"));
}

#[test]
fn check_requires_locales_to_follow_schema_namespace_directories() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lqs"), "delivery()\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(contains(
            "required locale file is missing for schema namespace `shop`: `en`",
        ))
        .stderr(contains("expected path: locales/shop/en.lgl"))
        .stderr(contains(
            "locale namespace `<root>` has no matching schema namespace",
        ));
}

#[test]
fn check_command_warns_for_secondary_locale_missing_messages() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        project.path().join("linguini.toml"),
        r#"[project]
name = "shop"
default_locale = "en"
locales = ["en", "ru"]

[paths]
schema = "schema"
locale = "locales"

[targets.ts]
out = "src/generated/linguini"
module = "esm"
declaration = true
"#,
    )
    .expect("config");

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lqs"), "delivery()\ncounted()\n").expect("schema file");
    fs::write(
        locale_dir.join("en.lgl"),
        "delivery = Delivered\ncounted = Counted\n",
    )
    .expect("default locale file");
    fs::write(locale_dir.join("ru.lgl"), "delivery = Доставлено\n").expect("secondary locale file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .success()
        .stdout(contains("Warning:"))
        .stdout(contains(
            "locale `ru` for schema namespace `shop` is missing 1 schema message: `counted`",
        ))
        .stdout(contains("Fix: add missing locale message stubs"))
        .stdout(contains("linguini fix missing-messages:shop:ru"));
}

#[test]
fn fix_command_applies_missing_locale_and_message_stubs() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        project.path().join("linguini.toml"),
        r#"[project]
name = "shop"
default_locale = "en"
locales = ["en", "ru"]

[paths]
schema = "schema"
locale = "locales"

[targets.ts]
out = "src/generated/linguini"
module = "esm"
declaration = true
"#,
    )
    .expect("config");

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lqs"), "delivery()\ncounted()\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    linguini()
        .current_dir(project.path())
        .args(["fix", "--all"])
        .assert()
        .success()
        .stdout(contains("applied missing-messages:shop:en"))
        .stdout(contains("applied missing-locale:shop:ru"));

    let en = fs::read_to_string(locale_dir.join("en.lgl")).expect("en locale");
    let ru = fs::read_to_string(locale_dir.join("ru.lgl")).expect("ru locale");
    assert!(en.contains("counted = TODO"));
    assert!(ru.contains("delivery = TODO"));
    assert!(ru.contains("counted = TODO"));
}

#[test]
fn build_command_generates_typescript_and_does_not_require_cldr_cache() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lqs"), "delivery(count: Number)\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = {count} deliveries\n").expect("locale file");

    let stale_file = project.path().join("src/generated/linguini/stale.ts");
    fs::create_dir_all(stale_file.parent().expect("stale parent")).expect("generated dir");
    fs::write(&stale_file, "export const stale = true;\\n").expect("stale file");

    linguini()
        .current_dir(project.path())
        .arg("build")
        .assert()
        .success()
        .stdout(contains("generated files:"))
        .stdout(contains("src/generated/linguini/locales/en.ts"))
        .stdout(contains("build: ok"));

    assert!(project
        .path()
        .join("src/generated/linguini/locales/en.ts")
        .exists());
    assert!(project
        .path()
        .join("src/generated/linguini/index.ts")
        .exists());
    assert!(!stale_file.exists());
    assert!(!project.path().join(".linguini/cache").exists());
}

#[test]
fn generate_command_outputs_rendered_locale_matrix() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(
        schema_dir.join("shop.lqs"),
        "enum Fruit {\n  apple\n  pear\n}\ncounted(count: Number, fruit: Fruit)\n",
    )
    .expect("schema file");
    fs::write(
        locale_dir.join("en.lgl"),
        "impl Fruit {\n  apple {\n    form gen(Plural) {\n      one => apple\n      _ => apples\n    }\n  }\n  pear {\n    form gen(Plural) {\n      one => pear\n      _ => pears\n    }\n  }\n}\ncounted = {count} {fruit.gen(count)}\n",
    )
    .expect("locale file");

    linguini()
        .current_dir(project.path())
        .arg("generate")
        .assert()
        .success()
        .stdout(contains("\u{1b}["))
        .stdout(contains("locale"))
        .stdout(contains("en"))
        .stdout(contains("fruit"))
        .stdout(contains("apple"))
        .stdout(contains("pear"))
        .stdout(contains("count"))
        .stdout(contains("5"))
        .stdout(contains("\u{1b}[32m=>\u{1b}[0m 1 apple"))
        .stdout(contains("\u{1b}[32m=>\u{1b}[0m 5 apples"))
        .stdout(predicates::str::contains("\"locales\"").not());
}
