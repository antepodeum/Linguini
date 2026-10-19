//! Build-time CLDR plural-rule lowering for canonical runtime templates.

use linguini_cldr::{Operand, PluralRules, RelationOperator};
use minijinja::context;

pub fn generate_plural_function(function_name: &str, rules: &PluralRules) -> String {
    generate_plural_function_with_visibility(function_name, rules, true)
}

pub(crate) fn generate_plural_function_with_visibility(
    function_name: &str,
    rules: &PluralRules,
    exported: bool,
) -> String {
    let categories: Vec<_> = rules.categories.iter().filter(|entry| entry.category != "other").map(|entry| {
        let conditions: Vec<Vec<_>> = entry.rule.conditions.iter().map(|condition| {
            condition.relations.iter().map(|relation| {
                let ranges: Vec<_> = relation.ranges.ranges.iter().map(|range| context! { start => range.start, end => range.end }).collect();
                context! {
                    operand => operand_name(relation.expression.operand),
                    modulo => relation.expression.modulo,
                    allow_fraction => matches!(relation.operator, RelationOperator::Within | RelationOperator::NotWithin),
                    negated => matches!(relation.operator, RelationOperator::NotEqual | RelationOperator::NotIn | RelationOperator::NotWithin),
                    ranges,
                }
            }).collect()
        }).collect();
        context! { literal => crate::ecmascript::string_literal(&entry.category), conditions }
    }).collect();
    crate::templates::render(
        "plural",
        context! { name => function_name, exported, categories },
    )
}

fn operand_name(operand: Operand) -> &'static str {
    match operand {
        Operand::N => "n",
        Operand::I => "i",
        Operand::V => "v",
        Operand::W => "w",
        Operand::F => "f",
        Operand::T => "t",
        Operand::C => "c",
        Operand::E => "e",
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_plural_function, generate_plural_function_with_visibility};
    use linguini_cldr::built_in_plural_rules;
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, Stdio};

    #[test]
    fn generated_plural_function_snapshot_is_stable() {
        let rules = built_in_plural_rules("ru").expect("rules");
        let output = generate_plural_function("pluralRu", &rules);
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
    fn generated_javascript_plural_function_snapshot_is_stable() {
        let rules = built_in_plural_rules("ru").expect("rules");
        let output = generate_plural_function_with_visibility("pluralRu", &rules, true);
        assert_eq!(output, generate_plural_function("pluralRu", &rules));
        let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/codegen-js-plural-ru.js");
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&snapshot, &output).expect("write JavaScript plural snapshot");
        }

        assert_eq!(
            output,
            std::fs::read_to_string(snapshot).expect("read JavaScript plural snapshot")
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
  const actual = pluralOperands(value);
  const expected = pluralOperands(expanded);
  for (const operand of ["n", "i", "v", "w", "f", "t"]) {{
    if (
      actual[operand].integer !== expected[operand].integer ||
      actual[operand].hasFraction !== expected[operand].hasFraction
    ) {{
      throw new Error(`${{value}}: native scientific ${{operand}} operand differs from ${{expanded}}`);
    }}
  }}
  if (actual.c.integer !== 0n || actual.e.integer !== 0n) {{
    throw new Error(`${{value}}: native scientific notation leaked into c/e operands`);
  }}
  if (pluralFr(value) !== pluralFr(expanded)) {{
    throw new Error(`${{value}}: native scientific category differs from ${{expanded}}`);
  }}
}}
if (pluralOperands(1000000000000000000000n).i.integer !== 1000000000000000000000n) {{
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
}
