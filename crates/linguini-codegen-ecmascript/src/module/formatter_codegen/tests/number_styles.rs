use super::*;

fn corpus(style: &str) -> serde_json::Value {
    let source = match style {
        "percent" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codegen/percent-styles.json"
        )),
        "scientific" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codegen/scientific-styles.json"
        )),
        _ => unreachable!(),
    };
    serde_json::from_str(source).unwrap()
}

#[test]
fn selected_numeric_styles_execute_the_independent_icu_corpora() {
    for style in ["percent", "scientific"] {
        let selected = request(IrFormatterKind::Number, &[("style", style)]);
        let name = formatter_binding(&selected);
        for case in corpus(style)["cases"].as_array().unwrap() {
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
            ] {
                assert!(!runtime.contains(absent), "{}: {absent}", case["locale"]);
            }
            if style == "percent" {
                assert!(!runtime.contains("scientific"));
            } else {
                assert!(!runtime.contains("scaleBy"));
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
for(const input of ['--1','1e8192','9'.repeat(8193),BigInt('9'.repeat(8193))]) assert.throws(()=>{name}(input),RangeError);
if({style:?}==='percent') for(const input of ['9'.repeat(8191),BigInt('9'.repeat(8191))]) assert.throws(()=>{name}(input),RangeError);
assert.doesNotThrow(()=>{name}('9'.repeat({limit})));
const marker=Error('coercion failed');
assert.throws(()=>{name}({{toString(){{throw marker;}}}}),error=>error===marker);
console.log('ok');
"#,
                samples = case["samples"],
                limit = if style == "percent" { 8190 } else { 8192 }
            );
            assert_eq!(execute(&script), "ok", "{}", case["locale"]);
        }
    }
}

#[test]
fn numeric_style_fixtures_share_only_selected_helpers() {
    for style in ["percent", "scientific"] {
        let mut bindings = DigitBindings::default();
        let mut plans = Vec::new();
        let mut source = crate::templates::render(
            "formatters/types",
            minijinja::context! { needs_number => true },
        );
        for case in corpus(style)["cases"].as_array().unwrap() {
            let locale = case["locale"].as_str().unwrap();
            let compiled = linguini_codegen::NumberFormatPlan::number(locale, style).unwrap();
            let plan = number_plan_context(&compiled, &mut bindings);
            let name = format!("{style}_{}", locale.replace('-', "_"));
            source.push_str(&crate::templates::render(
                "formatters/number",
                minijinja::context! {name,plan => plan.clone(),exported => true},
            ));
            plans.push(plan);
        }
        source.push_str(&number_helpers(&plans));
        assert_eq!(source.matches("function parseGeneratedDecimal(").count(), 1);
        assert_eq!(
            source.matches("function scaleBy100(").count(),
            usize::from(style == "percent")
        );
        assert_eq!(
            source
                .matches("function scientificGeneratedDecimal(")
                .count(),
            usize::from(style == "scientific")
        );
        assert_eq!(
            source.matches("function roundTo0FractionDigits(").count(),
            1
        );
        let filename = if style == "percent" {
            "percent.js"
        } else {
            "scientific-style.js"
        };
        let snapshot = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../tests/fixtures/golden/snapshots/js-runtime/{filename}"
        ));
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&snapshot, &source).unwrap();
        }
        assert_eq!(source, std::fs::read_to_string(snapshot).unwrap());
    }
}
