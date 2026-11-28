use super::*;

#[test]
fn wide_numeric_quanta_match_independent_exact_half_up_results() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/wide-number-quantum.json"
    )))
    .unwrap();
    let numbers = compiled_number_formatting("en").unwrap();
    let mut bindings = DigitBindings::default();
    let mut plans = Vec::new();
    let mut source = String::from("import assert from 'node:assert/strict';\n");
    let mut checks = String::new();
    for (index, case) in corpus["cases"].as_array().unwrap().iter().enumerate() {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.primary_group_size = None;
        pattern.positive.secondary_group_size = None;
        pattern.positive.min_fraction_digits = case["fraction_digits"].as_u64().unwrap() as u8;
        pattern.positive.max_fraction_digits = pattern.positive.min_fraction_digits;
        let compiled = linguini_codegen::NumberFormatPlan::new(
            &pattern,
            None,
            &numbers,
            case["quantum"].as_str(),
        )
        .unwrap();
        let plan = number_plan_context(&compiled, &mut bindings);
        let name = format!("wideQuantum{index}");
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! {
                name, plan => plan.clone(), exported => false,
            },
        ));
        plans.push(plan);
        for sample in case["samples"].as_array().unwrap() {
            checks.push_str(&format!(
                "assert.equal({name}({}),{});\n",
                sample["input"], sample["expected"]
            ));
        }
    }
    source.push_str(&number_helpers(&plans));
    source.push_str(&checks);
    source.push_str("console.log('ok');\n");
    assert_eq!(execute(&source), "ok");
}

#[test]
fn significant_helpers_require_normalized_inputs_from_every_caller() {
    let numbers = compiled_number_formatting("en").unwrap();
    for (width, minimum, ordinary_precision, reverse) in [
        (1, 3, None, false),
        (3, 3, None, false),
        (4, 3, None, false),
        (1, 2, None, false),
        (3, 2, None, false),
        (4, 2, None, false),
        (3, 3, Some(3), false),
        (3, 3, Some(3), true),
        (3, 3, Some(4), false),
        (3, 3, Some(4), true),
    ] {
        let mut bindings = DigitBindings::default();
        let mut scientific = numbers.decimal_pattern;
        scientific.positive.max_integer_digits = Some(width);
        scientific.positive.exponent_digits = Some(1);
        scientific.positive.min_fraction_digits = minimum - 1;
        scientific.positive.max_fraction_digits = 2;
        if width == 1 {
            scientific.positive.min_significant_digits = Some(minimum);
            scientific.positive.max_significant_digits = Some(3);
        }
        let mut plans = vec![number_plan(
            &scientific,
            None,
            &numbers,
            None,
            &mut bindings,
        )];
        if let Some(precision) = ordinary_precision {
            let mut ordinary = numbers.decimal_pattern;
            ordinary.positive.min_significant_digits = Some(precision);
            ordinary.positive.max_significant_digits = Some(precision);
            plans.push(number_plan(&ordinary, None, &numbers, None, &mut bindings));
        } else {
            // Sharing the unit-rounding template must retain ordinary record reuse.
            plans.push(number_plan(
                &numbers.decimal_pattern,
                None,
                &numbers,
                None,
                &mut bindings,
            ));
        }
        if reverse {
            plans.reverse();
        }
        let helpers = number_helpers(&plans);
        let normalized = ordinary_precision != Some(3);
        let needs_padding = minimum < 3 || ordinary_precision == Some(3);
        assert_eq!(
            helpers.contains(&format!("function padTo{minimum}SignificantDigits(")),
            needs_padding,
            "fixed scientific precision must not select an unused padding helper"
        );
        let padding_checks = if needs_padding {
            format!(
                "assert.equal(padTo{minimum}SignificantDigits('1',''),{zeros:?});\n\
                 assert.equal(padTo{minimum}SignificantDigits('0',''),{zeros:?});\n\
                 assert.equal(padTo{minimum}SignificantDigits('12','3'),'3');",
                zeros = "0".repeat(usize::from(minimum) - 1)
            )
        } else {
            String::new()
        };
        assert_eq!(
            helpers.contains("function roundGeneratedFraction("),
            width != 1,
            "only a single one-digit normalized precision can use literal fraction rounding"
        );
        if width != 1 {
            assert_eq!(
                helpers.contains("if (fractionDigits > MAX_GENERATED_DECIMAL_DIGITS)"),
                ordinary_precision.is_some(),
                "ordinary significant inputs require the expansion guard"
            );
        }
        let ordinary_checks = if let Some(precision) = ordinary_precision {
            format!(
                r#"
const tiny = Object.freeze({{integer:'0',fraction:'00123456'}});
assert.deepEqual(roundTo{precision}SignificantDigits(tiny),{{integer:'0',fraction:{fraction:?}}});
assert.equal(padTo{precision}SignificantDigits('0','001'),{padded:?});
assert.throws(()=>roundTo{precision}SignificantDigits({{integer:'0',fraction:'0'.repeat(8190)+'1'}}),RangeError);
"#,
                fraction = if precision == 3 { "00123" } else { "001235" },
                padded = if precision == 3 { "00100" } else { "001000" },
            )
        } else {
            "const ordinary=Object.freeze(parseGeneratedDecimal('1.25')); assert.equal(roundTo3FractionDigits(ordinary),ordinary);".to_owned()
        };
        let samples = match width {
            1 => "[['0','','0',''],['1','','1','00'],['1','23456','1','23'],['9','995','10','00']]",
            3 => "[['0','','0',''],['1','','1','00'],['12','3456','12','3'],['123','45','123',''],['999','5','1000','']]",
            4 => "[['0','','0',''],['1','','1','00'],['1234','5','1230',''],['9999','5','10000','']]",
            _ => unreachable!(),
        };
        let script = format!(
            r#"{helpers}
import assert from 'node:assert/strict';
const originalSearch=String.prototype.search;
let searches=0;
String.prototype.search=function(pattern){{searches++;return originalSearch.call(this,pattern);}};
for (const [integer,fraction,expectedInteger,expectedFraction] of {samples}) {{
  const decimal=Object.freeze({{integer,fraction}});
  const result=roundTo3SignificantDigits(decimal);
  assert.deepEqual(result,{{integer:expectedInteger,fraction:expectedFraction}});
  assert.deepEqual(decimal,{{integer,fraction}},'rounding mutated its input');
  assert.notEqual(result,decimal,'significant rounding retains its fresh padded record');
}}
{padding_checks}
assert.equal(searches===0,{normalized},'all callers must support the normalized shape proof');
{ordinary_checks}
String.prototype.search=originalSearch;
console.log('ok');
"#
        );
        assert_eq!(
            execute(&script),
            "ok",
            "width={width}, ordinary={ordinary_precision:?}, reverse={reverse}"
        );
    }
}

#[test]
fn unchanged_rounding_reuses_canonical_records_without_padding_or_mutation() {
    let mut digit_bindings = DigitBindings::default();
    let numbers = compiled_number_formatting("en").unwrap();
    let mut plans = Vec::new();
    for precision in 0..=6 {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.max_fraction_digits = precision;
        plans.push(number_plan(
            &pattern,
            None,
            &numbers,
            None,
            &mut digit_bindings,
        ));
    }
    let script = format!(
        r#"{}
import assert from 'node:assert/strict';
const rounders=[roundTo0FractionDigits,roundTo1FractionDigits,roundTo2FractionDigits,roundTo3FractionDigits,roundTo4FractionDigits,roundTo5FractionDigits,roundTo6FractionDigits];
for(const [precision,round] of rounders.entries()) {{
  for(const value of [0,-0,1,-1,1234n,'1.25','-1.25','0001.2500','1.'+'0'.repeat(precision)+'5']) {{
    const decimal=parseGeneratedDecimal(value),before={{...decimal}};
    const result=round(decimal);
    assert.deepEqual(decimal,before,'rounding mutated its canonical input');
    if(decimal.fraction.length<=precision) {{
      assert.equal(result,decimal,'unchanged digits must reuse the input');
      assert.equal(result.fraction,decimal.fraction,'unchanged digits must not be padded');
    }} else {{
      assert.notEqual(result,decimal,'truncated/rounded results must not alter the input');
      assert.equal(result.fraction.length,precision);
    }}
  }}
}}
console.log('ok');
"#,
        number_helpers(&plans)
    );
    assert_eq!(execute(&script), "ok");
}

#[test]
fn rounding_padding_matches_selected_minima_and_retains_quantum_rounding() {
    let mut digit_bindings = DigitBindings::default();
    let numbers = compiled_number_formatting("en").unwrap();
    let mut source = String::new();
    let mut plans = Vec::new();
    let mut checks = String::new();
    for minimum in 0..=3 {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.min_fraction_digits = minimum;
        pattern.positive.max_fraction_digits = 3;
        pattern.positive.primary_group_size = None;
        let plan = number_plan(&pattern, None, &numbers, None, &mut digit_bindings);
        let name = format!("minimum{minimum}");
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! {name,plan=>plan.clone(),exported=>false},
        ));
        plans.push(plan);
        for (value, integer, fraction) in [
            ("1", "1", ""),
            ("-0", "-0", ""),
            ("1.2", "1", "2"),
            ("1.25", "1", "25"),
            ("1.2344", "1", "234"),
            ("1.2345", "1", "235"),
            ("9.9995", "10", ""),
        ] {
            let fraction = format!("{fraction:0<width$}", width = usize::from(minimum));
            let expected = if fraction.is_empty() {
                integer.to_owned()
            } else {
                format!("{integer}.{fraction}")
            };
            checks.push_str(&format!("assert.equal({name}({value:?}),{expected:?});\n"));
        }
    }
    let mut quantum = numbers.decimal_pattern;
    quantum.positive.max_fraction_digits = 2;
    quantum.positive.rounding_increment = Some("5");
    plans.push(number_plan(
        &quantum,
        None,
        &numbers,
        None,
        &mut digit_bindings,
    ));
    let script = format!(
        r#"{source}{}
import assert from 'node:assert/strict';
{checks}
const decimal=parseGeneratedDecimal('1.02');
assert.notEqual(roundTo2FractionDigitsIncrement5(decimal),decimal);
assert.deepEqual(roundTo2FractionDigitsIncrement5(decimal),{{integer:'1',fraction:'00'}});
assert.deepEqual(roundTo2FractionDigitsIncrement5(parseGeneratedDecimal('1.025')),{{integer:'1',fraction:'05'}});
console.log('ok');
"#,
        number_helpers(&plans)
    );
    assert_eq!(execute(&script), "ok");
}
