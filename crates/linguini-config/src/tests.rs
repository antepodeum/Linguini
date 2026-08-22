use super::{parse_config, CONFIG_SCHEMA_JSON, DEFAULT_CONFIG_FILE};

#[test]
fn crate_exports_config_parser_and_default_filename() {
    let config = parse_config(
        r#"
[project]
name = "shop"
default_locale = "ru"
locales = ["ru"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"
"#,
    )
    .expect("valid config parses");

    assert_eq!(DEFAULT_CONFIG_FILE, "linguini.toml");
    assert_eq!(config.project.name, "shop");
}

#[test]
fn committed_config_schema_snapshot_matches_public_tables_and_defaults() {
    let schema: serde_json::Value = serde_json::from_str(CONFIG_SCHEMA_JSON).expect("valid schema");
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], serde_json::json!(["project", "paths"]));
    assert_eq!(
        schema["properties"]["targets"]["properties"]["ts"]["properties"]["out"]["default"],
        "src/generated/linguini"
    );
    assert_eq!(
        schema["properties"]["web"]["properties"]["routing"]["properties"]["locale_prefix"]["enum"],
        serde_json::json!(["always", "except-default", "never"])
    );
    assert!(CONFIG_SCHEMA_JSON.ends_with('\n'));
}
