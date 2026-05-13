use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

fn linguini() -> Command {
    Command::cargo_bin("linguini").expect("linguini binary")
}

#[test]
fn init_command_creates_project_files() {
    let project = TempDir::new().expect("temp project");

    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success()
        .stdout(contains("created linguini.toml"))
        .stdout(contains("created linguini/schema"))
        .stdout(contains("created linguini/locale"));

    assert!(project.path().join("linguini.toml").exists());
    assert!(project.path().join("linguini/schema").is_dir());
    assert!(project.path().join("linguini/locale").is_dir());
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
fn cldr_status_reports_missing_cache() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    linguini()
        .current_dir(project.path())
        .args(["cldr", "status"])
        .assert()
        .success()
        .stdout(contains("usable: false"))
        .stdout(contains("plurals: false"));
}
