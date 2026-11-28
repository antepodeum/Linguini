use std::collections::BTreeMap;

use linguini_codegen_ecmascript::{
    EcmaLocaleModule, EcmaProjectOptions, EcmaSource, ValidatedEcmaProject,
};
use linguini_codegen_vite::{ViteEmissionOptions, VitePlan};
use linguini_ir::{lower_locale_typed, lower_schema_typed};
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

#[test]
fn plural_variants_follow_message_closures_and_only_reachable_category_unions() {
    let schema_source =
        "one(value: Number)\nfew(value: Number)\ncategory(value: Number)\nliteral\n";
    let locale_source = "fn One(Plural) {\n one => One\n _ => Rest\n}\nfn Few(Plural) {\n few => Few\n _ => Rest\n}\none = {One(value)}\nfew = {Few(value)}\ncategory = {Plural(value)}\nliteral = Static\n";
    let schema = lower_schema_typed(&parse_schema_in(schema_source, SourceId(1)).unwrap());
    let locales = [EcmaLocaleModule {
        locale: "ru".into(),
        module: lower_locale_typed(&parse_locale_in(locale_source, SourceId(2)).unwrap()),
    }];
    let project = ValidatedEcmaProject::try_new(
        &schema,
        &locales,
        &EcmaProjectOptions {
            base_locale: Some("ru".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let plan = VitePlan::new(&project).unwrap();
    let sources = [
        EcmaSource::new(SourceId(1), "schema/main.lgs", schema_source),
        EcmaSource::new(SourceId(2), "locales/ru.lgl", locale_source),
    ];
    let files = plan
        .generate_files(&ViteEmissionOptions {
            output_root: "generated",
            sources: &sources,
            configured_locales: &["ru".into()],
            base_locale: "ru",
            applications: None,
        })
        .unwrap();
    let manifest: serde_json::Value = serde_json::from_str(
        &files
            .iter()
            .find(|file| file.path == "bundler/manifest.json")
            .unwrap()
            .contents,
    )
    .unwrap();
    let runtime = &manifest["plural_runtimes"]["ru"];
    assert_eq!(runtime["module"], "locales/ru/_runtime.js");
    assert_eq!(runtime["binding"], "pluralRu");
    assert_eq!(
        runtime["messages"],
        serde_json::json!({"one":["one"],"few":["few"],"category":null,"literal":[]})
    );
    let variants = runtime["variants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["categories"].to_string(),
                entry["module"].as_str().unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        variants.keys().map(String::as_str).collect::<Vec<_>>(),
        ["[\"few\",\"one\"]", "[\"few\"]", "[\"one\"]", "[]"]
    );
    for path in variants.values() {
        let code = &files
            .iter()
            .find(|file| file.path == *path)
            .unwrap()
            .contents;
        assert_eq!(code.matches("function pluralRu(").count(), 1);
        assert!(code.contains("export { pluralRu };"));
        assert!(!code.contains("return \"many\""));
        assert!(!code.contains("PluralCategoryDemand"));
    }
    assert_eq!(
        plan.generate_files(&ViteEmissionOptions {
            output_root: "generated",
            sources: &sources,
            configured_locales: &["ru".into()],
            base_locale: "ru",
            applications: None
        })
        .unwrap(),
        files
    );
}
