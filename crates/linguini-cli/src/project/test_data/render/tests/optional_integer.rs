use super::*;

mod cases {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/optional-integer.rs"
    ));
}

#[test]
fn numeric_previews_omit_optional_zero_after_rounding_and_keep_required_digits() {
    for case in cases::CASES {
        let (numbers, pattern) = case.formatting();
        for (input, expected) in case.samples {
            let actual = format_number_pattern(input, &pattern, &numbers, None, None);
            assert_eq!(
                actual.unwrap(),
                *expected,
                "{} ({}): {input}",
                case.name,
                case.pattern
            );
        }
    }
}
