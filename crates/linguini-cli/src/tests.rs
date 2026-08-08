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
    assert!(generated_shop.contains("    delivery: (count: number | bigint | string) =>"));
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
        "enum Color {\n  red\n}\nfirst(color: Color)\n",
    )
    .expect("schema a");
    fs::write(
        project.path().join("schema/b.lgs"),
        format!("{long_message}\n"),
    )
    .expect("schema b");
    fs::write(project.path().join("locales/a/en.lgl"), "first = First\n").expect("locale a en");
    fs::write(project.path().join("locales/a/fr.lgl"), "first = Premier\n").expect("locale a fr");
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
