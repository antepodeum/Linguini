use super::*;

#[test]
fn wide_numeric_quanta_match_independent_exact_half_up_results() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/wide-number-quantum.json"
    )))
    .unwrap();
    let numbers = compiled_number_formatting("en").unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.primary_group_size = None;
        pattern.positive.secondary_group_size = None;
        pattern.positive.min_fraction_digits = case["fraction_digits"].as_u64().unwrap() as u8;
        pattern.positive.max_fraction_digits = pattern.positive.min_fraction_digits;
        let quantum = case["quantum"].as_str().unwrap();
        let plan = NumberFormatPlan::new(&pattern, None, &numbers, Some(quantum)).unwrap();
        for sample in case["samples"].as_array().unwrap() {
            let input = sample["input"].as_str().unwrap();
            assert_eq!(
                format_number_plan(input, &plan, numbers.locale).unwrap(),
                sample["expected"].as_str().unwrap(),
                "quantum={quantum}, fraction={}: {input}",
                case["fraction_digits"]
            );
        }
    }
}

#[test]
fn significant_precision_ignores_wide_fraction_quanta() {
    let numbers = compiled_number_formatting("en").unwrap();
    let mut pattern = numbers.decimal_pattern;
    pattern.positive.primary_group_size = None;
    pattern.positive.secondary_group_size = None;
    pattern.positive.min_significant_digits = Some(3);
    pattern.positive.max_significant_digits = Some(3);
    for (exponent, samples) in [
        (
            None,
            [
                ("1234.56", "1230"),
                ("0.00123456", "0.00123"),
                ("-0", "-0.00"),
            ],
        ),
        (
            Some(1),
            [
                ("1234.56", "1.23E3"),
                ("0.00123456", "1.23E-3"),
                ("-0", "-0.00E0"),
            ],
        ),
    ] {
        pattern.positive.exponent_digits = exponent;
        let plan =
            NumberFormatPlan::new(&pattern, None, &numbers, Some("18446744073709551617")).unwrap();
        for (input, expected) in samples {
            assert_eq!(
                format_number_plan(input, &plan, numbers.locale).unwrap(),
                expected
            );
        }
    }
}
