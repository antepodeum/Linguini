use super::CRATE_PURPOSE;

#[test]
fn crate_has_unit_test_structure() {
    assert_eq!(CRATE_PURPOSE, "package import and registry support");
}
