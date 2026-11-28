use super::*;

#[test]
fn decimal_visibility_specializes_fixed_significant_and_scientific_output() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    let corpus: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixtures.join("codegen/decimal-separator.json")).unwrap(),
    )
    .unwrap();
    let numbers = compiled_number_formatting("en").unwrap();
    let mut bindings = DigitBindings::default();
    let mut plans = Vec::new();
    let mut source = crate::templates::render(
        "formatters/types",
        minijinja::context! { needs_number => true },
    );
    let mut checks = String::from("import assert from 'node:assert/strict';\n");
    for case in corpus.as_array().unwrap() {
        for visible in [false, true] {
            let mut pattern = numbers.decimal_pattern;
            pattern.negative = None;
            let part = &mut pattern.positive;
            part.min_fraction_digits = case["min_fraction"].as_u64().unwrap() as u8;
            part.max_fraction_digits = case["max_fraction"].as_u64().unwrap() as u8;
            part.min_significant_digits = case["min_significant"].as_u64().map(|n| n as u8);
            part.max_significant_digits = case["max_significant"].as_u64().map(|n| n as u8);
            part.decimal_separator_always = visible;
            if case["scientific"].as_bool().unwrap() {
                part.max_integer_digits = Some(1);
                part.exponent_digits = Some(2);
                part.primary_group_size = None;
                part.secondary_group_size = None;
            }
            let name = format!(
                "{}_{}",
                case["name"].as_str().unwrap(),
                if visible { "visible" } else { "ordinary" }
            );
            let plan = number_plan(&pattern, None, &numbers, None, &mut bindings);
            source.push_str(&crate::templates::render(
                "formatters/number",
                minijinja::context! { name, plan => plan.clone(), exported => true },
            ));
            plans.push(plan);
            for sample in case["samples"].as_array().unwrap() {
                let expected = &sample[if visible { "visible" } else { "ordinary" }];
                checks.push_str(&format!(
                    "assert.equal({name}({}), {});\n",
                    sample["input"], expected
                ));
            }
            checks.push_str(&format!(r#"
assert.equal({name}(1), {name}('1'));
assert.equal({name}(1n), {name}('1'));
assert.equal({name}(-0), {name}('-0'));
assert.equal({name}(NaN), 'NaN');
assert.equal({name}(Infinity), 'Infinity');
{{
  let calls = 0;
  assert.equal({name}({{[Symbol.toPrimitive](hint) {{assert.equal(hint, 'string'); calls++; return '1';}}}}), {name}('1'));
  assert.equal(calls, 1);
  const error = new Error('input failed');
  assert.throws(() => {name}({{toString() {{throw error;}}}}), thrown => thrown === error);
}}
"#));
        }
    }
    source.push_str(&number_helpers(&plans));
    assert!(!source.contains("decimal_separator_always"));
    let snapshot = fixtures.join("golden/snapshots/js-runtime/decimal-separator.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &source).unwrap();
    }
    checks.push_str("console.log('ok');\n");
    assert_eq!(execute(&format!("{source}{checks}")), "ok");
    assert_eq!(std::fs::read_to_string(snapshot).unwrap(), source);
}

#[test]
fn decimal_visibility_preserves_unicode_affixes_and_padding_width() {
    let mut bindings = DigitBindings::default();
    let mut plans = Vec::new();
    let mut source = String::from("import assert from 'node:assert/strict';\n");
    let mut checks = String::new();
    for (locale, input, expected) in [
        ("ar-EG", "-0", "\u{061c}-v12:٠٫:v34"),
        ("fa", "1234", "v12:۱٬۲۳۴٫:v34"),
        ("ff-Adlm", "1", "v12:𞥑.:v34"),
    ] {
        let numbers = compiled_number_formatting(locale).unwrap();
        let mut pattern = numbers.decimal_pattern;
        pattern.negative = None;
        pattern.positive.prefix = NumberAffix::Literal("v12:");
        pattern.positive.suffix = NumberAffix::Literal(":v34");
        pattern.positive.min_fraction_digits = 0;
        pattern.positive.max_fraction_digits = 0;
        pattern.positive.decimal_separator_always = true;
        let name = format!("unicode{}", plans.len());
        let plan = number_plan(&pattern, None, &numbers, None, &mut bindings);
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! { name, plan => plan.clone(), exported => false },
        ));
        plans.push(plan);
        checks.push_str(&format!(
            "assert.equal({name}({input:?}), {});\n",
            serde_json::to_string(expected).unwrap()
        ));
    }
    let numbers = compiled_number_formatting("en").unwrap();
    for (position, expected) in [
        (NumberPaddingPosition::BeforePrefix, "····$1.k"),
        (NumberPaddingPosition::AfterPrefix, "$····1.k"),
        (NumberPaddingPosition::BeforeSuffix, "$1.····k"),
        (NumberPaddingPosition::AfterSuffix, "$1.k····"),
    ] {
        let mut pattern = numbers.decimal_pattern;
        pattern.negative = None;
        pattern.positive.prefix = NumberAffix::CurrencySymbol;
        pattern.positive.suffix = NumberAffix::Literal("k");
        pattern.positive.min_fraction_digits = 0;
        pattern.positive.max_fraction_digits = 0;
        pattern.positive.decimal_separator_always = true;
        pattern.positive.padding = Some(NumberPadding {
            character: '·',
            width: 8,
            position,
        });
        let name = format!("padding{}", plans.len());
        let plan = number_plan(&pattern, Some("$"), &numbers, None, &mut bindings);
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! { name, plan => plan.clone(), exported => false },
        ));
        plans.push(plan);
        checks.push_str(&format!("assert.equal({name}('1.2'), {expected:?});\n"));
    }
    source.push_str(&number_helpers(&plans));
    source.push_str(&checks);
    source.push_str("console.log('ok');\n");
    assert_eq!(execute(&source), "ok");
}
