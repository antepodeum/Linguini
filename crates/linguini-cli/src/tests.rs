use super::{build_project, check_project, init_project, project::generate_project_data, Cli};
use clap::CommandFactory;
use std::fs;
use tempfile::{Builder, TempDir};

fn temp_project_dir(name: &str) -> std::io::Result<TempDir> {
    Builder::new()
        .prefix(&format!("linguini-{name}-"))
        .tempdir()
}

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
    assert!(subcommands.contains(&"generate".to_owned()));
    assert!(subcommands.contains(&"format".to_owned()));
    assert!(subcommands.contains(&"lsp".to_owned()));
    assert!(!subcommands.contains(&"cldr".to_owned()));
}

#[test]
fn init_creates_valid_project() {
    let project = temp_project_dir("init_creates_valid_project").expect("create temporary project");

    init_project(project.path()).expect("init project");

    assert!(project.path().join("linguini.toml").exists());
    assert!(project.path().join("schema").is_dir());
    assert!(project.path().join("locales").is_dir());
    let config = fs::read_to_string(project.path().join("linguini.toml")).expect("config");
    assert!(!config.contains("cache"));
    assert!(!config.contains("[web]"));
    assert!(!config.contains("framework ="));
    assert!(config.contains("[targets.ts]"));
    assert!(config.contains("out = \"src/generated/linguini\""));
    assert!(config.contains("gitignore = true"));
}

#[test]
fn init_reports_existing_project_items_truthfully() {
    let project = temp_project_dir("init_reports_existing").expect("create temporary project");

    init_project(project.path()).expect("initial init");
    let output = init_project(project.path()).expect("second init");

    assert!(output.contains("kept existing linguini.toml"));
    assert!(output.contains("kept existing schema"));
    assert!(output.contains("kept existing locales"));
    assert!(!output.contains("created"));
}

#[test]
fn check_lists_discovered_files() {
    let project =
        temp_project_dir("check_lists_discovered_files").expect("create temporary project");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lgs"), "delivery\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

    let output = check_project(project.path()).expect("check project");

    assert!(output.contains("schema/shop/delivery.lgs [shop.delivery]"));
    assert!(output.contains("locales/shop/delivery/en.lgl [en:shop.delivery]"));
}

#[test]
fn format_command_formats_discovered_project_files() {
    let project = temp_project_dir("format_command_formats_discovered_project_files")
        .expect("create temporary project");
    init_project(project.path()).expect("init project");

    fs::write(
        project.path().join("schema/shop.lgs"),
        "delivery(count:Number)\n",
    )
    .expect("schema file");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(locale_dir.join("en.lgl"), "delivery={count} deliveries\n").expect("locale file");

    let output = super::run(vec!["format".to_owned()], Ok(project.path().to_path_buf()))
        .expect("format command");

    assert!(output.contains("schema/shop.lgs"));
    assert_eq!(
        fs::read_to_string(project.path().join("schema/shop.lgs")).expect("schema"),
        "delivery(count: Number)\n"
    );
    assert_eq!(
        fs::read_to_string(locale_dir.join("en.lgl")).expect("locale"),
        "delivery = {count} deliveries\n"
    );
}

#[test]
fn format_command_rejects_files_outside_project() {
    let project = temp_project_dir("format_rejects_outside").expect("create temporary project");
    init_project(project.path()).expect("init project");
    let outside = tempfile::TempDir::new().expect("outside");
    let outside_file = outside.path().join("outside.lgs");
    fs::write(&outside_file, "delivery(count:Number)\n").expect("outside source");

    let error = super::run(
        vec![
            "format".to_owned(),
            outside_file.to_string_lossy().into_owned(),
        ],
        Ok(project.path().to_path_buf()),
    )
    .expect_err("outside format target must fail");

    assert!(error.to_string().contains("outside the project root"));
    assert_eq!(
        fs::read_to_string(outside_file).expect("outside source"),
        "delivery(count:Number)\n"
    );
}

#[cfg(unix)]
#[test]
fn format_command_rejects_symlink_targets() {
    use std::os::unix::fs::symlink;

    let project = temp_project_dir("format_rejects_symlink").expect("create temporary project");
    init_project(project.path()).expect("init project");
    let outside = tempfile::TempDir::new().expect("outside");
    let outside_file = outside.path().join("outside.lgs");
    fs::write(&outside_file, "delivery(count:Number)\n").expect("outside source");
    symlink(&outside_file, project.path().join("linked.lgs")).expect("symlink");

    let error = super::run(
        vec!["format".to_owned(), "linked.lgs".to_owned()],
        Ok(project.path().to_path_buf()),
    )
    .expect_err("symlink format target must fail");

    assert!(error.to_string().contains("symbolic-link target"));
    assert_eq!(
        fs::read_to_string(outside_file).expect("outside source"),
        "delivery(count:Number)\n"
    );
}

#[cfg(unix)]
#[test]
fn format_command_preserves_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let project =
        temp_project_dir("format_preserves_permissions").expect("create temporary project");
    init_project(project.path()).expect("init project");
    let source = project.path().join("schema/permissions.lgs");
    fs::write(&source, "delivery(count:Number)\n").expect("source");
    fs::set_permissions(&source, fs::Permissions::from_mode(0o640)).expect("permissions");

    super::run(
        vec!["format".to_owned(), "schema/permissions.lgs".to_owned()],
        Ok(project.path().to_path_buf()),
    )
    .expect("format");

    assert_eq!(
        fs::metadata(source).expect("metadata").permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn check_reports_schema_syntax_diagnostics() {
    let project = temp_project_dir("check_reports_schema_syntax_diagnostics")
        .expect("create temporary project");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::write(schema_dir.join("broken.lgs"), "delivery(fruit: Fruit\n").expect("schema file");

    let error = check_project(project.path()).expect_err("check fails");
    let rendered = error.to_string();

    assert!(rendered.contains("Error:"));
    assert!(rendered.contains("schema/shop/broken.lgs"));
    assert!(rendered.contains("schema syntax error"));
}

#[test]
fn check_keeps_independent_project_diagnostics_after_syntax_errors() {
    let project = temp_project_dir("check_keeps_independent_project_diagnostics")
        .expect("create temporary project");
    init_project(project.path()).expect("init project");

    fs::write(project.path().join("schema/shop.lgs"), "delivery\n").expect("schema file");
    fs::write(
        project.path().join("schema/broken.lgs"),
        "broken(value: String\n",
    )
    .expect("broken schema");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(locale_dir.join("en.lgl"), "").expect("empty locale");

    let error = check_project(project.path()).expect_err("project must fail");
    let rendered = error.to_string();
    assert!(rendered.contains("schema syntax error"), "{rendered}");
    assert!(
        rendered.contains("locale `en` for schema namespace `shop` is missing 1 schema message"),
        "{rendered}"
    );
}

#[test]
fn check_blocks_locale_semantic_errors() {
    let project =
        temp_project_dir("check_blocks_locale_semantic_errors").expect("create temporary project");
    init_project(project.path()).expect("init project");

    fs::write(project.path().join("schema/shop.lgs"), "delivery\n").expect("schema file");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(
        locale_dir.join("en.lgl"),
        "form Count(Plural) {\n  one => item\n}\ndelivery = Delivered\n",
    )
    .expect("locale file");

    let error = check_project(project.path()).expect_err("semantic error must block check");
    let rendered = error.to_string();
    assert!(
        rendered.contains("function `Count` is missing required `other` branch"),
        "{rendered}"
    );
}

#[test]
fn check_resolves_schema_enums_in_locale_semantics() {
    let project = temp_project_dir("check_resolves_schema_enums_in_locale_semantics")
        .expect("create temporary project");
    init_project(project.path()).expect("init project");

    fs::write(
        project.path().join("schema/shop.lgs"),
        "enum Fruit { apple }\ndelivery(fruit: Fruit, count: Number)\n",
    )
    .expect("schema file");
    let locale_dir = project.path().join("locales/shop");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(
        locale_dir.join("en.lgl"),
        "impl Fruit {\n  apple {\n    form nom(Plural) {\n      one => apple\n      _ => apples\n    }\n  }\n}\ndelivery = {count} {fruit.nom(count)}\n",
    )
    .expect("locale file");

    check_project(project.path()).expect("schema-backed locale semantics are valid");
}

#[test]
fn check_reports_missing_schema_message_for_empty_locale_file() {
    let project =
        temp_project_dir("check_reports_missing_schema_message").expect("create temporary project");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lgs"), "delivery\n").expect("schema file");
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
    let project =
        temp_project_dir("check_rejects_root_locale_file").expect("create temporary project");
    init_project(project.path()).expect("init project");

    fs::write(project.path().join("schema/shop.lgs"), "delivery\n").expect("schema file");
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
    let project = temp_project_dir("check_warns_for_secondary_locale_missing_messages")
        .expect("create temporary project");
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
    let project = temp_project_dir("build_generates_typescript").expect("create temporary project");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lgs"), "delivery(count: Number)\n").expect("schema file");
    fs::write(locale_dir.join("en.lgl"), "delivery = {count} deliveries\n").expect("locale file");

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
    let generated_gitignore =
        fs::read_to_string(project.path().join("src/generated/linguini/.gitignore"))
            .expect("generated gitignore");
    assert_eq!(
        generated_gitignore,
        "# Generated by Linguini. Do not edit.\n*\n"
    );
    let generated_locale =
        fs::read_to_string(project.path().join("src/generated/linguini/locales/en.ts"))
            .expect("read generated locale");
    assert!(generated_locale.contains("import { shop } from \"./en/shop\";"));
    assert!(generated_locale.contains("  shop,"));
    let generated_shop = fs::read_to_string(
        project
            .path()
            .join("src/generated/linguini/locales/en/shop.ts"),
    )
    .expect("read generated shop locale");
    assert!(generated_shop.contains("export const shop = {"));
    assert!(generated_shop.contains("  delivery: {"));
    assert!(generated_shop.contains(
        "    delivery: (...__lgl_args: [count: number | bigint | string] | [args: { count: number | bigint | string }]) =>"
    ));
    assert!(generated_shop
        .contains("normalizeMessageArgs(__lgl_args, [\"count\"]) as [number | bigint | string]"));
    assert!(project
        .path()
        .join("src/generated/linguini/.linguini-generated-manifest")
        .is_file());
    assert!(!project.path().join(".linguini/cache").exists());
}

#[test]
fn build_replaces_owned_files_and_preserves_unowned_files() {
    let project = temp_project_dir("build_replaces_existing_generated_tree")
        .expect("create temporary project");
    init_project(project.path()).expect("init project");

    let schema_dir = project.path().join("schema/shop");
    let locale_dir = project.path().join("locales/shop/delivery");
    fs::create_dir_all(&schema_dir).expect("schema dir");
    fs::create_dir_all(&locale_dir).expect("locale dir");
    fs::write(schema_dir.join("delivery.lgs"), "delivery(count: Number)\n").expect("schema file");
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
    assert_eq!(
        fs::read_to_string(out_dir.join("stale.ts")).expect("unowned file"),
        "export const stale = true;\n"
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("obsolete/nested/file.ts")).expect("nested unowned file"),
        "obsolete\n"
    );
}

#[test]
fn bundler_artifacts_are_deterministic_fallback_aware_and_transaction_owned() {
    let long_message = "message_name_longer_than_one_chunk";
    let project = temp_project_dir("bundler_artifacts").expect("create temporary project");
    fs::create_dir_all(project.path().join("schema")).expect("schema dir");
    fs::create_dir_all(project.path().join("locales/a")).expect("locale a dir");
    fs::create_dir_all(project.path().join("locales/b")).expect("locale b dir");
    fs::write(
        project.path().join("linguini.toml"),
        r#"
[project]
name = "bundler-artifacts"
default_locale = "en"
locales = ["en", "fr"]

[paths]
schema = "schema"
locale = "locales"

[targets.ts]
out = "src/generated/linguini"
declaration = false
gitignore = false
tree_shaking = false
"#,
    )
    .expect("config");
    fs::write(
        project.path().join("schema/a.lgs"),
        "enum Color {\n  red\n}\nfirst(color: Color)\nformatted(value: Number)\n",
    )
    .expect("schema a");
    fs::write(
        project.path().join("schema/b.lgs"),
        format!("{long_message}\n"),
    )
    .expect("schema b");
    fs::write(
        project.path().join("locales/a/en.lgl"),
        "first = First\nformatted = Number {value @number}\n",
    )
    .expect("locale a en");
    fs::write(
        project.path().join("locales/a/fr.lgl"),
        "first = Premier\nformatted = Nombre {value @number}\n",
    )
    .expect("locale a fr");
    fs::write(
        project.path().join("locales/b/en.lgl"),
        format!("{long_message} = Second\n"),
    )
    .expect("locale b en");

    build_project(project.path()).expect("first build");
    let out = project.path().join("src/generated/linguini");
    let manifest_path = out.join("bundler/manifest.json");
    let first_manifest = fs::read_to_string(&manifest_path).expect("manifest");
    let manifest: serde_json::Value = serde_json::from_str(&first_manifest).expect("manifest JSON");
    assert_eq!(manifest["version"], 1);
    assert!(manifest.get("applications").is_none());
    assert!(manifest.get("runtime_helpers").is_none());
    assert_eq!(
        manifest
            .as_object()
            .expect("legacy manifest object")
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "base_locale",
            "configured_locales",
            "effective_locales",
            "messages",
            "sources",
            "version",
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(manifest["base_locale"], "en");
    assert_eq!(
        manifest["configured_locales"],
        serde_json::json!(["en", "fr"])
    );
    assert_eq!(
        manifest["effective_locales"],
        serde_json::json!(["en", "fr"])
    );
    assert_eq!(manifest["messages"]["a.first"]["arity"], 1);
    let long_canonical_message = format!("b.{long_message}");
    assert_eq!(manifest["messages"][&long_canonical_message]["arity"], 0);

    let source_ids = manifest["sources"]
        .as_array()
        .expect("source table")
        .iter()
        .map(|source| {
            (
                source["path"].as_str().expect("source path"),
                source["id"].as_u64().expect("source id"),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert!(source_ids.keys().all(|path| {
        !path.starts_with('/')
            && !path.contains('\\')
            && path
                .split('/')
                .all(|component| !component.is_empty() && component != "." && component != "..")
    }));
    let fallback_ids = manifest["messages"][&long_canonical_message]["locales"]["fr"]["source_ids"]
        .as_array()
        .expect("fallback ids")
        .iter()
        .map(|id| id.as_u64().expect("numeric source id"))
        .collect::<Vec<_>>();
    assert_eq!(
        fallback_ids,
        [source_ids["schema/b.lgs"], source_ids["locales/b/en.lgl"]]
    );

    let module = manifest["messages"]["a.first"]["locales"]["fr"]["module"]
        .as_str()
        .expect("module path");
    let module_code = fs::read_to_string(out.join(module)).expect("module");
    let file_name = std::path::Path::new(module)
        .file_name()
        .expect("module filename")
        .to_string_lossy();
    assert!(module_code.contains("from \"../../../shared\""));
    assert!(module_code.ends_with(&format!("//# sourceMappingURL={file_name}.map\n")));
    let formatted_module = manifest["messages"]["a.formatted"]["locales"]["fr"]["module"]
        .as_str()
        .expect("formatted module");
    let formatted_code = fs::read_to_string(out.join(formatted_module)).expect("formatted module");
    assert!(formatted_code.contains("from \"../../../locales/fr/_runtime\""));
    for helper in [
        "function formatNumber(",
        "function formatCurrency(",
        "function formatDate(",
        "function plural",
    ] {
        assert!(
            !formatted_code.contains(helper),
            "unexpected helper body: {helper}"
        );
    }
    let source_map = fs::read_to_string(out.join(format!("{module}.map"))).expect("source map");
    let source_map: serde_json::Value = serde_json::from_str(&source_map).expect("source map JSON");
    assert_eq!(source_map["file"], file_name.as_ref());

    let actual_sources = manifest["sources"]
        .as_array()
        .expect("manifest sources")
        .iter()
        .map(|source| {
            project
                .path()
                .join(source["path"].as_str().expect("manifest source path"))
                .canonicalize()
                .expect("actual source")
        })
        .collect::<std::collections::BTreeSet<_>>();
    for message in manifest["messages"]
        .as_object()
        .expect("manifest messages")
        .values()
    {
        for locale in message["locales"]
            .as_object()
            .expect("message locales")
            .values()
        {
            let module = locale["module"].as_str().expect("locale module");
            let map_path = out.join(format!("{module}.map"));
            let map: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(&map_path).expect("read emitted source map"),
            )
            .expect("emitted source map JSON");
            for source in map["sources"].as_array().expect("map sources") {
                let source = source.as_str().expect("map source path");
                assert!(!source.starts_with('/'));
                assert!(!source.contains('\\'));
                let resolved = map_path
                    .parent()
                    .expect("map directory")
                    .join(source)
                    .canonicalize()
                    .expect("map source resolves");
                assert!(actual_sources.contains(&resolved));
            }
        }
    }
    let long_module = manifest["messages"][&long_canonical_message]["locales"]["fr"]["module"]
        .as_str()
        .expect("long module");
    assert!(long_module.matches('/').count() >= 4);

    let first_module = module_code;
    let first_map = fs::read_to_string(out.join(format!("{module}.map"))).expect("source map");
    build_project(project.path()).expect("repeat build");
    assert_eq!(
        fs::read_to_string(&manifest_path).expect("repeated manifest"),
        first_manifest
    );
    assert_eq!(
        fs::read_to_string(out.join(module)).expect("repeated module"),
        first_module
    );
    assert_eq!(
        fs::read_to_string(out.join(format!("{module}.map"))).expect("repeated map"),
        first_map
    );
    for locale in ["en", "fr"] {
        let locale_dir = out.join(format!("locales/{locale}"));
        assert!(locale_dir.join("_runtime.ts").is_file());
        assert_eq!(
            fs::read_dir(&locale_dir)
                .expect("locale runtime directory")
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name() == "_runtime.ts")
                .count(),
            1
        );
    }

    let stale_module = manifest["messages"][&long_canonical_message]["locales"]["fr"]["module"]
        .as_str()
        .expect("stale module")
        .to_owned();
    fs::write(
        out.join("user-owned.ts"),
        "export const userOwned = true;\n",
    )
    .expect("user file");
    let selected_config = fs::read_to_string(project.path().join("linguini.toml"))
        .expect("read config")
        .replace(
            "tree_shaking = false",
            "tree_shaking = true\nmessages = [\"a.first\"]",
        );
    fs::write(project.path().join("linguini.toml"), selected_config).expect("selected config");
    build_project(project.path()).expect("tree-shaken build");
    let selected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("selected manifest"))
            .expect("selected manifest JSON");
    assert_eq!(
        selected["messages"]
            .as_object()
            .expect("selected messages")
            .keys()
            .collect::<Vec<_>>(),
        ["a.first"]
    );
    assert!(!out.join(stale_module).exists());
    assert_eq!(
        fs::read_to_string(out.join("user-owned.ts")).expect("user file preserved"),
        "export const userOwned = true;\n"
    );
}

#[test]
fn bundler_semantic_artifacts_are_shared_mapped_and_manifested() {
    let project = temp_project_dir("bundler-semantic-artifacts").expect("project");
    fs::create_dir_all(project.path().join("schema")).expect("schema dir");
    fs::create_dir_all(project.path().join("locales/main")).expect("locale dir");
    fs::create_dir_all(project.path().join("src")).expect("source dir");
    fs::write(
        project.path().join("linguini.toml"),
        r#"
[project]
name = "bundler-semantic-artifacts"
default_locale = "en"
locales = ["en"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "src/generated/linguini"
declaration = false
gitignore = false
tree_shaking = true
messages = ["main.first", "main.second", "main.plain"]
framework = "svelte"
[targets.ts.bundler]
sources = ["src"]
exclude = []
"#,
    )
    .expect("config");
    fs::write(
        project.path().join("schema/main.lgs"),
        "first\nsecond\nplain\n",
    )
    .expect("schema");
    fs::write(
        project.path().join("locales/main/en.lgl"),
        "let shared = Common\nfirst = {shared}\nsecond = {shared}\nplain = Plain\n",
    )
    .expect("locale");

    build_project(project.path()).expect("build");
    let out = project.path().join("src/generated/linguini");
    let manifest_path = out.join("bundler/manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest"))
            .expect("manifest JSON");
    assert_eq!(manifest["version"], 1);

    let semantic = manifest["message_semantics"]
        .as_array()
        .expect("semantic descriptors");
    assert_eq!(semantic.len(), 1);
    let descriptor = &semantic[0];
    assert_eq!(
        descriptor
            .as_object()
            .expect("descriptor object")
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        ["kind", "locale", "module", "name", "source_ids"]
            .into_iter()
            .collect()
    );
    assert_eq!(descriptor["locale"], "en");
    assert_eq!(descriptor["kind"], "variable");
    assert_eq!(descriptor["name"], "main.shared");
    let semantic_module = descriptor["module"].as_str().expect("semantic module");
    assert_eq!(semantic_module.matches('/').count(), 4);
    assert!(out.join(semantic_module).is_file());
    let semantic_map_path = out.join(format!("{semantic_module}.map"));
    assert!(semantic_map_path.is_file());

    let source_ids = manifest["sources"]
        .as_array()
        .expect("source table")
        .iter()
        .map(|source| source["id"].as_u64().expect("source id"))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(descriptor["source_ids"]
        .as_array()
        .expect("semantic source ids")
        .iter()
        .all(|id| source_ids.contains(&id.as_u64().expect("semantic source id"))));

    let actual_sources = manifest["sources"]
        .as_array()
        .expect("source table")
        .iter()
        .map(|source| {
            project
                .path()
                .join(source["path"].as_str().expect("source path"))
                .canonicalize()
                .expect("actual source")
        })
        .collect::<std::collections::BTreeSet<_>>();
    let semantic_map: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&semantic_map_path).expect("semantic map"))
            .expect("semantic map JSON");
    for source in semantic_map["sources"]
        .as_array()
        .expect("semantic map sources")
    {
        let source = source.as_str().expect("semantic map source");
        assert!(!source.starts_with('/') && !source.contains('\\'));
        let resolved = semantic_map_path
            .parent()
            .expect("semantic map parent")
            .join(source)
            .canonicalize()
            .expect("semantic map source path");
        assert!(actual_sources.contains(&resolved));
    }

    let semantic_code = fs::read_to_string(out.join(semantic_module)).expect("semantic code");
    assert!(
        semantic_code.contains("export const") && semantic_code.contains("Common"),
        "{semantic_code}"
    );
    for message in ["main.first", "main.second"] {
        let module = manifest["messages"][message]["locales"]["en"]["module"]
            .as_str()
            .expect("message module");
        let code = fs::read_to_string(out.join(module)).expect("message code");
        assert!(code.contains("from \"../../semantic/en/variable/"));
        assert!(!code.contains("const shared ="));
        let map_path = out.join(format!("{module}.map"));
        let map: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&map_path).expect("message map"))
                .expect("message map JSON");
        assert!(map["sources"]
            .as_array()
            .expect("message map sources")
            .iter()
            .all(|source| {
                let source = source.as_str().expect("map source");
                !source.starts_with('/')
                    && !source.contains('\\')
                    && map_path
                        .parent()
                        .expect("map parent")
                        .join(source)
                        .is_file()
            }));
    }

    let selected_config = fs::read_to_string(project.path().join("linguini.toml"))
        .expect("config")
        .replace(
            "messages = [\"main.first\", \"main.second\", \"main.plain\"]",
            "messages = [\"main.plain\"]",
        );
    fs::write(project.path().join("linguini.toml"), selected_config).expect("selected config");
    build_project(project.path()).expect("selected build");
    let selected_manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("selected manifest"))
            .expect("selected manifest JSON");
    assert!(selected_manifest["message_semantics"]
        .as_array()
        .expect("selected semantic descriptors")
        .is_empty());
    assert!(!out.join(semantic_module).exists());
    assert!(!out.join(format!("{semantic_module}.map")).exists());
}

#[test]
fn bundler_manifest_bridges_application_references_before_output_mutation() {
    let project = temp_project_dir("bundler-applications").expect("project");
    fs::create_dir_all(project.path().join("schema")).expect("schema dir");
    fs::create_dir_all(project.path().join("locales/main")).expect("locale dir");
    fs::create_dir_all(project.path().join("src/app")).expect("app dir");
    fs::write(
        project.path().join("linguini.toml"),
        r#"
[project]
name = "bundler-applications"
default_locale = "en"
locales = ["en"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "src/generated/linguini"
declaration = false
gitignore = false
framework = "svelte"
[targets.ts.bundler]
sources = ["src", "src/app"]
exclude = []
[targets.ts.bundler.dynamic]
mode = "bundle"
allow = ["main.title", "main.items"]
"#,
    )
    .expect("config");
    fs::write(
        project.path().join("schema/main.lgs"),
        "title\nitems(count: Number)\n",
    )
    .expect("schema");
    fs::write(
        project.path().join("locales/main/en.lgl"),
        "title = Title\nitems = {count @number} items\n",
    )
    .expect("locale");
    let app_source = concat!(
        "import { l as tr } from \"../generated/linguini\";\r\n",
        "import { l as dynamic } from \"../generated/linguini\";\r\n",
        "import { messages as dynamicCall } from \"../generated/linguini\";\r\n",
        "const label = \"Привет\" + tr.main.title;\r\n",
        "const count = tr.main.items(2);\r\n",
        "const property = tr.main.title.extra;\r\n",
        "const optional = tr?.main.title;\r\n",
        "const optionalCall = tr.main.title?.();\r\n",
        "const optionalCallWithTrivia = tr.main.title /* trivia */ ?. ();\r\n",
        "const optionalParameterizedCall = tr.main.items?.(2);\r\n",
        "const dynamicLabel = dynamic.main[key];\r\n",
        "const dynamicItems = dynamicCall.main[key](2);\r\n",
        "const unrelated = object.main.title;\r\n",
    );
    let app_path = project.path().join("src/app/page.svelte");
    fs::write(&app_path, app_source).expect("app");
    let clean_source = concat!(
        "import {\r\n  helper,\r\n  messages as clean\r\n} from \"../generated/linguini\";\r\n",
        "const cleanLabel = clean.main.title;\r\n",
    );
    fs::write(project.path().join("src/app/z-clean.ts"), clean_source).expect("clean app");
    let duplicate_source = concat!(
        "import { l as same } from \"generated-a\";\r\n",
        "import { messages as same } from \"generated-b\";\r\n",
        "const duplicateLabel = same.main.title;\r\n",
    );
    fs::write(
        project.path().join("src/app/z-duplicate.ts"),
        duplicate_source,
    )
    .expect("duplicate app");

    build_project(project.path()).expect("build");
    let manifest_path = project
        .path()
        .join("src/generated/linguini/bundler/manifest.json");
    let first_text = fs::read_to_string(&manifest_path).expect("manifest");
    let manifest: serde_json::Value = serde_json::from_str(&first_text).expect("JSON");
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["locale_loading"], "eager");
    let source_ids = manifest["sources"]
        .as_array()
        .expect("source table")
        .iter()
        .map(|source| {
            (
                source["path"].as_str().expect("source path"),
                source["id"].as_u64().expect("source id"),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        manifest["message_runtimes"],
        serde_json::json!({
            "en": {
                "module": "locales/en/_runtime.ts",
                "source_ids": [source_ids["schema/main.lgs"], source_ids["locales/main/en.lgl"]],
            }
        })
    );
    assert!(project
        .path()
        .join("src/generated/linguini/locales/en/_runtime.ts")
        .is_file());
    assert_eq!(
        manifest["runtime_helpers"],
        serde_json::json!({
            "svelte_locale": {
                "import": "./svelte-locale.svelte.js",
                "file": "svelte-locale.svelte.ts"
            }
        })
    );
    let application = &manifest["applications"]["src/app/page.svelte"];
    assert_eq!(application["byte_length"], app_source.len());
    assert_eq!(application["source_id"], 0x8000_0000_u32);
    let digest = application["sha256"].as_str().expect("digest");
    assert_eq!(digest.len(), 64);
    assert!(digest
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
    assert!(!first_text.contains("Привет"));
    let references = application["references"].as_array().expect("references");
    assert_eq!(references.len(), 2);
    assert_eq!(references[0]["message"], "main.title");
    assert_eq!(references[0]["kind"], "value");
    assert_eq!(references[0]["arity"], 0);
    assert_eq!(references[0]["local"], "tr");
    assert_eq!(references[0]["provenance"]["kind"], "imported");
    assert_eq!(
        references[0]["provenance"]["module_specifier"],
        "../generated/linguini"
    );
    assert_eq!(references[0]["provenance"]["symbol"], "l");
    assert!(references[0]["binding_id"].is_string());
    assert_eq!(references[0]["binding_id"], references[1]["binding_id"]);
    assert_eq!(references[1]["message"], "main.items");
    assert_eq!(references[1]["kind"], "call");
    assert_eq!(references[1]["arity"], 1);
    let dynamic_references = application["dynamic_references"]
        .as_array()
        .expect("dynamic references");
    assert_eq!(dynamic_references.len(), 2);
    assert_eq!(dynamic_references[0]["kind"], "computed");
    assert_eq!(dynamic_references[0]["prefix"], "main");
    assert_eq!(dynamic_references[0]["reference_kind"], "value");
    assert_eq!(
        dynamic_references[0]["messages"],
        serde_json::json!([
            {"message": "main.items", "key": "items", "arity": 1},
            {"message": "main.title", "key": "title", "arity": 0},
        ])
    );
    assert_eq!(dynamic_references[1]["kind"], "computed");
    assert_eq!(dynamic_references[1]["prefix"], "main");
    assert_eq!(dynamic_references[1]["reference_kind"], "call");
    assert_eq!(
        dynamic_references[1]["messages"],
        serde_json::json!([
            {"message": "main.items", "key": "items", "arity": 1},
            {"message": "main.title", "key": "title", "arity": 0},
        ])
    );
    for (entry, local, expression, receiver, key) in [
        (
            &dynamic_references[0],
            "dynamic",
            "dynamic.main[key]",
            "dynamic.main",
            "key",
        ),
        (
            &dynamic_references[1],
            "dynamicCall",
            "dynamicCall.main[key]",
            "dynamicCall.main",
            "key",
        ),
    ] {
        let span = &entry["span"];
        let start = span["start"].as_u64().expect("dynamic span start") as usize;
        let end = span["end"].as_u64().expect("dynamic span end") as usize;
        assert_eq!(&app_source[start..end], expression);
        let receiver_span = &entry["receiver_span"];
        let receiver_start = receiver_span["start"]
            .as_u64()
            .expect("dynamic receiver span start") as usize;
        let receiver_end = receiver_span["end"]
            .as_u64()
            .expect("dynamic receiver span end") as usize;
        assert_eq!(&app_source[receiver_start..receiver_end], receiver);
        let key_span = &entry["key_span"];
        let key_start = key_span["start"].as_u64().expect("dynamic key span start") as usize;
        let key_end = key_span["end"].as_u64().expect("dynamic key span end") as usize;
        assert_eq!(&app_source[key_start..key_end], key);
        assert_eq!(entry["local"], local);
        assert_eq!(entry["provenance"]["kind"], "imported");
        assert_eq!(
            entry["provenance"]["module_specifier"],
            "../generated/linguini"
        );
        assert_eq!(
            entry["provenance"]["symbol"],
            if local == "dynamic" { "l" } else { "messages" }
        );
    }
    let start = references[0]["start"].as_u64().expect("start") as usize;
    let end = references[0]["end"].as_u64().expect("end") as usize;
    assert_eq!(&app_source.as_bytes()[start..end], b"tr.main.title");
    let unresolved = application["unresolved"].as_array().expect("unresolved");
    assert!(unresolved.iter().any(|entry| {
        entry["message"] == "main.title.extra" && entry["reason"] == "non_exact_message_path"
    }));
    assert!(unresolved
        .iter()
        .any(|entry| entry["message"] == "main.title" && entry["reason"] == "optional_chain"));
    assert!(unresolved
        .iter()
        .any(|entry| entry["message"] == "main.items" && entry["reason"] == "optional_chain"));
    assert!(application["analysis_dynamic_prefixes"]
        .as_array()
        .expect("dynamic prefixes")
        .iter()
        .any(|prefix| prefix == "main"));
    let page_import = &application["imports"][0];
    assert_eq!(page_import["binding_id"], references[0]["binding_id"]);
    assert_eq!(page_import["analyzer_exact_uses_only"], false);
    assert_eq!(page_import["analyzer_tracked_uses_only"], true);
    assert_eq!(page_import["transformable"], false);
    let dynamic_import = application["imports"]
        .as_array()
        .expect("imports")
        .iter()
        .find(|binding| binding["local"] == "dynamic")
        .expect("dynamic import");
    assert_eq!(
        dynamic_references[0]["binding_id"],
        dynamic_import["binding_id"]
    );
    assert_eq!(dynamic_import["analyzer_exact_uses_only"], false);
    assert_eq!(dynamic_import["analyzer_tracked_uses_only"], true);
    assert_eq!(dynamic_import["transformable"], true);
    let dynamic_call_import = application["imports"]
        .as_array()
        .expect("imports")
        .iter()
        .find(|binding| binding["local"] == "dynamicCall")
        .expect("dynamic call import");
    assert_eq!(
        dynamic_references[1]["binding_id"],
        dynamic_call_import["binding_id"]
    );
    assert_eq!(dynamic_call_import["analyzer_tracked_uses_only"], true);
    assert_eq!(dynamic_call_import["transformable"], true);
    assert_eq!(
        &app_source[page_import["removal_start"].as_u64().expect("start") as usize
            ..page_import["removal_end"].as_u64().expect("end") as usize],
        "import { l as tr } from \"../generated/linguini\";"
    );
    assert!(unresolved
        .iter()
        .filter(|entry| !entry["binding_id"].is_null())
        .all(|entry| entry["binding_id"] == page_import["binding_id"]));

    let clean = &manifest["applications"]["src/app/z-clean.ts"];
    let clean_import = &clean["imports"][0];
    assert_eq!(clean_import["imported"], "messages");
    assert_eq!(clean_import["local"], "clean");
    assert_eq!(clean_import["analyzer_exact_uses_only"], true);
    assert_eq!(clean_import["transformable"], true);
    assert_eq!(
        clean["references"][0]["binding_id"],
        clean_import["binding_id"]
    );
    assert_eq!(
        &clean_source[clean_import["removal_start"].as_u64().expect("start") as usize
            ..clean_import["removal_end"].as_u64().expect("end") as usize],
        ",\r\n  messages as clean"
    );

    let duplicate = &manifest["applications"]["src/app/z-duplicate.ts"];
    assert!(duplicate["imports"]
        .as_array()
        .expect("duplicate imports")
        .iter()
        .all(|binding| binding["transformable"] == false));
    assert!(duplicate["references"][0]["binding_id"].is_null());

    build_project(project.path()).expect("repeat build");
    assert_eq!(
        fs::read_to_string(&manifest_path).expect("manifest"),
        first_text
    );
    let config_path = project.path().join("linguini.toml");
    let reordered_config = fs::read_to_string(&config_path).expect("config").replace(
        "sources = [\"src\", \"src/app\"]",
        "sources = [\"src/app\", \"src\"]",
    );
    fs::write(&config_path, reordered_config).expect("reordered config");
    build_project(project.path()).expect("reordered source build");
    assert_eq!(
        fs::read_to_string(&manifest_path).expect("reordered manifest"),
        first_text
    );
    fs::write(&app_path, app_source.replace("Привет", "Здравствуйте")).expect("stale app");
    build_project(project.path()).expect("changed app build");
    let changed_text = fs::read_to_string(&manifest_path).expect("changed manifest");
    let changed: serde_json::Value = serde_json::from_str(&changed_text).expect("changed JSON");
    assert_ne!(
        changed["applications"]["src/app/page.svelte"]["sha256"],
        application["sha256"]
    );

    fs::write(&app_path, "import { l } from \"x\";\nl.main.title();\n").expect("mismatch app");
    build_project(project.path()).expect("arity mismatch remains unresolved");
    let mismatch: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("mismatch manifest"))
            .expect("mismatch JSON");
    let mismatch_app = &mismatch["applications"]["src/app/page.svelte"];
    assert!(mismatch_app["references"]
        .as_array()
        .expect("refs")
        .is_empty());
    assert!(mismatch_app["unresolved"]
        .as_array()
        .expect("unresolved")
        .iter()
        .any(|entry| entry["reason"] == "arity_mismatch"));
    assert_eq!(mismatch_app["imports"][0]["transformable"], false);

    let base_config = fs::read_to_string(&config_path).expect("read config");
    for framework in ["svelte", "sveltekit"] {
        let web_config = base_config.replace(
            "framework = \"svelte\"",
            &format!("framework = \"{framework}\""),
        ) + "\n[web.routing]\nlocale_prefix = \"always\"\n";
        fs::write(&config_path, web_config).expect("web config");
        build_project(project.path()).expect("web build");
        let web_manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path).expect("web manifest"))
                .expect("web manifest JSON");
        assert_eq!(
            web_manifest["runtime_helpers"]["svelte_effects"],
            serde_json::json!({
                "import": "./svelte-effects.svelte.js",
                "file": "svelte-effects.svelte.ts"
            })
        );
        assert!(project
            .path()
            .join("src/generated/linguini/svelte-effects.svelte.ts")
            .exists());
    }

    let dynamic_config = fs::read_to_string(&config_path)
        .expect("read config")
        .replace("exclude = []", "exclude = []\nlocale_loading = \"dynamic\"");
    fs::write(&config_path, dynamic_config).expect("dynamic locale config");
    build_project(project.path()).expect("dynamic locale loading build");
    let dynamic_manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("dynamic manifest"))
            .expect("dynamic manifest JSON");
    assert_eq!(dynamic_manifest["version"], 1);
    assert_eq!(dynamic_manifest["locale_loading"], "dynamic");
}

#[test]
fn bundler_dynamic_error_rejects_before_output_mutation() {
    let project = temp_project_dir("bundler-dynamic-strict").expect("project");
    fs::create_dir_all(project.path().join("schema")).expect("schema dir");
    fs::create_dir_all(project.path().join("locales/main")).expect("locale dir");
    fs::create_dir_all(project.path().join("src")).expect("src dir");
    fs::create_dir_all(project.path().join("build/generated")).expect("output dir");
    fs::write(
        project.path().join("linguini.toml"),
        r#"
[project]
name = "bundler-dynamic-strict"
default_locale = "en"
locales = ["en"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "build/generated"
declaration = false
gitignore = false
framework = "svelte"
[targets.ts.bundler]
sources = ["src"]
"#,
    )
    .expect("config");
    fs::write(project.path().join("schema/main.lgs"), "title\n").expect("schema");
    fs::write(
        project.path().join("locales/main/en.lgl"),
        "title = Title\n",
    )
    .expect("locale");
    fs::write(
        project.path().join("src/page.ts"),
        "import { l } from \"generated\";\nconst label = l.main[key];\n",
    )
    .expect("application");
    let sentinel = project.path().join("build/generated/sentinel.ts");
    fs::write(&sentinel, "preserve me\n").expect("sentinel");

    let error = build_project(project.path()).expect_err("strict dynamic access must fail");
    let message = error.to_string();
    assert!(message.contains("src/page.ts:"), "{message}");
    assert!(message.contains("dynamic message access"), "{message}");
    assert!(
        message.contains("[targets.ts.bundler.dynamic]"),
        "{message}"
    );
    assert_eq!(
        fs::read_to_string(&sentinel).expect("sentinel").as_str(),
        "preserve me\n"
    );
}

fn dynamic_bundler_project(name: &str, policy: &str, source: &str) -> TempDir {
    let project = temp_project_dir(name).expect("project");
    fs::create_dir_all(project.path().join("schema")).expect("schema dir");
    fs::create_dir_all(project.path().join("locales/main")).expect("locale dir");
    fs::create_dir_all(project.path().join("src")).expect("src dir");
    fs::write(
        project.path().join("linguini.toml"),
        format!(
            r#"
[project]
name = "{name}"
default_locale = "en"
locales = ["en"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "build/generated"
declaration = false
gitignore = false
framework = "svelte"
[targets.ts.bundler]
sources = ["src"]
{policy}
"#
        ),
    )
    .expect("config");
    fs::write(
        project.path().join("schema/main.lgs"),
        "title\nitems(count: Number)\n",
    )
    .expect("schema");
    fs::write(
        project.path().join("locales/main/en.lgl"),
        "title = Title\nitems = {count} items\n",
    )
    .expect("locale");
    fs::write(project.path().join("src/page.ts"), source).expect("application");
    project
}

#[test]
fn bundler_dynamic_bundle_rejects_unknown_allow_entries() {
    let project = dynamic_bundler_project(
        "bundler-dynamic-unknown",
        "[targets.ts.bundler.dynamic]\nmode = \"bundle\"\nallow = [\"main.unknown\"]",
        "import { l } from \"generated\";\nconst label = l.main[key];\n",
    );
    let error = build_project(project.path()).expect_err("unknown allow must fail");
    assert!(error.to_string().contains("unknown compiled message leaf"));
    assert!(!project.path().join("build/generated").exists());
}

#[test]
fn bundler_dynamic_bundle_rejects_ambiguous_and_unsafe_shapes() {
    let cases = [
        (
            "factory",
            "import { createLinguini as make } from \"runtime\";\nexport const label = make(\"en\");\n",
            "kind `factory`",
        ),
        (
            "bare",
            "import { l } from \"generated\";\nconst label = l;\n",
            "kind `bare`",
        ),
        (
            "implicit",
            "const label = l.main[key];\n",
            "provenance `implicit`",
        ),
        (
            "multiple",
            "import { l } from \"generated\";\nconst label = l.main[first][second];\n",
            "kind `multiple_computed`",
        ),
        (
            "uncertain",
            "import { l } from \"generated\";\nconst label = l.main[key].title;\n",
            "kind `uncertain`",
        ),
        (
            "duplicate",
            "import { l as same } from \"generated-a\";\nimport { messages as same } from \"generated-b\";\nconst label = same.main[key];\n",
            "no unique removable import binding",
        ),
        (
            "untracked",
            "import { l } from \"generated\";\nconsume(l);\nconst label = l.main[key];\n",
            "kind `bare`",
        ),
    ];
    for (name, source, reason) in cases {
        let project = dynamic_bundler_project(
            &format!("bundler-dynamic-{name}"),
            "[targets.ts.bundler.dynamic]\nmode = \"bundle\"\nallow = [\"main.title\"]",
            source,
        );
        let error = match build_project(project.path()) {
            Ok(output) => panic!("{name}: unsafe dynamic shape must fail: {output}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains(reason), "{name}: {error}");
        assert!(!project.path().join("build/generated").exists(), "{name}");
    }
}

#[test]
fn generate_renders_locale_enum_and_plural_matrix() {
    let project = temp_project_dir("generate_renders_locale_enum_and_plural_matrix")
        .expect("create temporary project");
    init_project(project.path()).expect("init project");

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

    let output = generate_project_data(project.path()).expect("generated data");
    let plain = strip_ansi(&output);

    assert!(!output.contains("\x1b["));
    assert!(!output.contains("\"locales\""));
    assert!(plain.contains("locale en"));
    assert!(plain.contains("message shop.counted"));
    assert!(plain.contains("fruit=apple"));
    assert!(plain.contains("fruit=pear"));
    assert!(plain.contains("count=5"));
    assert!(plain.contains("=> 1 apple"));
    assert!(plain.contains("=> 5 apples"));
}

fn strip_ansi(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if code == 'm' {
                    break;
                }
            }
            continue;
        }
        output.push(character);
    }
    output
}
