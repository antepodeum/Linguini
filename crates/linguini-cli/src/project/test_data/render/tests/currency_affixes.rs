use super::*;

mod cases {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-affixes.rs"
    ));
}

#[test]
fn numeric_previews_replace_only_compiled_currency_slots() {
    let numbers = compiled_number_formatting("en").unwrap();
    for case in cases::CASES {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.prefix = case.prefix;
        pattern.positive.suffix = case.suffix;
        pattern.positive.min_fraction_digits = 0;
        pattern.positive.max_fraction_digits = 0;
        pattern.positive.primary_group_size = None;
        pattern.positive.secondary_group_size = None;
        pattern.negative = case
            .negative
            .map(|(prefix, suffix)| linguini_cldr::NumberPatternPart {
                prefix,
                suffix,
                ..pattern.positive
            });
        let plan =
            NumberFormatPlan::new_with_currency_code(&pattern, Some("$"), "usd", &numbers, None)
                .unwrap();
        for (input, expected) in [("1", case.positive), ("-1", case.negative_text)] {
            assert_eq!(
                format_number_plan(input, &plan, numbers.locale).unwrap(),
                expected,
                "{}: {}",
                case.name,
                case.pattern
            );
        }
    }
}

#[test]
fn currency_previews_use_the_complete_pinned_letter_boundary_pattern() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-boundaries.json"
    )))
    .unwrap();
    let schema = linguini_ir::IrModule::default();
    for case in cases.as_array().unwrap() {
        let renderer = Renderer::new(&schema, &schema, case["locale"].as_str().unwrap());
        let arguments = [
            IrFormatterArgument {
                name: "code".into(),
                value: case["code"].as_str().unwrap().to_owned(),
            },
            IrFormatterArgument {
                name: "accounting".into(),
                value: case["accounting"].to_string(),
            },
        ];
        let input = case["input"].as_str().unwrap();
        assert_eq!(
            renderer.format_currency(input, &arguments).unwrap(),
            case["positive"].as_str().unwrap(),
            "{case}"
        );
        assert_eq!(
            renderer
                .format_currency(&format!("-{input}"), &arguments)
                .unwrap(),
            case["negative"].as_str().unwrap(),
            "{case}"
        );
    }
}
