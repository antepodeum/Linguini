use super::*;
use linguini_cldr::{
    compiled_currency_formatting, compiled_currency_symbol, compiled_number_formatting,
    NumberPaddingPosition,
};

mod currency_affix_cases {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-affixes.rs"
    ));
}

#[test]
fn currency_substitution_preserves_literal_affix_fragments() {
    let numbers = compiled_number_formatting("en").unwrap();
    for case in currency_affix_cases::CASES {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.prefix = case.prefix;
        pattern.positive.suffix = case.suffix;
        pattern.negative = case
            .negative
            .map(|(prefix, suffix)| linguini_cldr::NumberPatternPart {
                prefix,
                suffix,
                ..pattern.positive
            });
        let plan = NumberFormatPlan::new(&pattern, Some("$"), &numbers, None).unwrap();
        for (prefix, suffix, expected) in [
            (&plan.affixes.prefix, &plan.affixes.suffix, case.positive),
            (
                &plan.affixes.negative_prefix,
                &plan.affixes.negative_suffix,
                case.negative_text,
            ),
        ] {
            assert_eq!(
                format!("{prefix}1{suffix}"),
                expected,
                "{}: {}",
                case.name,
                case.pattern
            );
        }
    }
}

#[test]
fn selected_pattern_compiles_affixes_grouping_quantum_scientific_symbols_and_padding() {
    let mut numbers = compiled_number_formatting("fa").unwrap();
    numbers.minimum_grouping_digits = 2;
    let mut pattern = numbers.decimal_pattern;
    pattern.negative = None;
    pattern.positive.prefix = NumberAffix::CurrencySymbol;
    pattern.positive.suffix = NumberAffix::Literal("%");
    pattern.positive.secondary_group_size = Some(2);
    pattern.positive.scale = 100;
    pattern.positive.min_significant_digits = Some(2);
    pattern.positive.max_significant_digits = Some(4);
    pattern.positive.decimal_separator_always = true;
    pattern.positive.rounding_increment = Some("5");
    pattern.positive.exponent_digits = Some(2);
    pattern.positive.exponent_sign_always = true;
    pattern.positive.padding = Some(NumberPadding {
        character: 'x',
        width: 12,
        position: NumberPaddingPosition::BeforeSuffix,
    });
    let plan = NumberFormatPlan::new(&pattern, Some("€"), &numbers, Some("00025")).unwrap();
    assert_eq!(
        plan.affixes,
        NumberAffixPlan {
            prefix: "€".into(),
            suffix: "%".into(),
            negative_prefix: "\u{200e}−€".into(),
            negative_suffix: "%".into()
        }
    );
    assert_eq!(
        plan.grouping,
        Some(NumberGroupingPlan {
            primary_size: 3,
            secondary_size: 2,
            threshold: 4
        })
    );
    assert_eq!((plan.scale, plan.scale_shift), (100, 2));
    assert_eq!(plan.rounding, NumberRoundingPlan::new(3, "25").unwrap());
    assert_eq!(
        plan.scientific,
        Some(NumberScientificPlan {
            exponent_digits: 2,
            separator: "×۱۰^",
            positive_sign: "\u{200e}+",
            negative_sign: "\u{200e}−",
            carry_fraction: Some("000".into())
        })
    );
    assert_eq!(plan.padding, pattern.positive.padding);
    assert_eq!(plan.digits, "۰۱۲۳۴۵۶۷۸۹");
    assert_eq!(plan.decimal_symbol, "٫");
    assert_eq!(plan.group_symbol, "٬");
    assert!(plan.decimal_separator_always);
    let mut negative = pattern.positive;
    negative.decimal_separator_always = false;
    negative.prefix = NumberAffix::Parts(&[
        NumberAffixPart::Literal("("),
        NumberAffixPart::CurrencySymbol,
    ]);
    negative.suffix = NumberAffix::Literal(")");
    pattern.negative = Some(negative);
    let plan = NumberFormatPlan::new(&pattern, Some("€"), &numbers, None).unwrap();
    assert_eq!(plan.affixes.negative_prefix, "(€");
    assert_eq!(plan.affixes.negative_suffix, ")");
    assert_eq!(plan.rounding.increment, "5");
    assert!(
        plan.decimal_separator_always,
        "the positive pattern owns numeric choices"
    );
}

#[test]
fn plan_boundaries_preserve_optional_demand_and_avoid_narrowing_thresholds() {
    let mut numbers = compiled_number_formatting("en").unwrap();
    let mut pattern = numbers.decimal_pattern;
    for (scale, shift) in [(1, 0), (100, 2), (1000, 3)] {
        pattern.positive.scale = scale;
        assert_eq!(
            NumberFormatPlan::new(&pattern, None, &numbers, None)
                .unwrap()
                .scale_shift,
            shift
        );
    }
    for scale in [0, 2, u16::MAX] {
        pattern.positive.scale = scale;
        assert!(NumberFormatPlan::new(&pattern, None, &numbers, None).is_none());
    }
    pattern.positive.scale = 1;
    for quantum in ["", "-1", "1.5", "x"] {
        assert!(NumberFormatPlan::new(&pattern, None, &numbers, Some(quantum)).is_none());
    }
    pattern.positive.primary_group_size = Some(u8::MAX);
    pattern.positive.secondary_group_size = None;
    numbers.minimum_grouping_digits = u8::MAX;
    assert_eq!(
        NumberFormatPlan::new(&pattern, None, &numbers, Some("0"))
            .unwrap()
            .grouping,
        Some(NumberGroupingPlan {
            primary_size: u8::MAX,
            secondary_size: u8::MAX,
            threshold: 509
        })
    );
    pattern.positive.primary_group_size = Some(0);
    assert!(NumberFormatPlan::new(&pattern, None, &numbers, None).is_none());
    pattern.positive.primary_group_size = Some(3);
    pattern.positive.secondary_group_size = Some(0);
    assert!(NumberFormatPlan::new(&pattern, None, &numbers, None).is_none());
    pattern.positive.secondary_group_size = None;
    numbers.minimum_grouping_digits = 0;
    assert!(NumberFormatPlan::new(&pattern, None, &numbers, None).is_none());
    pattern.positive.primary_group_size = None;
    let plan = NumberFormatPlan::new(&pattern, None, &numbers, None).unwrap();
    assert!(plan.grouping.is_none());
    assert!(plan.scientific.is_none());
    assert!(plan.padding.is_none());
    assert!(!plan.decimal_separator_always);
}

#[test]
fn all_reachable_pinned_number_percent_and_currency_patterns_have_shared_plans() {
    let mut seen = std::collections::BTreeSet::new();
    for locale in linguini_cldr::locale_resolution_candidates() {
        let Some(numbers) = compiled_number_formatting(locale) else {
            continue;
        };
        if !seen.insert(numbers.locale) {
            continue;
        }
        for pattern in [numbers.decimal_pattern, numbers.percent_pattern] {
            assert!(
                NumberFormatPlan::new(&pattern, None, &numbers, None).is_some(),
                "{locale}"
            );
        }
        for code in ["USD", "CHF", "JPY"] {
            for accounting in [false, true] {
                assert!(
                    NumberFormatPlan::currency(locale, code, accounting).is_some(),
                    "{locale}/{code}/{accounting}"
                );
            }
        }
        let currency = compiled_currency_formatting(locale).unwrap();
        let symbols = NumberFormatData {
            decimal_symbol: currency.decimal_symbol,
            group_symbol: currency.group_symbol,
            ..numbers
        };
        let symbol = compiled_currency_symbol(locale, "USD").unwrap_or("USD");
        for pattern in [
            Some(currency.standard_pattern),
            currency.accounting_pattern,
            currency.standard_alpha_next_to_number.copied(),
            currency.accounting_alpha_next_to_number.copied(),
        ]
        .into_iter()
        .flatten()
        {
            assert!(
                NumberFormatPlan::new(&pattern, Some(symbol), &symbols, Some("0")).is_some(),
                "{locale}"
            );
        }
    }
    assert!(
        seen.len() > 500,
        "pinned resolution coverage unexpectedly shrank: {}",
        seen.len()
    );
}
