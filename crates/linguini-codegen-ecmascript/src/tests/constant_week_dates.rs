use super::*;
use crate::test_runtime::root_project_files;

#[test]
fn constant_week_wording_executes_without_cardinal_data_in_every_module_shape() {
    for locale in ["sa", "ccp", "aa", "la"] {
        assert!(linguini_cldr::built_in_plural_rules(locale).is_none());
        for skeleton in ["yw", "MMMMW"] {
            let source = format!("root = {{value @date(skeleton = {skeleton:?})}}\n");
            let (files, physical) = root_project_files(locale, "root(value: Date)\n", &source);
            for file in files.iter().filter(|file| file.path.ends_with(".js")) {
                for unused in [
                    "pluralOperands",
                    "pluralOperandMatches",
                    "weekNumberModulo",
                    "-count-other",
                ] {
                    assert!(
                        !file.contents.contains(unused),
                        "{locale}/{skeleton}: {} contains {unused}",
                        file.path
                    );
                }
            }
            let script = format!(
                r#"
import assert from 'node:assert/strict';
const ordinary=await import('./locales/'+{locale:?}+'.js');
const physical=await import({physical:?});
const standalone=await import('./standalone.js');
for(const root of [ordinary.root,physical.message,standalone.message]) {{
 for(const value of ['2021-01-01','2021-01-04','2021-05-30','0000-01-01',0,-1,new Date('2021-05-31T00:00:00Z')]) {{
  assert.equal(root(value),ordinary.root(value));
  assert.equal(root({{value}}),ordinary.root(value));
 }}
 assert.throws(()=>root('2024-02-30'),RangeError);
 const failure=new Error('caller getter');
 const value=new Date(0);value.getUTCDate=()=>{{throw failure}};
 assert.throws(()=>root(value),error=>error===failure);
}}
console.log('verified');
"#
            );
            assert_eq!(
                execute_project(&files, &script),
                "verified",
                "{locale}/{skeleton}"
            );
        }
    }
}
