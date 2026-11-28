use super::*;

fn corpus() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/percent-styles.json"
    )))
    .unwrap()
}

#[test]
fn selected_percent_styles_execute_the_independent_icu_corpus() {
    let selected = request(IrFormatterKind::Number, &[("style", "percent")]);
    let name = formatter_binding(&selected);
    for case in corpus()["cases"].as_array().unwrap() {
        let runtime = formatter_data_declaration(
            case["locale"].as_str().unwrap(),
            std::slice::from_ref(&selected),
            false,
        );
        for absent in [
            "function formatNumber(",
            "formatCurrency",
            "coerceDate",
            "roundTo3FractionDigits",
            "scientific",
        ] {
            assert!(!runtime.contains(absent), "{}: {absent}", case["locale"]);
        }
        let script = format!(
            r#"
{runtime}
import assert from 'node:assert/strict';
Intl.NumberFormat=class{{constructor(){{throw Error('no host formatter');}}}};
for(const {{input,expected}} of {samples}) {{
  assert.equal({name}(input),expected,input);
  const number=Number(input);
  if(String(number)===input || input==='-0') assert.equal({name}(number),expected,input);
  if(/^-?\d+$/.test(input) && input!=='-0') assert.equal({name}(BigInt(input)),expected,input);
  let reads=0;
  assert.equal({name}({{[Symbol.toPrimitive](hint){{assert.equal(hint,'string');reads++;return input;}}}}),expected,input);
  assert.equal(reads,1);
}}
for(const input of [NaN,Infinity,-Infinity]) assert.equal({name}(input),String(input));
for(const input of ['--1','1e8192','9'.repeat(8191),BigInt('9'.repeat(8191))]) assert.throws(()=>{name}(input),RangeError);
const marker=Error('coercion failed');
assert.throws(()=>{name}({{toString(){{throw marker;}}}}),error=>error===marker);
console.log('ok');
"#,
            samples = case["samples"]
        );
        assert_eq!(execute(&script), "ok", "{}", case["locale"]);
    }
}

#[test]
fn percent_fixture_shares_selected_scale_precision_and_digit_helpers() {
    let mut bindings = DigitBindings::default();
    let mut plans = Vec::new();
    let mut source = crate::templates::render(
        "formatters/types",
        minijinja::context! { needs_number => true },
    );
    for case in corpus()["cases"].as_array().unwrap() {
        let locale = case["locale"].as_str().unwrap();
        let compiled = linguini_codegen::NumberFormatPlan::number(locale, "percent").unwrap();
        let plan = number_plan_context(&compiled, &mut bindings);
        let name = format!("percent_{}", locale.replace('-', "_"));
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! {name,plan => plan.clone(),exported => true},
        ));
        plans.push(plan);
    }
    source.push_str(&number_helpers(&plans));
    assert_eq!(source.matches("function parseGeneratedDecimal(").count(), 1);
    assert_eq!(source.matches("function scaleBy100(").count(), 1);
    assert_eq!(
        source.matches("function roundTo0FractionDigits(").count(),
        1
    );
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-runtime/percent.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &source).unwrap();
    }
    assert_eq!(source, std::fs::read_to_string(snapshot).unwrap());
}
