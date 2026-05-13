use super::{build_project, check_project, init_project, Cli};
use clap::CommandFactory;
use linguini_test_support::temp_project_dir;
use std::fs;

#[test]
fn cli_argument_parser_is_clap_backed() {
    let command = Cli::command();
    let subcommands: Vec<_> = command
        .get_subcommands()
        .map(|command| command.get_name().to_owned())
        .collect();

    assert!(subcommands.contains(&"init".to_owned()));
    assert!(subcommands.contains(&"check".to_owned()));
    assert!(subcommands.contains(&"fix".to_owned()));
    assert!(subcommands.contains(&"build".to_owned()));
    assert!(!subcommands.contains(&"cldr".to_owned()));
}

#[test]
fn init_creates_valid_project() {
    let project = temp_project_dir("init_creates_valid_project");

    init_project(project.path()).expect("init project");

    assert!(project.path().join("linguini.toml").exists());
    assert!(project.path().join("schema").is_dir());
    assert!(project.path().join("locales").is_dir());
    let config = fs::read_to_string(project.path().join("linguini.toml")).expect("config");
    assert!(!config.contains("cache"));
    assert!(config.contains("[targets.ts]"));
    assert!(config.contains("out = \"src/generated/linguini\""));
}

#[test]
fn check_lists_discovered_files() {
    let project = temp_project_dir("check_lists_discovered_files");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    let output = check_project(project.path()).expect("check project");

    assert!(output.contains("schema/shop/delivery.lqs [shop.delivery]"));
    assert!(output.contains("locales/shop/delivery/en.lgl [en:shop.delivery]"));
}

#[test]
fn check_reports_schema_syntax_diagnostics() {
    let project = temp_project_dir("check_reports_schema_syntax_diagnostics");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::write(schema_dir.join("broken.lqs"), "delivery(fruit: Fruit\n").expect("schema file");

    let error = check_project(project.path()).expect_err("check fails");
    let rendered = error.to_string();

    assert!(rendered.contains("Error:"));
    assert!(rendered.contains("schema/shop/broken.lqs"));
    assert!(rendered.contains("schema syntax error"));
}

#[test]
fn check_reports_missing_schema_message_for_empty_locale_file() {
    let project = temp_project_dir("check_reports_missing_schema_message");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "").expect("locale file");

    let error = check_project(project.path()).expect_err("check fails on missing message");
    let rendered = error.to_string();

    assert!(rendered.contains(
        "locale `en` for schema namespace `shop.delivery` is missing 1 schema message: `delivery`"
    ));
    assert!(rendered.contains("locales/shop/delivery/en.lgl"));
    assert!(rendered.contains("Fix: add missing locale message stubs"));
    assert!(!rendered.contains(",- ["));
    assert!(!rendered.contains("locale file contains no declarations"));
}

#[test]
fn check_rejects_root_locale_file_for_schema_namespace() {
    let project = temp_project_dir("check_rejects_root_locale_file");
    init_project(project.path()).expect("init project");

    fs::write(project.path().join("schema/shop.lqs"), "delivery()\n").expect("schema file");
    fs::write(
        project.path().join("locales/en.lgl"),
        "delivery = Delivered\n",
    )
    .expect("locale file");

    let error = check_project(project.path()).expect_err("check fails on misplaced locale");
    let rendered = error.to_string();

    assert!(rendered.contains("required locale file is missing for schema namespace `shop`: `en`"));
    assert!(rendered.contains("expected path: locales/shop/en.lgl"));
    assert!(rendered.contains("locale namespace `<root>` has no matching schema namespace"));
    assert!(!rendered.contains(",- ["));
}

#[test]
fn check_warns_for_secondary_locale_missing_messages() {
    let project = temp_project_dir("check_warns_for_secondary_locale_missing_messages");
    init_project(project.path()).expect("init project");

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

    let output = check_project(project.path()).expect("secondary locale gaps are warnings");

    assert!(output.contains("Warning:"));
    assert!(output.contains(
        "locale `ru` for schema namespace `shop` is missing 1 schema message: `counted`"
    ));
    assert!(output.contains("Fix: add missing locale message stubs"));
    assert!(output.contains("linguini fix missing-messages:shop:ru"));
}

#[test]
fn build_generates_typescript_project_files_without_cldr_cache() {
    let project = temp_project_dir("build_generates_typescript");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lqs"), "delivery(count: Number)\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = {count} deliveries\n").expect("locale file");

    let stale_file = project.path().join("src/generated/linguini/stale.ts");
    fs::create_dir_all(stale_file.parent().expect("stale parent")).expect("generated dir");
    fs::write(&stale_file, "export const stale = true;\\n").expect("stale file");

    let output = build_project(project.path()).expect("build project");

    assert!(output.contains("schema files:"));
    assert!(output.contains("locale files:"));
    assert!(output.contains("generated files:"));
    assert!(output.contains("src/generated/linguini/locales/en.ts"));
    assert!(output.contains("build: ok"));
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
fn build_replaces_existing_generated_tree() {
    let project = temp_project_dir("build_replaces_existing_generated_tree");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lqs"), "delivery(count: Number)\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = {count} deliveries\n").expect("locale file");

    build_project(project.path()).expect("initial build");
    let out_dir = project.path().join("src/generated/linguini");
    let index_path = out_dir.join("index.ts");
    let locale_path = out_dir.join("locales/en.ts");
    let original_index = fs::read_to_string(&index_path).expect("read generated index");
    let original_locale = fs::read_to_string(&locale_path).expect("read generated locale");

    fs::write(&index_path, "// user edit that must be replaced\n").expect("corrupt index");
    fs::write(&locale_path, "// user edit that must be replaced\n").expect("corrupt locale");
    fs::write(out_dir.join("stale.ts"), "export const stale = true;\n").expect("stale file");
    fs::create_dir_all(out_dir.join("obsolete/nested")).expect("obsolete dir");
    fs::write(out_dir.join("obsolete/nested/file.ts"), "obsolete\n").expect("obsolete file");

    let output = build_project(project.path()).expect("second build");

    assert!(output.contains("replaced generated tree: src/generated/linguini"));
    assert_eq!(
        fs::read_to_string(&index_path).expect("read regenerated index"),
        original_index
    );
    assert_eq!(
        fs::read_to_string(&locale_path).expect("read regenerated locale"),
        original_locale
    );
    assert!(!out_dir.join("stale.ts").exists());
    assert!(!out_dir.join("obsolete").exists());
}
