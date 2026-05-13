use assert_cmd::Command;
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
        .stdout(contains("created linguini/schema"))
        .stdout(contains("created linguini/locale"));

    let config = fs::read_to_string(project.path().join("linguini.toml")).expect("config");
    assert!(project.path().join("linguini.toml").exists());
    assert!(project.path().join("linguini/schema").is_dir());
    assert!(project.path().join("linguini/locale").is_dir());
    assert!(!config.contains("cache"));
}

#[test]
fn check_command_lists_schema_and_locale_files() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let schema_dir = project.path().join("linguini/schema/shop");
    let locale_dir = project.path().join("linguini/locale/shop/delivery");
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
            "linguini/schema/shop/delivery.lqs [shop.delivery]",
        ))
        .stdout(contains("locale files:"))
        .stdout(contains(
            "linguini/locale/shop/delivery/en.lgl [en:shop.delivery]",
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

    let schema_dir = project.path().join("linguini/schema/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::write(schema_dir.join("broken.lqs"), "delivery(fruit: Fruit\n").expect("schema file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(contains("Error:"))
        .stderr(contains("linguini/schema/shop/broken.lqs"))
        .stderr(contains("schema syntax error"));
}

#[test]
fn build_command_does_not_require_cldr_cache() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    linguini()
        .current_dir(project.path())
        .arg("build")
        .assert()
        .success()
        .stdout(contains("build: ok"));

    assert!(!project.path().join(".linguini/cache").exists());
}
