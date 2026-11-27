use super::*;

mod cases {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-affixes.rs"
    ));
}

#[test]
fn generated_currency_substitution_preserves_literal_marks() {
    let numbers = compiled_number_formatting("en").unwrap();
    let mut bindings = DigitBindings::default();
    let mut plans = Vec::new();
    let mut source = crate::templates::render(
        "formatters/types",
        minijinja::context! { needs_number => true },
    );
    let mut checks = String::from("import assert from 'node:assert/strict';\n");
    for case in cases::CASES {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.prefix = case.prefix;
        pattern.positive.suffix = case.suffix;
        pattern.positive.min_fraction_digits = 0;
        pattern.positive.max_fraction_digits = 0;
        pattern.positive.primary_group_size = None;
        pattern.positive.secondary_group_size = None;
        pattern.negative = case.negative.map(|(prefix, suffix)| NumberPatternPart {
            prefix,
            suffix,
            ..pattern.positive
        });
        let name = format!("affix_{}", case.name);
        let plan = number_plan(&pattern, Some("$"), &numbers, None, &mut bindings);
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! { name, plan => plan.clone(), exported => false },
        ));
        plans.push(plan);
        for (input, expected) in [("1", case.positive), ("-1", case.negative_text)] {
            checks.push_str(&format!(
                "assert.equal({name}({input:?}), {}, {});\n",
                serde_json::to_string(expected).unwrap(),
                serde_json::to_string(case.pattern).unwrap()
            ));
        }
        checks.push_str(&format!(
            "assert.equal({name}(1), {name}('1'));\nassert.equal({name}(1n), {name}('1'));\n"
        ));
    }
    source.push_str(&number_helpers(&plans));
    assert!(!source.contains("NumberAffix"));
    assert!(!source.contains("CurrencySymbol"));
    source.push_str(&checks);
    source.push_str("console.log('ok');\n");
    assert_eq!(execute(&source), "ok");
}

#[test]
fn currency_separators_are_selected_without_changing_decimal_output() {
    for (locale, number, standard, accounting) in [
        (
            "de-AT",
            "1\u{a0}234,5",
            "$\u{a0}1.234,50",
            "-$\u{a0}1.234,50",
        ),
        (
            "fr-CH",
            "1'234,5",
            "1'234.50\u{a0}$US",
            "(1'234.50\u{a0}$US)",
        ),
        ("en", "1,234.5", "$1,234.50", "($1,234.50)"),
    ] {
        let runtime = formatter_data_declaration(
            locale,
            &[
                request(IrFormatterKind::Number, &[]),
                request(IrFormatterKind::Currency, &[]),
                request(IrFormatterKind::Currency, &[("accounting", "true")]),
            ],
            false,
        );
        let script = format!(
            r#"{runtime}
import assert from 'node:assert/strict';
Intl.NumberFormat=class {{constructor(){{throw Error('no host formatter');}}}};
for(const value of [1234.5,'1234.5']) {{
  assert.equal(formatNumber(value),{number:?});
  assert.equal(formatCurrency(value),{standard:?});
  assert.equal(formatCurrency_accounting_true('-1234.5'),{accounting:?});
}}
console.log('ok');
"#
        );
        assert_eq!(execute(&script), "ok", "{locale}");
        assert!(!runtime.contains("currencyDecimal"), "{locale}");
        assert!(!runtime.contains("currencyGroup"), "{locale}");
    }
}

#[test]
fn currency_letter_boundaries_select_the_pinned_alternate_pattern() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-boundaries.json"
    )))
    .unwrap();
    for case in cases.as_array().unwrap() {
        let code = case["code"].as_str().unwrap();
        let accounting = if case["accounting"].as_bool().unwrap() {
            "true"
        } else {
            "false"
        };
        let request = request(
            IrFormatterKind::Currency,
            &[("code", code), ("accounting", accounting)],
        );
        let name = formatter_binding(&request);
        let runtime =
            formatter_data_declaration(case["locale"].as_str().unwrap(), &[request], false);
        for forbidden in [
            "alphaNextToNumber",
            "currencySpacing",
            "GeneralCategory",
            "NumberAffix",
        ] {
            assert!(!runtime.contains(forbidden), "{forbidden}: {case}");
        }
        let script = format!(
            r#"{runtime}
import assert from 'node:assert/strict';
Intl.NumberFormat = class {{ constructor() {{ throw Error('host formatter'); }} }};
const value = {input};
assert.equal({name}(value), {positive});
assert.equal({name}(Number(value)), {positive});
assert.equal({name}('-' + value), {negative});
assert.equal({name}(-Number(value)), {negative});
assert.equal({name}(1234567n), {name}('1234567'));
console.log('ok');
"#,
            input = case["input"],
            positive = case["positive"],
            negative = case["negative"]
        );
        assert_eq!(execute(&script), "ok", "{case}");
    }
}
