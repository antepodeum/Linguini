use super::*;

mod cases {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/optional-integer.rs"
    ));
}

#[test]
fn zero_integer_is_omitted_only_when_the_selected_fraction_requires_it() {
    let mut bindings = DigitBindings::default();
    let mut plans = Vec::new();
    let mut source = crate::templates::render(
        "formatters/types",
        minijinja::context! { needs_number => true },
    );
    let mut checks = String::from("import assert from 'node:assert/strict';\nIntl.NumberFormat=class {constructor(){throw Error('host formatter');}};\n");
    for case in cases::CASES {
        let (numbers, pattern) = case.formatting();
        let plan = number_plan(&pattern, None, &numbers, None, &mut bindings);
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! {
                name => case.name, plan => plan.clone(), exported => true,
            },
        ));
        plans.push(plan);
        for (input, expected) in case.samples {
            checks.push_str(&format!(
                "assert.equal({}({}), {}, {});\n",
                case.name,
                serde_json::to_string(input).unwrap(),
                serde_json::to_string(expected).unwrap(),
                serde_json::to_string(&format!("{}: {input}", case.pattern)).unwrap()
            ));
            let numeric = if *input == "-0" {
                "-0".to_owned()
            } else {
                format!("Number({})", serde_json::to_string(input).unwrap())
            };
            checks.push_str(&format!(
                "assert.equal({}({numeric}), {});\n",
                case.name,
                serde_json::to_string(expected).unwrap()
            ));
        }
        checks.push_str(&format!(
            "for(const value of [0n,1n,1234n]) assert.equal({0}(value), {0}(String(value)));\n",
            case.name
        ));
    }
    source.push_str(&number_helpers(&plans));
    let snapshot = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-runtime/optional-integer.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &source).unwrap();
    }
    assert_eq!(source, std::fs::read_to_string(snapshot).unwrap());
    assert!(!source.contains("omit_integer_zero"));
    assert!(!source.contains("NumberPattern"));
    source.push_str(&checks);
    source.push_str(r#"
let reads=0;
assert.equal(formatOptionalInteger({toString(){reads++;return '0.5';}}), '.5');
assert.equal(reads,1);
const marker=new Error('coercion');
assert.throws(()=>formatOptionalInteger({toString(){throw marker;}}), error=>error===marker);
for(const value of [NaN,Infinity,-Infinity]) assert.equal(formatOptionalInteger(value),String(value));
const large='9'.repeat(1000);
assert.equal(formatOptionalInteger(BigInt(large)),large+'.0');
console.log('ok');
"#);
    assert_eq!(execute(&source), "ok");
}
