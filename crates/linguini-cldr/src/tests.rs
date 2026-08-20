use super::parse_plural_rule;

#[test]
fn crate_exports_plural_parser() {
    assert!(parse_plural_rule("i = 1 and v = 0").is_ok());
}
