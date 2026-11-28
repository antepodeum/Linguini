use super::*;
use crate::test_runtime::root_project_artifacts;
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

#[test]
fn numeric_styles_share_canonical_requests_in_every_module_shape() {
    let schema_source = concat!(
        "type Ratio = Decimal @number(style = \"percent\")\n",
        "type Scientific = Number @number(style = \"scientific\")\n",
        "type Unused = Date @date(style = \"full\")\n",
        "root(value: Ratio, measure: Scientific, amount: Decimal)\n"
    );
    let locale_source =
        "root = {value}|{measure}|{amount @number}|{amount @number(style = \"decimal\")}\n";
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
            "formatNumber",
            "formatNumber_style_percent",
            "formatNumber_style_scientific",
            "scientificGeneratedDecimal",
            "parseGeneratedDecimal",
            "scaleBy100",
        ] {
            assert_eq!(
                source.matches(&format!("function {name}(")).count(),
                1,
                "{path}/{name}"
            );
        }
        assert!(!source.contains("formatNumber_style_decimal"));
        assert!(!source.contains("coerceDate"));
    }
    let script = format!(
        r#"
import assert from 'node:assert/strict';
const ordinary=await import('./locales/en.js');
const physical=await import({physical:?});
const standalone=await import('./standalone.js');
Intl.NumberFormat=class{{constructor(){{throw Error('no host formatter');}}}};
for(const root of [ordinary.root,physical.message,standalone.message]) {{
 assert.equal(root('0.005','1234','1234.5'),'1%|1.234E3|1,234.5|1,234.5');
 assert.equal(root({{value:'0.005',measure:'1234',amount:'1234.5'}}),'1%|1.234E3|1,234.5|1,234.5');
 assert.equal(root(-0,-0,-0),'-0%|-0E0|-0|-0');
 let reads=0;
 assert.equal(root({{toString(){{reads++;return '0.005'}}}},0,0),'1%|0E0|0|0');
 assert.equal(reads,1);
 assert.throws(()=>root('invalid',0,0),RangeError);
}}
console.log('ok');
"#
    );
    assert_eq!(execute_project(&files, &script), "ok");
}
