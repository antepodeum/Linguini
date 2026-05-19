use super::{parse_plural_rule, CRATE_PURPOSE};

#[test]
fn crate_exports_core_cldr_entry_points() {
    assert_eq!(CRATE_PURPOSE, "compiled CLDR plural and formatting data");
    assert!(parse_plural_rule("i = 1 and v = 0").is_ok());
}
