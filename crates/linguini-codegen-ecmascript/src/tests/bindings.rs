use super::*;
use crate::test_runtime::root_project_files;

#[test]
fn named_functions_keep_precedence_without_shadowing_or_parameter_defaults() {
    for (declaration, ty, sample) in [
        ("enum Fruit { apple }", "Fruit", "apple"),
        (
            "type Amount = Decimal @currency(code = \"USD\")",
            "Amount",
            "1",
        ),
        (
            "type Stamp = Date @date(style = \"long\")",
            "Stamp",
            "2026-05-13",
        ),
    ] {
        for body in [
            "{helper(count)}",
            "{Forward(helper,count)}",
            "{fn(helper: helper) { _ => {helper(count)} }}",
        ] {
            let schema_source = format!("{declaration}\nroot(helper: {ty}, count: Number)\n");
            let form = if ty == "Fruit" {
                "impl Fruit { apple { label = UNUSED_FORM } }\n"
            } else {
                ""
            };
            let locale_source = format!("{form}fn helper(count: Number) {{ _ => {{count}} }}\nfn Forward(helper: {ty}, count: Number) {{ _ => {{helper(count)}} }}\nroot = {body}\n");
            let (files, physical) = root_project_files("en", &schema_source, &locale_source);
            for file in files.iter().filter(|file| file.path.ends_with(".js")) {
                for unused in ["UNUSED_FORM", "formatCurrency", "formatDate"] {
                    assert!(
                        !file.contents.contains(unused),
                        "{ty} {body}: {} contains {unused}",
                        file.path
                    );
                }
            }
            let script = format!(
                r#"
import assert from 'node:assert/strict';
const ordinary = await import('./locales/en.js');
const physical = await import({physical:?});
const standalone = await import('./standalone.js');
for (const root of [ordinary.root,physical.message,standalone.message]) {{
  assert.equal(root({sample:?},1234.5),'1234.5');
  assert.equal(root({{helper:{sample:?},count:2}}),'2');
  const seen=[];
  const unused={{[Symbol.toPrimitive]() {{ throw new Error('unused parameter coerced'); }}}};
  const count={{[Symbol.toPrimitive](hint) {{ seen.push(hint); return '1234.5'; }}}};
  const args=Object.defineProperties({{}},{{count:{{enumerable:true,get() {{seen.push('count');return count;}}}},helper:{{enumerable:true,get() {{seen.push('helper');return unused;}}}}}});
  assert.equal(root(args),'1234.5');
  assert.deepEqual(seen,['helper','count','string']);
  const failure=new Error('conversion failed');
  assert.throws(()=>root({{helper:unused,count:{{[Symbol.toPrimitive]() {{throw failure;}}}}}}),error=>error===failure);
}}
console.log('verified');
"#,
                physical = physical
            );
            assert_eq!(execute_project(&files, &script), "verified", "{ty} {body}");
        }
    }
}

#[test]
fn source_functions_cannot_shadow_generated_javascript_intrinsics() {
    let names = [
        "String",
        "Number",
        "BigInt",
        "Date",
        "Math",
        "Object",
        "Array",
        "Reflect",
        "Symbol",
        "JSON",
        "RegExp",
        "Boolean",
        "Set",
        "Map",
        "WeakMap",
        "Promise",
        "Intl",
        "Error",
        "TypeError",
        "RangeError",
        "NaN",
        "Infinity",
        "undefined",
    ];
    let schema_source = "root(value: Number, stamp: Date)\n";
    let mut locale_source = String::new();
    for name in names {
        locale_source.push_str(&format!(
            "fn {name}(value: Number) {{ _ => {name} {{value}} }}\n"
        ));
    }
    locale_source.push_str("root = ");
    for name in names {
        locale_source.push_str(&format!("{{{name}(value)}}|"));
    }
    locale_source.push_str("{value @number}|{stamp @date(skeleton = \"yw\")}\n");
    let (files, physical) = root_project_files("en", schema_source, &locale_source);
    let names = serde_json::to_string(&names).unwrap();
    let script = format!(
        r#"
import assert from 'node:assert/strict';
const ordinary = await import('./locales/en.js');
const physical = await import({physical:?});
const standalone = await import('./standalone.js');
const names = {names};
for (const root of [ordinary.root, physical.message, standalone.message]) {{
  const expected = value => names.map(name => name+' '+value).join('|')+'|'+value+'|week 1 of 2021';
  assert.equal(root(2, '2021-01-01'), expected(2));
  assert.equal(root({{value:2, stamp:new Date('2021-01-01T00:00:00Z')}}), expected(2));
  assert.equal(root({{value:2n, stamp:0}}), names.map(name=>name+' 2').join('|')+'|2|week 1 of 1970');
  const seen=[];
  const args=Object.defineProperties({{}}, {{value:{{enumerable:true,get(){{seen.push('value');return 2;}}}},stamp:{{enumerable:true,get(){{seen.push('stamp');return '2021-01-01';}}}}}});
  assert.equal(root(args), expected(2));
  assert.deepEqual(seen,['value','stamp']);
  const failure=new Error('caller coercion');
  assert.throws(()=>root({{[Symbol.toPrimitive](){{throw failure;}}}},0),error=>error===failure);
  assert.throws(()=>root(2,'invalid'),RangeError);
}}
console.log('verified');
"#
    );
    assert_eq!(execute_project(&files, &script), "verified");
}

#[test]
fn source_numeric_type_names_do_not_collide_with_standalone_helper_types() {
    let schema_source = concat!(
        "enum GeneratedDecimal { first }\n",
        "enum GeneratedNumeric { second }\n",
        "root(kind: GeneratedDecimal, measure: GeneratedNumeric, value: Number)\n",
    );
    let (files, _) = root_project_files(
        "en",
        schema_source,
        "root = {kind} {measure} {value @number}\n",
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-complete-project/type-names/numeric");
    let selected = files
        .into_iter()
        .filter(|file| {
            matches!(
                file.path.as_str(),
                "shared.js" | "shared.d.ts" | "standalone.js"
            )
        })
        .collect::<Vec<_>>();
    for file in &selected {
        let path = root.join(&file.path);
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &file.contents).unwrap();
        }
        assert_eq!(file.contents, std::fs::read_to_string(path).unwrap());
    }
    assert_eq!(
        execute_project(
            &selected,
            r#"
import assert from 'node:assert/strict';
import {message} from './standalone.js';
assert.equal(message('first','second',1234.5),'first second 1,234.5');
assert.equal(message({kind:'first',measure:'second',value:2n}),'first second 2');
assert.throws(()=>message('first','second','invalid'),RangeError);
console.log('verified');
"#
        ),
        "verified"
    );
}

#[test]
fn source_variables_cannot_shadow_runtime_intrinsics_or_lexical_parameters() {
    let source = concat!(
        "let undefined = Prefix\n",
        "fn Captured(value: Number) { _ => {value @number} }\n",
        "fn Decorated(value: Number) { _ => {undefined}:{Captured(value)} }\n",
        "root = {undefined}|{value @number}|{Decorated(value)}|{fn(undefined: value) { _ => {undefined @number} }}\n",
    );
    let (files, physical) = root_project_files("en", "root(value: Number)\n", source);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-complete-project/bindings/variables");
    for file in files.iter().filter(|file| {
        matches!(
            file.path.as_str(),
            "shared.js" | "shared.d.ts" | "standalone.js"
        )
    }) {
        let path = root.join(&file.path);
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &file.contents).unwrap();
        }
        assert_eq!(file.contents, std::fs::read_to_string(path).unwrap());
    }
    let script = format!(
        r#"
import assert from 'node:assert/strict';
const ordinary=await import('./locales/en.js');
const physical=await import({physical:?});
const standalone=await import('./standalone.js');
for(const root of [ordinary.root,physical.message,standalone.message]) {{
  for(const [value,formatted] of [[0,'0'],[2n,'2'],[-0,'-0'],[NaN,'NaN'],[Infinity,'Infinity'],[-Infinity,'-Infinity'],['1234.5','1,234.5']]) {{
    const expected='Prefix|'+formatted+'|Prefix:'+formatted+'|'+formatted;
    assert.equal(root(value),expected);
    assert.equal(root({{value}}),expected);
  }}
  const seen=[];
  const args=Object.defineProperty({{}},'value',{{enumerable:true,get(){{seen.push('value');return NaN;}}}});
  assert.equal(root(args),'Prefix|NaN|Prefix:NaN|NaN');
  assert.deepEqual(seen,['value']);
  const failure=new Error('caller conversion');
  assert.throws(()=>root({{[Symbol.toPrimitive](){{throw failure;}}}}),error=>error===failure);
}}
console.log('verified');
"#
    );
    assert_eq!(execute_project(&files, &script), "verified");
}
