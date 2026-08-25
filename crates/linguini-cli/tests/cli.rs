use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn linguini() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("linguini")
}

#[test]
fn help_is_generated_by_cli_argument_parser() {
    let snapshot = include_str!("../../../tests/fixtures/golden/snapshots/cli-help.txt");

    linguini()
        .arg("--help")
        .assert()
        .success()
        .stdout(snapshot)
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
    assert!(!config.contains("[web]"));
    assert!(!config.contains("framework ="));
    assert!(config.contains("[targets.ts]"));
    assert!(config.contains("out = \"src/generated/linguini\""));
    assert!(config.contains("gitignore = true"));
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
    fs::write(schema_dir.join("delivery.lgs"), "delivery\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .success()
        .stdout(contains("schema files:"))
        .stdout(contains("schema/shop/delivery.lgs [shop.delivery]"))
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
    fs::write(schema_dir.join("broken.lgs"), "delivery(fruit: Fruit\n").expect("schema file");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(contains("Error:"))
        .stderr(contains("schema/shop/broken.lgs"))
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
    fs::write(schema_dir.join("delivery.lgs"), "delivery\n").expect("schema file");
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
    fs::write(schema_dir.join("shop.lgs"), "delivery\ncounted\n").expect("schema file");
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
    fs::write(schema_dir.join("shop.lgs"), "delivery\n").expect("schema file");
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
fn build_rejects_filesystem_and_group_namespace_collision() {
    let project = TempDir::new().expect("temp project");
    linguini()
        .current_dir(project.path())
        .arg("init")
        .assert()
        .success();

    let nested_schema_dir = project.path().join("schema/shop");
    let outer_locale_dir = project.path().join("locales/shop");
    let nested_locale_dir = outer_locale_dir.join("checkout");
    fs::create_dir_all(&nested_schema_dir).expect("nested schema dir");
    fs::create_dir_all(&nested_locale_dir).expect("nested locale dir");
    fs::write(
        project.path().join("schema/shop.lgs"),
        "checkout { title }\n",
    )
    .expect("outer schema");
    fs::write(nested_schema_dir.join("checkout.lgs"), "title\n").expect("nested schema");
    fs::write(
        outer_locale_dir.join("en.lgl"),
        "checkout { title = Outer }\n",
    )
    .expect("outer locale");
    fs::write(nested_locale_dir.join("en.lgl"), "title = Nested\n").expect("nested locale");

    linguini()
        .current_dir(project.path())
        .arg("build")
        .assert()
        .failure()
        .stderr(contains("duplicate message symbol `shop.checkout.title`"));
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
declaration = true
"#,
    )
    .expect("config");

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lgs"), "delivery\ncounted\n").expect("schema file");
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
fn check_and_build_can_deny_project_warnings() {
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
declaration = true
"#,
    )
    .expect("config");

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lgs"), "delivery\ncounted\n").expect("schema file");
    fs::write(
        locale_dir.join("en.lgl"),
        "delivery = Delivered\ncounted = Counted\n",
    )
    .expect("default locale file");
    fs::write(locale_dir.join("ru.lgl"), "delivery = Доставлено\n").expect("secondary locale file");

    for command in ["check", "build"] {
        linguini()
            .current_dir(project.path())
            .args([command, "--deny-warnings"])
            .assert()
            .failure()
            .stderr(contains("Warning:"))
            .stderr(contains(
                "locale `ru` for schema namespace `shop` is missing 1 schema message: `counted`",
            ));
    }

    assert!(!project.path().join("src/generated/linguini").exists());
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
declaration = true
"#,
    )
    .expect("config");

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lgs"), "delivery\ncounted\n").expect("schema file");
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
fn fix_command_applies_fix_type_and_file_scope() {
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
declaration = true
"#,
    )
    .expect("config");

    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lgs"), "delivery\ncounted\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("en locale");
    fs::write(locale_dir.join("ru.lgl"), "delivery = Доставлено\n").expect("ru locale");

    linguini()
        .current_dir(project.path())
        .args(["fix", "--file", "locales/shop/ru.lgl", "--all"])
        .assert()
        .success()
        .stdout(contains("applied missing-messages:shop:ru"))
        .stdout(
            predicates::str::is_match("missing-messages:shop:en")
                .unwrap()
                .not(),
        );

    let en = fs::read_to_string(locale_dir.join("en.lgl")).expect("en locale");
    let ru = fs::read_to_string(locale_dir.join("ru.lgl")).expect("ru locale");
    assert!(!en.contains("counted = TODO"));
    assert!(ru.contains("counted = TODO"));

    linguini()
        .current_dir(project.path())
        .args(["fix", "--type", "missing-messages"])
        .assert()
        .success()
        .stdout(contains("applied missing-messages:shop:en"));

    let en = fs::read_to_string(locale_dir.join("en.lgl")).expect("en locale");
    assert!(en.contains("counted = TODO"));
}

#[test]
fn build_command_generates_typescript_preserves_user_files_and_needs_no_cldr_cache() {
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
    fs::write(schema_dir.join("delivery.lgs"), "delivery(count: Number)\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = {count} deliveries\n").expect("locale file");

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
    let generated_gitignore =
        fs::read_to_string(project.path().join("src/generated/linguini/.gitignore"))
            .expect("generated gitignore");
    assert_eq!(
        generated_gitignore,
        "# Generated by Linguini. Do not edit.\n*\n"
    );

    let user_file = project.path().join("src/generated/linguini/user-notes.txt");
    fs::write(&user_file, "keep me\n").expect("user file");
    linguini()
        .current_dir(project.path())
        .arg("build")
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(user_file).expect("preserved user file"),
        "keep me\n"
    );
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
        schema_dir.join("shop.lgs"),
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
        .stdout(predicates::str::contains("\u{1b}[").not())
        .stdout(contains("locale"))
        .stdout(contains("en"))
        .stdout(contains("message shop.counted"))
        .stdout(contains("fruit"))
        .stdout(contains("apple"))
        .stdout(contains("pear"))
        .stdout(contains("count"))
        .stdout(contains("5"))
        .stdout(contains("=> 1 apple"))
        .stdout(contains("=> 5 apples"))
        .stdout(predicates::str::contains("\"locales\"").not());
}

#[test]
fn check_exposes_machine_diagnostic_formats() {
    linguini()
        .args(["check", "--help"])
        .assert()
        .success()
        .stdout(contains("--format <FORMAT>"))
        .stdout(contains("possible values: human, json, sarif"));
}

#[test]
fn check_json_reports_structured_diagnostics_and_fixes() {
    let project = machine_diagnostic_project();
    let assert = linguini()
        .current_dir(project.path())
        .args(["check", "--format", "json"])
        .assert()
        .failure()
        .stderr("");
    let document: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid JSON diagnostics");

    assert_eq!(document["version"], 1);
    assert_eq!(document["tool"]["name"], "linguini");
    assert_eq!(document["success"], false);
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["code"] == "linguini.syntax"
            && diagnostic["category"] == "syntax"
            && diagnostic["severity"] == "error"
            && diagnostic["path"] == "schema/broken.lgs"
            && diagnostic["range"]["start"]["line"].as_u64().is_some()
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["severity"] == "warning" && diagnostic["path"] == "locales/shop/ru.lgl"
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["related"].as_array().is_some_and(|related| {
            related.iter().any(|location| {
                location["path"] == "locales/shop/en.lgl"
                    && location["range"]["start"]["byteOffset"].as_u64().is_some()
            })
        })
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["fixes"].as_array().is_some_and(|fixes| {
            fixes.iter().any(|fix| {
                fix["kind"] == "replace"
                    && fix["path"].as_str().is_some()
                    && fix["range"]["start"]["line"].as_u64().is_some()
                    && fix["replacement"]
                        .as_str()
                        .is_some_and(|text| !text.is_empty())
            })
        })
    }));
}

#[test]
fn check_sarif_emits_sarif_2_1_results_and_artifact_changes() {
    let project = machine_diagnostic_project();
    let assert = linguini()
        .current_dir(project.path())
        .args(["check", "--format", "sarif"])
        .assert()
        .failure()
        .stderr("");
    let document: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid SARIF JSON");

    assert_eq!(document["version"], "2.1.0");
    assert_eq!(
        document["$schema"],
        "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json"
    );
    let run = &document["runs"][0];
    assert_eq!(run["tool"]["driver"]["name"], "Linguini");
    assert_eq!(run["invocations"][0]["executionSuccessful"], false);
    assert!(run["tool"]["driver"]["rules"]
        .as_array()
        .is_some_and(|rules| rules.iter().any(|rule| rule["id"] == "linguini.syntax")));
    let results = run["results"].as_array().expect("SARIF results");
    assert!(results.iter().any(|result| result["level"] == "error"));
    assert!(results.iter().any(|result| result["level"] == "warning"));
    assert!(results.iter().any(|result| {
        result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"] == "schema/broken.lgs"
            && result["locations"][0]["physicalLocation"]["region"]["startLine"]
                .as_u64()
                .is_some()
    }));
    assert!(results.iter().any(|result| {
        result["relatedLocations"]
            .as_array()
            .is_some_and(|locations| !locations.is_empty())
    }));
    assert!(results.iter().any(|result| {
        result["fixes"].as_array().is_some_and(|fixes| {
            fixes.iter().any(|fix| {
                fix["artifactChanges"][0]["artifactLocation"]["uri"]
                    .as_str()
                    .is_some()
                    && fix["artifactChanges"][0]["replacements"][0]["deletedRegion"]["startLine"]
                        .as_u64()
                        .is_some()
                    && fix["artifactChanges"][0]["replacements"][0]["insertedContent"]["text"]
                        .as_str()
                        .is_some_and(|text| !text.is_empty())
            })
        })
    }));
}

#[test]
fn check_matches_documented_resolvable_type_and_branch_scope() {
    let project = TempDir::new().expect("temp project");
    fs::write(
        project.path().join("linguini.toml"),
        r#"[project]
name = "docs-contract"
default_locale = "en"
locales = ["en"]

[paths]
schema = "schema"
locale = "locales"
"#,
    )
    .expect("config");
    fs::create_dir_all(project.path().join("schema")).expect("schema dir");
    fs::create_dir_all(project.path().join("locales/main")).expect("locale dir");
    fs::write(
        project.path().join("schema/main.lgs"),
        "enum Choice { yes, no }\ndelivery(count: Number)\nunused\n",
    )
    .expect("schema");
    fs::write(
        project.path().join("locales/main/en.lgl"),
        "form Label(Choice) {\n  yes => Yes\n}\n\
         fn choose(Plural, value: String) {\n  _ => {value}\n}\n\
         delivery = {choose(count, count)}\n\
         unused = Not referenced by application source\n",
    )
    .expect("locale");

    let assert = linguini()
        .current_dir(project.path())
        .args(["check", "--format", "json"])
        .assert()
        .failure()
        .stderr("");
    let document: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid JSON diagnostics");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    let codes = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic["code"].as_str())
        .collect::<Vec<_>>();

    assert!(codes.contains(&"linguini.incomplete_match"));
    assert!(codes.contains(&"linguini.type_mismatch"));
    assert!(
        diagnostics.iter().all(|diagnostic| {
            !diagnostic["code"]
                .as_str()
                .is_some_and(|code| code.contains("unused"))
                && !diagnostic["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("unused message"))
        }),
        "application usage is outside analyzer scope: {diagnostics:#?}"
    );
}

#[test]
fn configured_unused_message_analysis_is_project_scoped_and_machine_readable() {
    let project = unused_message_project();
    let assert = linguini()
        .current_dir(project.path())
        .args(["check", "--format", "json"])
        .assert()
        .success()
        .stderr("");
    let document: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid JSON diagnostics");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    let unused = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic["code"] == "unused_message")
        .collect::<Vec<_>>();

    assert_eq!(document["success"], true);
    assert_eq!(unused.len(), 2, "{diagnostics:#?}");
    assert!(unused.iter().all(|diagnostic| {
        diagnostic["category"] == "lint"
            && diagnostic["lintName"] == "unused_message"
            && diagnostic["severity"] == "warning"
            && diagnostic["path"] == "schema/main.lgs"
            && diagnostic["range"]["start"]["line"].as_u64().is_some()
    }));
    let messages = unused
        .iter()
        .filter_map(|diagnostic| diagnostic["message"].as_str())
        .collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|message| message.contains("main.unused")));
    assert!(messages
        .iter()
        .any(|message| message.contains("main.excluded_only")));
    assert!(messages
        .iter()
        .all(|message| !message.contains("main.ignored")));

    linguini()
        .current_dir(project.path())
        .args(["check", "--deny-warnings"])
        .assert()
        .failure()
        .stderr(contains("schema message `main.unused` is not referenced"));
    linguini()
        .current_dir(project.path())
        .args(["build", "--deny-warnings"])
        .assert()
        .failure()
        .stderr(contains("schema message `main.unused` is not referenced"));
    assert!(!project.path().join("build/generated").exists());
}

#[test]
fn configured_unused_message_analysis_is_emitted_in_sarif() {
    let project = unused_message_project();
    let assert = linguini()
        .current_dir(project.path())
        .args(["check", "--format", "sarif"])
        .assert()
        .success()
        .stderr("");
    let document: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid SARIF diagnostics");
    let run = &document["runs"][0];

    assert!(run["tool"]["driver"]["rules"]
        .as_array()
        .is_some_and(|rules| rules.iter().any(|rule| rule["id"] == "unused_message")));
    assert!(run["results"].as_array().is_some_and(|results| {
        results.iter().any(|result| {
            result["ruleId"] == "unused_message"
                && result["level"] == "warning"
                && result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    == "schema/main.lgs"
                && result["locations"][0]["physicalLocation"]["region"]["startLine"]
                    .as_u64()
                    .is_some()
        })
    }));
}

#[test]
fn configured_unused_message_analysis_excludes_typescript_output_automatically() {
    let project = unused_message_project();
    let config_path = project.path().join("linguini.toml");
    let config = fs::read_to_string(&config_path)
        .expect("config")
        .replace("out = \"build/generated\"", "out = \"src/generated\"");
    fs::write(&config_path, config).expect("updated config");
    fs::create_dir_all(project.path().join("src/generated")).expect("generated output dir");
    fs::write(
        project.path().join("src/generated/stale.ts"),
        "l.main.unused();\n",
    )
    .expect("stale generated source");

    let assert = linguini()
        .current_dir(project.path())
        .args(["check", "--format", "json"])
        .assert()
        .success()
        .stderr("");
    let document: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid JSON diagnostics");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("diagnostic array");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["code"] == "unused_message"
            && diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("main.unused"))
    }));
}

#[test]
fn configured_unused_message_analysis_fails_for_a_missing_source_root() {
    let project = unused_message_project();
    fs::remove_dir_all(project.path().join("src")).expect("remove configured source");

    linguini()
        .current_dir(project.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(contains("src"))
        .stderr(contains("NotFound"));
}

#[test]
fn build_json_keeps_stdout_machine_readable_and_generates_files() {
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
    fs::write(schema_dir.join("delivery.lgs"), "delivery\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    let assert = linguini()
        .current_dir(project.path())
        .args(["build", "--format", "json"])
        .assert()
        .success()
        .stderr("");
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("UTF-8 output");
    let document: Value = serde_json::from_str(&stdout).expect("valid JSON build diagnostics");

    assert_eq!(document["success"], true);
    assert_eq!(document["diagnostics"].as_array().map(Vec::len), Some(0));
    assert!(!stdout.contains("build: ok"));
    assert!(project
        .path()
        .join("src/generated/linguini/index.ts")
        .exists());
}

fn unused_message_project() -> TempDir {
    let project = TempDir::new().expect("temp project");
    fs::write(
        project.path().join("linguini.toml"),
        r#"[project]
name = "unused-analysis"
default_locale = "en"
locales = ["en"]

[paths]
schema = "schema"
locale = "locales"

[targets.ts]
out = "build/generated"
declaration = true

[analysis.unused_messages]
sources = ["src"]
exclude = ["src/excluded"]
ignore = ["main.ignored"]
"#,
    )
    .expect("config");
    fs::create_dir_all(project.path().join("schema")).expect("schema dir");
    fs::create_dir_all(project.path().join("locales/main")).expect("locale dir");
    fs::create_dir_all(project.path().join("src/routes")).expect("application dir");
    fs::create_dir_all(project.path().join("src/excluded")).expect("excluded dir");
    fs::write(
        project.path().join("schema/main.lgs"),
        "used\nvalue\ndynamic { one two }\nignored\nexcluded_only\nunused\n",
    )
    .expect("schema");
    fs::write(
        project.path().join("locales/main/en.lgl"),
        "used = Used\nvalue = Value\ndynamic {\n  one = One\n  two = Two\n}\n\
         ignored = Ignored\nexcluded_only = Excluded\nunused = Unused\n",
    )
    .expect("locale");
    fs::write(
        project.path().join("src/routes/+page.svelte"),
        r#"<script lang="ts">
          import { l as messages } from "$lib/generated/linguini/svelte";
          const value = messages.main.value;
          const selected = messages.main.dynamic[key];
        </script>
        <h1>{messages.main.used()}</h1>
        "#,
    )
    .expect("application source");
    fs::write(
        project.path().join("src/excluded/generated.ts"),
        "l.main.excluded_only();\n",
    )
    .expect("excluded application source");
    project
}

fn machine_diagnostic_project() -> TempDir {
    let project = TempDir::new().expect("temp project");
    fs::write(
        project.path().join("linguini.toml"),
        r#"[project]
name = "shop"
default_locale = "en"
locales = ["en", "ru"]

[paths]
schema = "schema"
locale = "locales"
"#,
    )
    .expect("config");
    let schema_dir = project.path().join("schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("shop.lgs"), "delivery\ncounted\n").expect("valid schema");
    fs::write(schema_dir.join("broken.lgs"), "broken(value: String\n").expect("invalid schema");
    fs::write(
        locale_dir.join("en.lgl"),
        "delivery = Delivered\ndelivery = Duplicate\n",
    )
    .expect("default locale");
    fs::write(locale_dir.join("ru.lgl"), "delivery = Доставлено\n").expect("secondary locale");
    project
}
