use super::*;

#[test]
fn unchanged_rounding_reuses_canonical_records_without_padding_or_mutation() {
    let numbers = compiled_number_formatting("en").unwrap();
    let mut plans = Vec::new();
    for precision in 0..=6 {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.max_fraction_digits = precision;
        plans.push(number_plan(&pattern, None, &numbers, None));
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
    let numbers = compiled_number_formatting("en").unwrap();
    let mut source = String::new();
    let mut plans = Vec::new();
    let mut checks = String::new();
    for minimum in 0..=3 {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.min_fraction_digits = minimum;
        pattern.positive.max_fraction_digits = 3;
        pattern.positive.primary_group_size = None;
        let plan = number_plan(&pattern, None, &numbers, None);
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
    plans.push(number_plan(&quantum, None, &numbers, None));
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
