use super::*;
use crate::test_runtime::root_project_files;

#[test]
fn standalone_plural_closures_share_demand_and_preserve_full_module_observations() {
    for locale in ["ru", "en", "ja"] {
        for (body, root, reduced) in [
            ("fn A(Plural) {\n one => One\n _ => Rest\n}\n", "{A(value)}", true),
            ("fn A(Plural) {\n one => One\n _ => Rest\n}\nfn B(Plural) {\n few => Few\n _ => Rest\n}\n", "{A(value)}|{B(value)}", true),
            ("fn A(Plural) {\n one => One\n other => Other\n _ => Default\n}\n", "{A(value)}", false),
            ("fn A(Plural) {\n one => One\n _ => Rest\n}\n", "{A(value)}|{Plural(value)}", false),
        ] {
            let (files, physical) = root_project_files(locale, "root(value: Number)\n", &format!("{body}root = {root}\n"));
            let standalone = &files.iter().find(|file| file.path == "standalone.js").unwrap().contents;
            let classifier = format!("function plural{}(", locale[..1].to_uppercase() + &locale[1..]);
            assert_eq!(standalone.matches(&classifier).count(), 1);
            if reduced && locale == "ru" { assert!(!standalone.contains("return \"many\"")); }
            let script = format!(r#"
import assert from 'node:assert/strict';
const ordinary=(await import('./locales/{locale}.js')).root;
const physical=(await import({physical:?})).message;
const standalone=(await import('./standalone.js')).message;
const observe=(fn,value)=>{{try{{return ['value',fn(value)]}}catch(error){{return [error.constructor.name,error.message]}}}};
const inputs=[...Array.from({{length:401}},(_,i)=>i-200),-0,0.5,Number.MIN_VALUE,Number.MAX_VALUE,Number.MAX_SAFE_INTEGER,
 '0.0','1.0','1.00','1.2','2.5','1c6','1.2c6','-100000000000000000000001','1'.repeat(8192),'1'.repeat(8193),
 100000000000000000000001n,NaN,Infinity,undefined,null,true,'bad','one','few','many','other','zero','two'];
for(const value of inputs){{const expected=observe(ordinary,value);assert.deepEqual(observe(physical,value),expected);assert.deepEqual(observe(standalone,value),expected);}}
for(const fn of [ordinary,physical,standalone]){{
 let reads=0;const value={{toString(){{reads++;return '21'}}}};
 assert.equal(fn(value),ordinary('21'));assert.equal(reads,{calls});
 const failure=new Error('coercion');assert.throws(()=>fn({{toString(){{throw failure}}}}),error=>error===failure);
}}
console.log('ok');
"#, calls = if root.contains('|') {2} else {1});
            assert_eq!(execute_project(&files, &script), "ok", "{locale}/{body}/{root}");
        }
    }
}
