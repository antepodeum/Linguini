use super::{cache_root, parse_plural_rule, CRATE_PURPOSE};
use std::path::Path;

#[test]
fn crate_exports_core_cldr_entry_points() {
    assert_eq!(CRATE_PURPOSE, "CLDR ingestion and plural rules");
    assert!(cache_root(Path::new("/tmp/shop"), ".linguini/cldr").ends_with(".linguini/cldr"));
    assert!(parse_plural_rule("i = 1 and v = 0").is_ok());
}
