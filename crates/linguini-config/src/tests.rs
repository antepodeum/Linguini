use super::{parse_config, DEFAULT_CONFIG_FILE};

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
cache = ".linguini/cache"
"#,
    )
    .expect("valid config parses");

    assert_eq!(DEFAULT_CONFIG_FILE, "linguini.toml");
    assert_eq!(config.project.name, "shop");
}
