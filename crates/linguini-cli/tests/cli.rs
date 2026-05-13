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
        .stdout(contains("build"))
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
        .stdout(contains(
            "schema/shop/delivery.lqs [shop.delivery]",
        ))
        .stdout(contains("locale files:"))
        .stdout(contains(
            "locales/shop/delivery/en.lgl [en:shop.delivery]",
        ));
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
        .stderr(contains("missing locale implementation for schema message `delivery`"))
        .stderr(contains("locales/shop/delivery/en.lgl"))
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
        .stderr(contains("missing locale implementation for schema message `counted`"))
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
        .stderr(contains("missing locale file for schema namespace `shop` and locale `en`"))
        .stderr(contains("create locales/shop/en.lgl"))
        .stderr(contains("locale namespace `<root>` does not match any schema file"));
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
    fs::write(schema_dir.join("delivery.lqs"), "delivery(count: Number)\n")
        .expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = {count} deliveries\n")
        .expect("locale file");

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
    assert!(project.path().join("src/generated/linguini/index.ts").exists());
    assert!(!project.path().join(".linguini/cache").exists());
}
