use linguini_cldr::{Operand, PluralRule, PluralRules, RelationOperator};

use crate::ecmascript::EcmaScriptTarget;

pub fn generate_plural_function(function_name: &str, rules: &PluralRules) -> String {
    generate_plural_function_for_target(function_name, rules, EcmaScriptTarget::TypeScript, true)
}

pub(crate) fn generate_plural_function_for_target(
    function_name: &str,
    rules: &PluralRules,
    target: EcmaScriptTarget,
    exported: bool,
) -> String {
    let mut output = String::new();
    match target {
        EcmaScriptTarget::TypeScript => output.push_str(&format!(
            "{}function {function_name}(value: number | bigint | string): string {{\n",
            if exported { "export " } else { "" }
        )),
        EcmaScriptTarget::JavaScript => output.push_str(&format!(
            "/**\n * @param {{number | bigint | string}} value\n * @returns {{string}}\n */\n{}function {function_name}(value) {{\n",
            if exported { "export " } else { "" }
        )),
    }
    output.push_str(
        "  if (value === \"zero\" || value === \"one\" || value === \"two\" || value === \"few\" || value === \"many\" || value === \"other\") return value;\n",
    );
    output.push_str("  const operands = pluralOperands(value);\n");

    for category in rules
        .categories
        .iter()
        .filter(|category| category.category != "other")
    {
        output.push_str(&format!(
            "  if ({}) return \"{}\";\n",
            rule_expression(&category.rule),
            category.category
        ));
    }

    output.push_str("  return \"other\";\n");
    output.push_str("}\n\n");
    output.push_str(match target {
        EcmaScriptTarget::TypeScript => PLURAL_OPERANDS_TYPESCRIPT_HELPER,
        EcmaScriptTarget::JavaScript => PLURAL_OPERANDS_JAVASCRIPT_HELPER,
    });
    output
}

fn rule_expression(rule: &PluralRule) -> String {
    if rule.conditions.is_empty() {
        return "true".to_owned();
    }

    rule.conditions
        .iter()
        .map(|condition| {
            condition
                .relations
                .iter()
                .map(|relation| {
                    let value = operand_expression(relation.expression.operand);
                    relation_expression(
                        &value,
                        relation.expression.modulo,
                        relation.operator,
                        &relation.ranges.ranges,
                    )
                })
                .collect::<Vec<_>>()
                .join(" && ")
        })
        .map(|condition| format!("({condition})"))
        .collect::<Vec<_>>()
        .join(" || ")
}

fn relation_expression(
    value: &str,
    modulo: Option<u64>,
    operator: RelationOperator,
    ranges: &[linguini_cldr::Range],
) -> String {
    let ranges = ranges
        .iter()
        .map(|range| format!("[{}n, {}n]", range.start, range.end))
        .collect::<Vec<_>>()
        .join(", ");
    let modulo = modulo.map_or_else(|| "undefined".to_owned(), |value| format!("{value}n"));
    let allow_fraction = matches!(
        operator,
        RelationOperator::Within | RelationOperator::NotWithin
    );
    let contains = format!("pluralOperandMatches({value}, {modulo}, {allow_fraction}, [{ranges}])");

    match operator {
        RelationOperator::Equal | RelationOperator::In | RelationOperator::Within => contains,
        RelationOperator::NotEqual | RelationOperator::NotIn | RelationOperator::NotWithin => {
            format!("!{contains}")
        }
    }
}

fn operand_expression(operand: Operand) -> String {
    match operand {
        Operand::N => "operands.n",
        Operand::I => "operands.i",
        Operand::V => "operands.v",
        Operand::W => "operands.w",
        Operand::F => "operands.f",
        Operand::T => "operands.t",
        Operand::C => "operands.c",
        Operand::E => "operands.e",
    }
    .to_owned()
}

const PLURAL_OPERANDS_TYPESCRIPT_HELPER: &str =
    include_str!("templates/plural-operands.runtime.ts");
const PLURAL_OPERANDS_JAVASCRIPT_HELPER: &str =
    include_str!("templates/plural-operands.runtime.js");

#[cfg(test)]
mod tests {
    use super::{generate_plural_function, generate_plural_function_for_target};
    use crate::ecmascript::EcmaScriptTarget;
    use linguini_cldr::built_in_plural_rules;
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, Stdio};

    #[test]
    fn generated_plural_function_snapshot_is_stable() {
        let rules = built_in_plural_rules("ru").expect("rules");
        let output = generate_plural_function("pluralRu", &rules);
        let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/codegen-ts-plural-ru.ts");
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
        let output = generate_plural_function_for_target(
            "pluralRu",
            &rules,
            EcmaScriptTarget::JavaScript,
            true,
        );
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

        let generated = generate_plural_function_for_target(
            "pluralFr",
            &rules,
            EcmaScriptTarget::JavaScript,
            true,
        );
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
