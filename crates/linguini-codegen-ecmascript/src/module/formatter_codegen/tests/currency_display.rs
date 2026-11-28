use super::*;

fn corpus() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-display.json"
    )))
    .unwrap()
}

fn selected_request(case: &serde_json::Value) -> FormatterRequest {
    request(
        IrFormatterKind::Currency,
        &[
            ("code", case["code"].as_str().unwrap()),
            (
                "accounting",
                if case["accounting"].as_bool().unwrap() {
                    "true"
                } else {
                    "false"
                },
            ),
            ("display", "code"),
        ],
    )
}

#[test]
fn currency_code_display_matches_the_exact_icu_corpus_without_runtime_policy() {
    let corpus = corpus();
    let cases = corpus["cases"].as_array().unwrap();
    let locales = cases
        .iter()
        .map(|case| case["locale"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    for locale in locales {
        let selected = cases
            .iter()
            .filter(|case| case["locale"] == locale)
            .collect::<Vec<_>>();
        let requests = selected
            .iter()
            .map(|case| selected_request(case))
            .collect::<Vec<_>>();
        let runtime = formatter_data_declaration(locale, &requests, false);
        for absent in [
            "coerceDate",
            "scaleBy",
            "scientificGeneratedDecimal",
            "CurrencySymbol",
            "CurrencyCode",
            "Intl.NumberFormat",
            "¤",
        ] {
            assert!(!runtime.contains(absent), "{locale}: {absent}");
        }
        let mut script=format!("{runtime}\nimport assert from 'node:assert/strict';\nIntl.NumberFormat=class{{constructor(){{throw Error('no host formatter');}}}};\n");
        for case in selected {
            let name = formatter_binding(&selected_request(case));
            script.push_str(&format!(r#"
for(const {{input,expected}} of {samples}) {{
  assert.equal({name}(input),expected,input);
  if(String(Number(input))===input || input==='-0') assert.equal({name}(Number(input)),expected,input);
  if(/^-?\d+$/.test(input) && input!=='-0') assert.equal({name}(BigInt(input)),expected,input);
  let reads=0;
  assert.equal({name}({{[Symbol.toPrimitive](hint){{assert.equal(hint,'string');reads++;return input;}}}}),expected,input);
  assert.equal(reads,1);
}}
const {name}Failure=Error('coercion failed');
assert.throws(()=>{name}({{toString(){{throw {name}Failure;}}}}),error=>error==={name}Failure);
assert.throws(()=>{name}('invalid'),RangeError);
for(const input of [NaN,Infinity,-Infinity]) assert.equal({name}(input),String(input));
"#,samples=case["samples"]));
        }
        script.push_str("console.log('ok');\n");
        assert_eq!(execute(&script), "ok", "{locale}");
    }
}

#[test]
fn currency_display_fixture_shares_precision_and_digit_helpers() {
    let corpus = corpus();
    let mut bindings = DigitBindings::default();
    let mut plans = Vec::new();
    let mut source = crate::templates::render(
        "formatters/types",
        minijinja::context! { needs_number => true },
    );
    for (locale, code, accounting) in [
        ("en", "USD", false),
        ("en", "USD", true),
        ("ar-EG", "USD", false),
        ("fr-CH", "JPY", false),
        ("hi", "KWD", false),
        ("ff-Adlm", "ZZZ", false),
    ] {
        assert!(corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["locale"] == locale
                && case["code"] == code
                && case["accounting"] == accounting));
        let compiled = linguini_codegen::NumberFormatPlan::currency_with_display(
            locale, code, accounting, "code",
        )
        .unwrap();
        let plan = number_plan_context(&compiled, &mut bindings);
        let name = format!(
            "currency_{}_{}_{}",
            locale.replace('-', "_"),
            code,
            if accounting { "accounting" } else { "standard" }
        );
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! { name,plan => plan.clone(),exported => true },
        ));
        plans.push(plan);
    }
    source.push_str(&number_helpers(&plans));
    for name in [
        "parseGeneratedDecimal",
        "roundTo0FractionDigits",
        "roundTo2FractionDigits",
        "roundTo3FractionDigits",
    ] {
        assert_eq!(
            source.matches(&format!("function {name}(")).count(),
            1,
            "{name}"
        );
    }
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-runtime/currency-display.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &source).unwrap();
    }
    assert_eq!(source, std::fs::read_to_string(snapshot).unwrap());
}
