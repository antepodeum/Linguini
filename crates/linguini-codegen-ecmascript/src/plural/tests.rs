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
    assert_eq!(russian.matches("const integerValueModulo10 =").count(), 1);
    assert_eq!(russian.matches("const integerValueModulo100 =").count(), 1);
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
