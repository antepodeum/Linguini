//! The native bundler plan accepts a host's runtime capabilities without a framework dependency.
use linguini_codegen_ecmascript::{
    EcmaLocaleModule, EcmaProjectOptions, EcmaSource, ValidatedEcmaProject,
};
use linguini_codegen_vite::{LocaleLoading, ViteApplicationPlan, ViteEmissionOptions, VitePlan};
use linguini_ir::{lower_locale_typed, lower_schema_typed};
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};
use std::collections::BTreeMap;

#[test]
fn selected_artifacts_and_custom_runtime_bindings_work_across_output_roots() {
    let schema_source = "hello(name: String)\nunused\n";
    let locale_source = "fn Wrap(value: String) { _ => Hello {value} }\nhello = {Wrap(name)}\nunused = UNUSED_PAYLOAD_SENTINEL\n";
    let schema = lower_schema_typed(&parse_schema_in(schema_source, SourceId(1)).unwrap());
    let locales = [EcmaLocaleModule {
        locale: "en".into(),
        module: lower_locale_typed(&parse_locale_in(locale_source, SourceId(2)).unwrap()),
    }];
    let project = ValidatedEcmaProject::try_new(
        &schema,
        &locales,
        &EcmaProjectOptions {
            base_locale: Some("en".into()),
            tree_shaking: true,
            included_messages: vec!["hello".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let plan = VitePlan::new(&project).unwrap();
    assert_eq!(
        plan.message_arities(),
        BTreeMap::from([("hello".into(), 1)])
    );
    let sources = [
        EcmaSource::new(SourceId(1), "schema/main.lgs", schema_source),
        EcmaSource::new(SourceId(2), "locales/en.lgl", locale_source),
    ];
    let applications = ViteApplicationPlan {
        locale_loading: LocaleLoading::Dynamic,
        applications: BTreeMap::new(),
        runtime_helpers: BTreeMap::from([(
            "locale_state".into(),
            serde_json::json!({"import":"./runtime/locale.js", "file":"runtime/locale.js"}),
        )]),
    };
    let configured_locales = vec!["en".to_owned()];
    for output_root in ["generated", "build/client/localization"] {
        let options = ViteEmissionOptions {
            output_root,
            sources: &sources,
            configured_locales: &configured_locales,
            base_locale: "en",
            applications: Some(&applications),
        };
        let files = plan.generate_files(&options).unwrap();
        assert_eq!(plan.generate_files(&options).unwrap(), files);
        let manifest: serde_json::Value = serde_json::from_str(
            &files
                .iter()
                .find(|file| file.path == "bundler/manifest.json")
                .unwrap()
                .contents,
        )
        .unwrap();
        assert_eq!(manifest["messages"].as_object().unwrap().len(), 1);
        assert_eq!(
            manifest["runtime_helpers"]["locale_state"]["file"],
            "runtime/locale.js"
        );
        assert_eq!(manifest["locale_loading"], "dynamic");
        assert_eq!(manifest["message_semantics"].as_array().unwrap().len(), 1);
        for file in &files {
            assert!(
                !file.contents.contains("svelte"),
                "framework leaked into {}",
                file.path
            );
            if file.path.ends_with(".js") {
                assert!(!file.contents.contains("UNUSED_PAYLOAD_SENTINEL"));
            }
            if file.path.ends_with(".map") {
                let map: serde_json::Value = serde_json::from_str(&file.contents).unwrap();
                let prefix =
                    "../".repeat(output_root.split('/').count() + file.path.matches('/').count());
                assert!(map["sources"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|path| path.as_str().unwrap().starts_with(&prefix)));
                assert!(!map["mappings"].as_str().unwrap().is_empty());
            }
        }
    }
}
