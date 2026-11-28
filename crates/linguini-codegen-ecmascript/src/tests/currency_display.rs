use super::*;
use crate::test_runtime::root_project_artifacts;
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

#[test]
fn currency_displays_share_default_symbol_requests_in_every_module_shape() {
    let schema_source="type Money = Decimal @currency(code = \"USD\", display = \"code\")\nroot(value: Money, amount: Decimal)\n";
    let locale_source="root = {value}|{amount @currency(code = \"USD\")}|{amount @currency(code = \"USD\", display = \"symbol\")}\n";
    let schema = lower_schema(&parse_schema_in(schema_source, SourceId(1)).unwrap());
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
    for path in ["locales/en/_runtime.js", "standalone.js"] {
        let source = &files
            .iter()
            .find(|file| file.path == path)
            .unwrap()
            .contents;
        for name in [
            "formatCurrency_code_USD",
            "formatCurrency_code_USD_display_code",
            "parseGeneratedDecimal",
            "roundTo2FractionDigits",
        ] {
            assert_eq!(
                source.matches(&format!("function {name}(")).count(),
                1,
                "{path}/{name}"
            );
        }
        for absent in [
            "display_symbol",
            "coerceDate",
            "scaleBy",
            "scientificGeneratedDecimal",
        ] {
            assert!(!source.contains(absent), "{path}/{absent}");
        }
    }
    let script = format!(
        r#"
import assert from 'node:assert/strict';
const ordinary=await import('./locales/en.js');
const physical=await import({physical:?});
const standalone=await import('./standalone.js');
Intl.NumberFormat=class{{constructor(){{throw Error('no host formatter');}}}};
for(const root of [ordinary.root,physical.message,standalone.message]) {{
 assert.equal(root('1.005','1.005'),'USD\u00a01.01|$1.01|$1.01');
 assert.equal(root({{value:'1.005',amount:'1.005'}}),'USD\u00a01.01|$1.01|$1.01');
 assert.equal(root(-0,-0),'-USD\u00a00.00|-$0.00|-$0.00');
 assert.equal(root(2n,2n),'USD\u00a02.00|$2.00|$2.00');
 let reads=0;
 assert.equal(root({{toString(){{reads++;return '1.005'}}}},0),'USD\u00a01.01|$0.00|$0.00');
 assert.equal(reads,1);
 assert.throws(()=>root('invalid',0),RangeError);
}}
console.log('ok');
"#
    );
    assert_eq!(execute_project(&files, &script), "ok");
}
