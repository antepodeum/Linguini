use super::{generate_plural_function, generate_plural_function_with_visibility};
use linguini_cldr::built_in_plural_rules;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn generated_public_plural_function_snapshot_is_stable() {
    let rules = built_in_plural_rules("ru").expect("rules");
    let output = generate_plural_function("pluralRu", &rules);
    assert_eq!(
        output,
        generate_plural_function_with_visibility("pluralRu", &rules, true)
    );
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/codegen-js-plural-ru.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &output).expect("write plural snapshot");
    }

    assert_eq!(
        output,
        std::fs::read_to_string(snapshot).expect("read plural snapshot")
    );
}

#[test]
fn generated_plural_runtime_matches_canonical_and_native_number_semantics() {
    let rules = built_in_plural_rules("fr").expect("French plural rules");
    assert!(rules.categories.iter().any(|category| {
        category
            .rule
            .conditions
            .iter()
            .flat_map(|condition| &condition.relations)
            .any(|relation| relation.expression.operand == linguini_cldr::Operand::E)
    }));

    let valid = [
        "1.2c6",
        "1.20050c3",
        "1c3",
        "2c3",
        "  +1.2c6  ",
        "0c6",
        "1.",
    ];
    let malformed = [
        "1c",
        "1c-3",
        "1c+3",
        "1c2e3",
        "1C3",
        "123e5",
        "1.0000001e6",
        "1e0",
        "-123E5",
        "1e-2",
        "1e+2",
        ".5",
        "1e2.0",
        "1c2x",
        "1e2e3",
        "--1",
    ];

    let cases = valid
        .iter()
        .map(|sample| {
            let expected = rules.category_for(sample).expect("Rust category");
            format!("[{sample:?}, {expected:?}]")
        })
        .collect::<Vec<_>>()
        .join(",\n");
    for sample in malformed {
        assert!(
            rules.category_for(sample).is_err(),
            "Rust unexpectedly accepted {sample:?}"
        );
    }

    let generated = generate_plural_function_with_visibility("pluralFr", &rules, true);
    let malformed = malformed
        .iter()
        .map(|sample| format!("{sample:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let script = format!(
        r#"{generated}
for (const [sample, expected] of [{cases}]) {{
  const actual = pluralFr(sample);
  if (actual !== expected) {{
throw new Error(`${{sample}}: expected ${{expected}}, got ${{actual}}`);
  }}
}}
for (const category of ["zero", "one", "two", "few", "many", "other"]) {{
  if (pluralFr(category) !== category) {{
throw new Error(`${{category}}: plural conversion is not idempotent`);
  }}
}}
for (const [value, expanded] of [
  [1e21, "1000000000000000000000"],
  [1e-7, "0.0000001"],
]) {{
  if (pluralFr(value) !== pluralFr(expanded)) {{
throw new Error(`${{value}}: native scientific category differs from ${{expanded}}`);
  }}
}}
if (pluralFr(1000000000000000000000n) !== "many") {{
  throw new Error("bigint plural input must remain exact");
}}
for (const sample of [{malformed}]) {{
  let rejected = false;
  try {{ pluralFr(sample); }} catch (error) {{ rejected = error instanceof RangeError; }}
  if (!rejected) throw new Error(`${{sample}}: expected RangeError`);
}}
for (const value of [NaN, Infinity, -Infinity]) {{
  let rejected = false;
  try {{ pluralFr(value); }} catch (error) {{ rejected = error instanceof RangeError; }}
  if (!rejected) throw new Error(`${{value}}: expected RangeError`);
}}
"#
    );

    execute(&script);
}

#[test]
fn native_integer_suffixes_and_proven_fractions_skip_parsing_preserving_other_inputs() {
    let samples = (-200_i64..=200)
        .chain(9_007_199_254_740_980..=9_007_199_254_740_991)
        .chain(-9_007_199_254_740_991..=-9_007_199_254_740_980)
        .collect::<Vec<_>>();
    for locale in ["ru", "uk"] {
        let rules = built_in_plural_rules(locale).unwrap();
        let generated = generate_plural_function("plural", &rules);
        let cases = samples
            .iter()
            .map(|value| {
                format!(
                    "[{value}, {:?}]",
                    rules.category_for(&value.to_string()).unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        execute(&format!(
            r#"{generated}
const cases = [{cases}, [-0, "many"]];
const fractions = [0.1, -0.1, 1.5, 1.0000000000000002, 1e-7, -1e-7,
  Number.MIN_VALUE, -Number.MIN_VALUE, 1e-308, 4503599627370495.5];
for (let exponent = -324; exponent <= 15; exponent++) {{
  for (const mantissa of [1, 1.2345678901234567, 9]) {{
    const value = Number(`${{mantissa}}e${{exponent}}`);
    if (Number.isFinite(value) && !Number.isInteger(value)) fractions.push(value, -value);
  }}
}}
for (const value of fractions) cases.push([value, "other"]);
const originalString = globalThis.String;
const originalExec = RegExp.prototype.exec;
let conversions = 0, matches = 0;
let actual;
try {{
  globalThis.String = () => {{ conversions++; throw new Error("unexpected string conversion"); }};
  RegExp.prototype.exec = () => {{ matches++; throw new Error("unexpected regex parsing"); }};
  actual = cases.map(([value]) => plural(value));
}} finally {{
  globalThis.String = originalString;
  RegExp.prototype.exec = originalExec;
}}
if (conversions || matches) throw new Error("proven native inputs parsed text");
for (let index = 0; index < cases.length; index++) {{
  if (actual[index] !== cases[index][1]) throw new Error(JSON.stringify([cases[index], actual[index]]));
}}
for (const category of ["zero", "one", "two", "few", "many", "other"]) {{
  if (plural(category) !== category) throw new Error("lost category passthrough");
}}
for (const [value, expected] of [
  ["1.0", "other"], ["1.00", "other"], ["1c3", "many"], [1.5, "other"],
  [1e-7, "other"], [1e21, "many"], [9007199254740992, "few"],
  ["0".repeat(8191) + "1", "one"], [BigInt("1" + "0".repeat(8189) + "21"), "one"],
]) {{
  if (plural(value) !== expected) throw new Error("non-safe-integer input changed");
}}
let hints = [];
const boxed = {{ [Symbol.toPrimitive](hint) {{ hints.push(hint); return "21"; }} }};
if (plural(boxed) !== "one" || JSON.stringify(hints) !== '["string"]') throw new Error("coercion changed");
const sentinel = new Error("sentinel");
try {{
  plural({{ [Symbol.toPrimitive]() {{ throw sentinel; }} }});
  throw new Error("coercion error swallowed");
}} catch (error) {{ if (error !== sentinel) throw error; }}
for (const value of ["1".repeat(8193), BigInt("1".repeat(8193)), "1c8193", "invalid", NaN, Infinity, -Infinity]) {{
  let rejected = false;
  try {{ plural(value); }} catch (error) {{ rejected = error instanceof RangeError; }}
  if (!rejected) throw new Error("input validation bypassed");
}}
"#
        ));
    }
    // An integer suffix shortcut alone does not prove fractions are always `other`.
    for guard in ["v within 0..1", "v % 2 = 0"] {
        let rules = linguini_cldr::PluralRules {
            locale: "fraction-probe".to_owned(),
            categories: [("one", 1), ("few", 2)]
                .map(|(category, residue)| {
                    let source = format!("{guard} and i % 10 = {residue}");
                    linguini_cldr::PluralCategoryRule {
                        category: category.to_owned(),
                        rule: linguini_cldr::parse_plural_rule(&source).unwrap(),
                        source,
                    }
                })
                .to_vec(),
        };
        let generated = generate_plural_function("plural", &rules);
        let cases = ["1.5", "2.5", "1.25", "2.25"]
            .map(|value| format!("[{value}, {:?}]", rules.category_for(value).unwrap()))
            .join(",");
        execute(&format!(
            r#"{generated}
for (const [value, expected] of [{cases}]) {{
  if (plural(value) !== expected) throw new Error("permissive fractional guard bypassed");
}}
"#
        ));
    }
}

#[test]
fn constant_plural_results_skip_native_number_parsing_and_keep_validation() {
    use linguini_cldr::{Condition, PluralCategoryRule, PluralRule, PluralRules};
    let mut rules = ["ja", "zh", "ko", "th"]
        .map(|locale| built_in_plural_rules(locale).unwrap())
        .to_vec();
    rules.push(PluralRules {
        locale: "constant-one".to_owned(),
        categories: vec![PluralCategoryRule {
            category: "one".to_owned(),
            source: String::new(),
            rule: PluralRule {
                conditions: vec![Condition { relations: vec![] }],
            },
        }],
    });
    for rules in rules {
        let expected = rules.category_for("1.5").unwrap();
        let generated = generate_plural_function("plural", &rules);
        execute(&format!(
            r#"{generated}
const numbers = [0, -0, 1, -1, 0.5, -0.5, Number.MIN_VALUE, Number.MAX_VALUE,
  Number.MAX_SAFE_INTEGER, 9007199254740992];
for (let exponent = -324; exponent <= 308; exponent++) {{
  for (const mantissa of [1, 1.2345678901234567, 9]) {{
    const value = Number(`${{mantissa}}e${{exponent}}`);
    if (Number.isFinite(value)) numbers.push(value, -value);
  }}
}}
const originalString = globalThis.String;
const originalExec = RegExp.prototype.exec;
let conversions = 0, matches = 0;
try {{
  globalThis.String = value => {{ conversions++; return originalString(value); }};
  RegExp.prototype.exec = function(value) {{ matches++; return originalExec.call(this, value); }};
  for (const value of numbers) {{
    if (plural(value) !== {expected:?}) throw Error("constant numeric category changed");
  }}
}} finally {{ globalThis.String = originalString; RegExp.prototype.exec = originalExec; }}
if (conversions || matches) throw Error("constant numeric result parsed text");
for (const value of ["0001.2500", "-0", " +1. ", "1c3", "1c8191",
  "0".repeat(8191) + "1", "1." + "0".repeat(8191),
  BigInt("9".repeat(8192)), -BigInt("9".repeat(8192))]) {{
  if (plural(value) !== {expected:?}) throw Error("valid nonnumber category changed");
}}
for (const category of ["zero", "one", "two", "few", "many", "other"]) {{
  if (plural(category) !== category) throw Error("category passthrough changed");
}}
let hints = [];
const boxed = {{ [Symbol.toPrimitive](hint) {{ hints.push(hint); return "1.00c3"; }} }};
if (plural(boxed) !== {expected:?} || JSON.stringify(hints) !== '["string"]') throw Error("coercion changed");
const sentinel = new Error("sentinel");
try {{
  plural({{ [Symbol.toPrimitive]() {{ throw sentinel; }} }});
  throw Error("coercion error swallowed");
}} catch (error) {{ if (error !== sentinel) throw error; }}
for (const value of [NaN, Infinity, -Infinity, "1c8192", "1c-1", "1c+1", "1e3", "1E3",
  "invalid", ".5", "", "1".repeat(8193), "1." + "0".repeat(8192),
  BigInt("9".repeat(8193)), -BigInt("9".repeat(8193)),
  {{ [Symbol.toPrimitive]() {{ return "one"; }} }}, new Number(1e21)]) {{
  let rejected = false;
  try {{ plural(value); }} catch (error) {{ rejected = error instanceof RangeError; }}
  if (!rejected) throw Error("constant result bypassed input validation");
}}
"#
        ));
    }
}

fn execute(script: &str) {
    let mut child = Command::new("node")
        .args(["--input-type=module", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node.js is required to execute the generated plural runtime test");
    child
        .stdin
        .take()
        .expect("Node stdin")
        .write_all(script.as_bytes())
        .expect("write generated runtime to Node");
    let output = child.wait_with_output().expect("wait for Node");
    assert!(
        output.status.success(),
        "generated plural runtime failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn reduced_classifiers_preserve_observed_categories_and_validation_across_families() {
    use linguini_codegen::PluralCategoryDemand;
    let mut script = String::from("import assert from 'node:assert/strict';\n");
    for locale in [
        "en", "ru", "fr", "ar", "cy", "br", "ga", "gd", "kw", "lt", "lv", "mt", "ro", "sl", "dsb",
        "pl", "cs", "sk", "is", "mk", "he", "pt", "es", "ja",
    ] {
        let rules = built_in_plural_rules(locale).unwrap();
        for keys in [
            vec![],
            vec!["one"],
            vec!["one", "few"],
            vec!["zero", "two", "many"],
        ] {
            let categories = keys.iter().map(|key| (*key).to_owned()).collect();
            let full = generate_plural_function_with_visibility("full", &rules, false);
            let reduced = super::generate_plural_function_for_demand(
                "reduced",
                &rules,
                &PluralCategoryDemand::Categories(categories),
            );
            script.push_str(&format!(r#"{{
const full=(()=>{{{full};return full;}})();
const reduced=(()=>{{{reduced};return reduced;}})();
const keys=new Set({keys:?});
const observation=(fn,value)=>{{try{{const category=fn(value);return ['value',keys.has(category)?category:null];}}catch(error){{return [error.constructor.name,error.message];}}}};
for(const value of [...Array.from({{length:401}},(_,i)=>i-200),'0.0','1.00','1.2','11.20','1c6','1.2c6','1'.repeat(8192),'1'.repeat(8193),1e30,Number.MIN_VALUE,Number.MAX_VALUE,10000000000000000000001n,NaN,Infinity,null,undefined,true,'bad','zero','one','two','few','many','other']){{
 assert.deepEqual(observation(reduced,value),observation(full,value),{locale:?}+':'+String(value));
}}
let reads=0;const input={{toString(){{reads++;return '21';}}}};
reduced(input);assert.equal(reads,1);
const failure=new Error('coercion');assert.throws(()=>reduced({{toString(){{throw failure;}}}}),error=>error===failure);
}}
"#));
        }
    }
    execute(&script);
}

#[test]
fn compiled_predicates_match_cldr_across_locales_operators_and_operand_demand() {
    use linguini_cldr::{parse_plural_rule, PluralCategoryRule, PluralRules};
    let mut rules = [
        "en", "ru", "fr", "ar", "cy", "br", "ga", "gd", "kw", "lt", "lv", "mt", "ro", "sl", "dsb",
        "pl", "cs", "sk", "is", "mk", "he", "pt", "es", "ja",
    ]
    .into_iter()
    .map(|locale| built_in_plural_rules(locale).unwrap())
    .collect::<Vec<_>>();
    for expression in [
        "n within 0..2",
        "n not within 0..2",
        "n = 0..2",
        "n != 0..2",
        "n in 0..2",
        "n not in 0..2",
        "n % 10 within 1..2",
        "n % 0 within 1..2",
        "n % 10 = 1 or i % 10 = 2",
        "c % 10 = 1 or e % 10 = 2",
        "n not within 1,3..5",
        "i != 1,3..5",
        "v % 10 = 1",
        "i = 1,3..5",
        "v = 0..2",
        "w = 1..2",
        "c = 3",
        "e % 10 = 6",
        "",
        "n = 1 or i = 2 or v = 1 or w = 2 or f = 3 or t = 4 or c = 5 or e = 6",
    ] {
        rules.push(PluralRules {
            locale: expression.to_owned(),
            categories: vec![PluralCategoryRule {
                category: "one".to_owned(),
                source: expression.to_owned(),
                rule: parse_plural_rule(expression).unwrap(),
            }],
        });
    }
    let samples = (0..=200)
        .map(|value| value.to_string())
        .chain(
            [
                "0.0",
                "0.1",
                "0.10",
                "1.0",
                "1.1",
                "1.10",
                "1.20050",
                "2.0",
                "2.1",
                "3.14",
                "10.01",
                "21.50",
                "1000",
                "1000000",
                "1000001",
                "1.2c3",
                "1.2c6",
                "-1",
                "-2.5",
                "  +1.2c6  ",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .collect::<Vec<_>>();
    let mut script = String::new();
    for rules in rules {
        let generated = generate_plural_function_with_visibility("plural", &rules, false);
        assert!(!generated.contains("pluralOperandMatches"));
        assert!(!generated.contains("ranges.some"));
        assert!(!generated.contains("pluralOperands("));
        assert!(!generated.contains("??="));
        let cases = samples
            .iter()
            .map(|sample| format!("[{sample:?}, {:?}]", rules.category_for(sample).unwrap()))
            .collect::<Vec<_>>()
            .join(",");
        script.push_str(&format!("{{\n{generated}\nfor (const [sample, expected] of [{cases}]) {{ const actual = plural(sample); if (actual !== expected) throw new Error(JSON.stringify([{:?}, sample, expected, actual])); }}\n}}\n", rules.locale));
    }
    execute(&script);

    let english = generate_plural_function("pluralEn", &built_in_plural_rules("en").unwrap());
    for unused in [
        "trimmedFraction",
        "const fraction =",
        "compactExponent",
        "const hasFraction",
        "const significantFractionDigits",
        "const fractionValue",
        "const significantFractionValue",
    ] {
        assert!(
            !english.contains(unused),
            "unused English operation: {unused}"
        );
    }
    let russian = generate_plural_function("pluralRu", &built_in_plural_rules("ru").unwrap());
    assert_eq!(russian.matches("const integerTailModulo10 =").count(), 1);
    assert!(!russian.contains("integerTail % 100"));
    assert_eq!(russian.matches("Math.abs(value) % 100").count(), 1);
    assert!(!russian.contains("BigInt("));
    execute(&format!(
        r#"{russian}
const originalBigInt = globalThis.BigInt;
globalThis.BigInt = () => {{ throw new Error("fractional Russian input must skip bigint construction"); }};
try {{
  for (const sample of ["1.0", "1.50", 1.5, 1e-7]) {{
    if (pluralRu(sample) !== "other") throw new Error("Russian fractional input lost visible digits");
  }}
}} finally {{ globalThis.BigInt = originalBigInt; }}
"#
    ));
    let japanese = generate_plural_function("pluralJa", &built_in_plural_rules("ja").unwrap());
    assert!(!japanese.contains("BigInt("));
    assert!(!japanese.contains("const digits"));
    execute(&format!("{japanese}\nfor (const value of ['invalid', NaN, Infinity, '1'.repeat(8193)]) {{ let rejected = false; try {{ pluralJa(value); }} catch(error) {{ rejected = error instanceof RangeError; }} if (!rejected) throw new Error('unused operands must retain validation'); }}"));
}

#[test]
fn all_operand_input_semantics_remain_exact_when_requested() {
    use linguini_cldr::{parse_plural_rule, PluralCategoryRule, PluralRules};
    let mut script = String::new();
    for expression in [
        "n within 1200..1201",
        "i = 1200",
        "v = 2",
        "w = 1",
        "f = 50",
        "t = 5",
        "c = 3",
        "e = 3",
        "n % 100 = 1",
        "i % 100 = 1",
        "c = 0",
        "e = 0",
    ] {
        let rules = PluralRules {
            locale: "probe".to_owned(),
            categories: vec![PluralCategoryRule {
                category: "one".to_owned(),
                source: expression.to_owned(),
                rule: parse_plural_rule(expression).unwrap(),
            }],
        };
        let generated = generate_plural_function_with_visibility("pluralProbe", &rules, false);
        if expression.starts_with("c ") || expression.starts_with("e ") {
            assert!(!generated.contains("const digits"));
            assert!(!generated.contains("BigInt("));
        }
        let expected = rules.category_for("1.20050c3").unwrap();
        script.push_str(&format!(r#"{{
{generated}
if (pluralProbe("1.20050c3") !== {expected:?}) throw new Error("compact input lost its exact operands: {expression}");
for (const [value, expanded] of [[1e21, "1000000000000000000000"], [1e-7, "0.0000001"], [-1e21, "-1000000000000000000000"]]) {{
  if (pluralProbe(value) !== pluralProbe(expanded)) throw new Error("native scientific operand differs: {expression}");
}}
if (pluralProbe(1000000000000000000001n) !== pluralProbe("1000000000000000000001")) throw new Error("bigint input lost precision: {expression}");
if ({modulo} && pluralProbe(1000000000000000000001n) !== "one") throw new Error("bigint remainder lost precision: {expression}");
}}
"#, modulo = expression.contains("%")));
    }
    execute(&script);
}

#[test]
fn decimal_moduli_compile_bounded_integer_suffixes_without_expanding_large_integers() {
    use linguini_cldr::{parse_plural_rule, PluralCategoryRule, PluralRules};
    let samples = [
        "0",
        "0001",
        "-00101",
        "+0002",
        "3.01",
        "0.0001",
        "1.20c2",
        "1.2005c3",
        "0.0001c3",
        "9007199254740991",
        "9007199254740992",
        "9007199254740993",
        "999999999999999",
        "9999999999999999",
        "9999999999999999999",
    ]
    .map(str::to_owned)
    .to_vec();
    let mut script = String::new();
    for digits in 1..=19_u32 {
        let modulo = 10_u64.pow(digits);
        for operand in ["i", "n"] {
            for mixed in [false, true] {
                let mut source = format!("{operand} % {modulo} within 1..3");
                if mixed {
                    source.push_str(" or i % 10 = 1");
                }
                let rules = PluralRules {
                    locale: "probe".to_owned(),
                    categories: vec![PluralCategoryRule {
                        category: "one".to_owned(),
                        rule: parse_plural_rule(&source).unwrap(),
                        source,
                    }],
                };
                let generated = generate_plural_function_with_visibility("plural", &rules, false);
                let native_integer = operand == "i" && digits <= 15;
                assert_eq!(
                    generated.matches("let integerTail;").count(),
                    usize::from(native_integer)
                );
                assert_eq!(
                    generated.matches("const integerTail =").count(),
                    usize::from(!native_integer)
                );
                assert!(!generated.contains("const integerValue ="));
                assert!(!generated.contains("const integer ="));
                let mut cases = samples
                    .iter()
                    .map(|sample| {
                        format!("[{sample:?}, {:?}]", rules.category_for(sample).unwrap())
                    })
                    .collect::<Vec<_>>();
                // The canonical Rust evaluator bounds its full integer to u128. These
                // larger generated-runtime inputs have explicit independent expectations.
                for (sample, expected) in [
                    ("1c8191".to_owned(), "other"),
                    ("12c8189".to_owned(), "other"),
                    ("0".repeat(8192), "other"),
                    ("9".repeat(8192), "other"),
                    (
                        format!("{}01", "9".repeat(8190)),
                        if mixed || digits <= 2 { "one" } else { "other" },
                    ),
                    (format!("{}{}1", "9".repeat(8173), "0".repeat(18)), "one"),
                    (
                        format!("{}03.01", "0".repeat(8187)),
                        if operand == "i" { "one" } else { "other" },
                    ),
                ] {
                    cases.push(format!("[{sample:?}, {expected:?}]"));
                }
                let cases = cases.join(",");
                script.push_str(&format!(r#"{{
{generated}
const bigint = globalThis.BigInt;
const repeat = String.prototype.repeat;
globalThis.BigInt = value => {{
  if ({digits} <= 15) throw Error('small integer suffix must use exact Number arithmetic');
  if (String(value).length > {digits}) throw Error('unbounded integer construction');
  return bigint(value);
}};
String.prototype.repeat = function(count) {{
  if ({integer_only} && count >= {digits}) throw Error('unneeded integer zero expansion');
  return repeat.call(this, count);
}};
try {{
  for (const [sample, expected] of [{cases}]) {{
    const actual = plural(sample);
    if (actual !== expected) throw Error(JSON.stringify([{operand:?}, {digits}, sample.slice(-40), sample.length, expected, actual]));
  }}
  for (const [value, expanded] of [[1e21, '1000000000000000000000'], [1e-7, '0.0000001'], [-1e21, '1000000000000000000000'], [100000000000000000001n, '100000000000000000001']]) {{
    if (plural(value) !== plural(expanded)) throw Error('native input suffix differs');
  }}
}} finally {{ globalThis.BigInt = bigint; String.prototype.repeat = repeat; }}
}}
"#, integer_only = operand == "i"));
            }
        }
    }
    execute(&script);
}

#[test]
fn exhaustive_plural_tails_become_returns_and_drop_unused_moduli() {
    use linguini_cldr::{parse_plural_rule, PluralCategoryRule, PluralRules};
    let rules = PluralRules {
        locale: "test".to_owned(),
        categories: [
            ("one", "i % 6 = 0"),
            ("few", "i % 10 = 0"),
            ("many", "i % 30 not in 0,6,10,12,18,20,24"),
        ]
        .into_iter()
        .map(|(category, source)| PluralCategoryRule {
            category: category.to_owned(),
            source: source.to_owned(),
            rule: parse_plural_rule(source).unwrap(),
        })
        .collect(),
    };
    let generated = generate_plural_function_with_visibility("plural", &rules, false);
    assert!(!generated.contains("% 30n"));
    assert!(generated.contains("return \"many\";\n}"));
    let cases = (0..=1000)
        .map(|value| {
            format!(
                "[{value}, {:?}]",
                rules.category_for(&value.to_string()).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    execute(&format!(
        r#"{generated}
for (const [value, expected] of [{cases}]) {{
  for (const sample of [value, -value, String(value), BigInt(value), String(value) + '.5']) {{
    if (plural(sample) !== expected) throw Error(JSON.stringify([sample, expected]));
  }}
}}
for (const category of ['zero', 'one', 'two', 'few', 'many', 'other']) {{
  if (plural(category) !== category) throw Error('category passthrough changed');
}}
for (const invalid of ['invalid', NaN, Infinity, '1'.repeat(8193)]) {{
  let rejected = false;
  try {{ plural(invalid); }} catch (error) {{ rejected = error instanceof RangeError; }}
  if (!rejected) throw Error('exhaustiveness must retain input validation');
}}
"#
    ));
    let russian = generate_plural_function("pluralRu", &built_in_plural_rules("ru").unwrap());
    assert_eq!(russian.matches("return \"many\";").count(), 1);
    assert!(russian.contains("if (visibleFractionDigits !== 0) return \"other\";"));
    assert!(russian.contains("return \"many\";\n}"));
}

#[test]
fn public_classifier_names_preserve_exports_and_leave_runtime_bindings_available() {
    for (name, public_name) in [
        ("String", "String"),
        ("Number", "Number"),
        ("BigInt", "BigInt"),
        ("Math", "Math"),
        ("RangeError", "RangeError"),
        ("undefined", "undefined"),
        ("MAX_PLURAL_DECIMAL_DIGITS", "MAX_PLURAL_DECIMAL_DIGITS"),
        ("throwInvalidPluralNumber", "throwInvalidPluralNumber"),
        ("class", "__lgl_name_636C617373"),
        ("a.b", "__lgl_name_a$b"),
    ] {
        for locale in ["ru", "fr", "ar", "sl"] {
            let rules = built_in_plural_rules(locale).unwrap();
            assert!(rules.category_for("1c8192").is_err());
            let code = generate_plural_function(name, &rules);
            if name == "String" && locale == "ru" {
                let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/golden/snapshots/js-runtime/plural-name.js");
                if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
                    std::fs::write(&path, &code).unwrap();
                }
                assert_eq!(code, std::fs::read_to_string(path).unwrap());
            }
            let cases = [
                "0",
                "1",
                "2",
                "5",
                "11",
                "1.00",
                "1.2",
                "1000001",
                "9007199254740993",
                "1c3",
            ]
            .map(|value| serde_json::json!([value, rules.category_for(value).unwrap()]));
            let script = format!(
                r#"
import assert from 'node:assert/strict';
const module=await import('./classifier.js');
assert.deepEqual(Object.keys(module),[{public_name:?}]);
const classify=module[{public_name:?}];
for(const [value,expected] of {cases}) assert.equal(classify(value),expected);
for(const value of [0,1,2,5,11,1000001]) assert.equal(classify(value),classify(String(value)));
for(const value of [1n,1000001n,9007199254740993n]) assert.equal(classify(value),classify(String(value)));
for(const value of ['zero','one','two','few','many','other']) assert.equal(classify(value),value);
for(const value of [NaN,Infinity,'invalid','1e8192','1c8192']) assert.throws(()=>classify(value),RangeError);
console.log('verified');
"#,
                cases = serde_json::to_string(&cases).unwrap()
            );
            assert_eq!(
                crate::test_runtime::execute_project(
                    &[crate::EcmaGeneratedFile {
                        path: "classifier.js".into(),
                        contents: code
                    }],
                    &script
                ),
                "verified",
                "{locale}/{name}"
            );
        }
    }
}
