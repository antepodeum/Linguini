use super::*;
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

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
            let schema = lower_schema(&parse_schema_in(&schema_source, SourceId(1)).unwrap());
            let locales = [EcmaLocaleModule {
                locale: "en".into(),
                module: lower_locale(&parse_locale_in(&locale_source, SourceId(2)).unwrap()),
            }];
            let project =
                ValidatedEcmaProject::try_new(&schema, &locales, &project_options("en")).unwrap();
            let sources = [
                crate::EcmaSource::new(SourceId(1), "schema.lgs", &schema_source),
                crate::EcmaSource::new(SourceId(2), "en.lgl", &locale_source),
            ];
            let mut files = crate::generate_project_files(&project).unwrap();
            for artifact in project.semantic_artifacts().unwrap() {
                let compiled =
                    crate::compile_bundler_semantic_module(&project, &artifact, &sources).unwrap();
                files.push(EcmaGeneratedFile {
                    path: artifact.module_path,
                    contents: compiled.code,
                });
            }
            let artifact = project.message_artifacts().unwrap().remove(0);
            let physical =
                crate::compile_bundler_message_artifact_module(&project, &artifact, &sources)
                    .unwrap();
            files.push(EcmaGeneratedFile {
                path: artifact.module_path.clone(),
                contents: physical.code,
            });
            let standalone = crate::compile_message_module(
                &project,
                "en",
                "root",
                "standalone.js",
                "./shared.js",
                &sources,
            )
            .unwrap();
            files.push(EcmaGeneratedFile {
                path: "standalone.js".into(),
                contents: standalone.code,
            });
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
                physical = format!("./{}", artifact.module_path)
            );
            assert_eq!(execute_project(&files, &script), "verified", "{ty} {body}");
        }
    }
}
