use super::*;
use crate::test_runtime::root_project_artifacts;
use linguini_ir::{IrModuleBuilder, SchemaIr};
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

#[test]
fn source_date_types_preserve_native_date_inputs_in_every_module_shape() {
    let schema_source =
        "enum Date { entry }\ntype Input = Date\nroot(value: Input, direct: Date)\n";
    let locale_source = "fn Forward(value: Input, direct: Date) { _ => {value @date(style = \"short\")}|{direct @date(skeleton = \"yw\")} }\nroot = {Forward(value,direct)}\n";
    let original = lower_schema(&parse_schema_in(schema_source, SourceId(1)).unwrap());
    // The public SDK also accepts this source name. It must not capture the globalThis
    // namespace used to distinguish native Date from the source enum's public type.
    let mut namespace = original.enums()[0].clone();
    namespace.name = "globalThis".into();
    namespace.variants = vec!["namespace".into()];
    let schema = SchemaIr::try_from_module(
        IrModuleBuilder::new()
            .push_enum(original.enums()[0].clone())
            .push_enum(namespace)
            .push_type_alias(original.type_aliases()[0].clone())
            .push_message(original.messages()[0].clone())
            .build()
            .unwrap(),
    )
    .unwrap();
    let locales = [EcmaLocaleModule {
        locale: "en".into(),
        module: lower_locale(&parse_locale_in(locale_source, SourceId(2)).unwrap()),
    }];
    let project = ValidatedEcmaProject::try_new(&schema, &locales, &project_options("en")).unwrap();
    let sources = [
        crate::EcmaSource::new(SourceId(1), "schema.lgs", schema_source),
        crate::EcmaSource::new(SourceId(2), "en.lgl", locale_source),
    ];
    let (files, physical) = root_project_artifacts("en", &project, &sources);
    for file in files.iter().filter(|file| {
        file.path.starts_with("shared.")
            || file.path.starts_with("messages.")
            || file.path.starts_with("locales/")
            || file.path.starts_with("bundler/")
            || file.path == "standalone.js"
    }) {
        assert_snapshot(
            &format!(
                "tests/fixtures/golden/snapshots/js-complete-project/type-names/date-input/{}",
                file.path
            ),
            &file.contents,
        );
    }
    let script = format!(
        r#"
import assert from 'node:assert/strict';
const ordinary=await import('./locales/en.js');
const physical=await import({physical:?});
const standalone=await import('./standalone.js');
const expected='1/2/26|week 1 of 2026';
for(const root of [ordinary.root,physical.message,standalone.message]) {{
 for(const input of [new Date('2026-01-02T00:00:00Z'),'2026-01-02',Date.UTC(2026,0,2)]) {{
  assert.equal(root(input,input),expected);
  assert.equal(root({{value:input,direct:input}}),expected);
 }}
 const reads=[],date=new Date('2026-01-02T00:00:00Z');
 let timeReads=0;
 date.getTime=()=>{{timeReads++;return Date.UTC(2026,0,2)}};
 assert.equal(root({{get value(){{reads.push('value');return date}},get direct(){{reads.push('direct');return date}}}}),expected);
 assert.deepEqual(reads,['value','direct']);assert.equal(timeReads,2);
 const failure=new Error('date getter failed');
 date.getTime=()=>{{throw failure}};
 assert.throws(()=>root(date,date),error=>error===failure);
 for(const invalid of [new Date(NaN),Infinity,'invalid-date']) assert.throws(()=>root(invalid,0),RangeError);
}}
console.log('verified');
"#
    );
    assert_eq!(execute_project(&files, &script), "verified");
}
