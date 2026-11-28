use super::*;

#[test]
fn currency_code_display_preserves_icu_results_and_source_call_paths() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-display.json"
    )))
    .unwrap();
    let records = corpus["cases"].as_array().unwrap();
    let empty = IrModule::default();
    for case in records {
        let locale = case["locale"].as_str().unwrap();
        let renderer = Renderer::new(&empty, &empty, locale);
        let arguments = [
            IrFormatterArgument {
                name: "code".into(),
                value: case["code"].as_str().unwrap().into(),
            },
            IrFormatterArgument {
                name: "accounting".into(),
                value: case["accounting"].to_string(),
            },
            IrFormatterArgument {
                name: "display".into(),
                value: "code".into(),
            },
        ];
        for sample in case["samples"].as_array().unwrap() {
            assert_eq!(
                renderer
                    .format_currency(sample["input"].as_str().unwrap(), &arguments)
                    .unwrap(),
                sample["expected"].as_str().unwrap(),
                "{case}"
            );
        }
    }
    let locales = records
        .iter()
        .map(|case| case["locale"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    for accounting in [false, true] {
        let schema_source=concat!(
            "type Money = Decimal @currency(code = \"USD\", display = \"code\", accounting = \"$ACCOUNTING\")\ntype Amount = Money\n",
            "enum Mode { receipt }\n",
            "sample0(value: Amount)\nsample1(value: Decimal)\nsample2(value: Decimal)\nsample3(value: Decimal)\n",
            "sample4(value: Decimal, mode: Mode)\nsample5(value: Money)\nsample6(value: Money)\nsample7(value: Decimal)\n"
        ).replace("$ACCOUNTING",if accounting {"true"}else{"false"});
        let locale_source=concat!(
            "fn Echo(value: String) { _ => {value} }\n",
            "impl Mode { receipt { form label(value: Decimal) { _ => {value @currency(code = \"USD\", display = \"code\", accounting = \"$ACCOUNTING\")} } } }\n",
            "sample0 = {value}\nsample1 = {value @currency(code = \"USD\", display = \"code\", accounting = \"$ACCOUNTING\")}\n",
            "sample2 = {Echo(value @currency(code = \"USD\", display = \"code\", accounting = \"$ACCOUNTING\"))}\n",
            "sample3 = {fn(copy: value @currency(code = \"USD\", display = \"code\", accounting = \"$ACCOUNTING\")) { _ => {copy} }}\n",
            "sample4 = {mode.label(value)}\nsample5 = {fn(copy: value) { _ => {copy} }}\n",
            "sample6 = {value @currency(code = \"USD\", display = \"symbol\", accounting = \"$ACCOUNTING\")}\n",
            "sample7 = {value @currency(code = \"USD\", accounting = \"$ACCOUNTING\")}\n"
        ).replace("$ACCOUNTING",if accounting {"true"}else{"false"});
        let schema = lower_schema(&parse_schema(&schema_source).unwrap());
        let module = lower_locale(&parse_locale(&locale_source).unwrap());
        let mut cases = Vec::new();
        for case in records
            .iter()
            .filter(|case| case["code"] == "USD" && case["accounting"] == accounting)
        {
            let locale = case["locale"].as_str().unwrap();
            let renderer = Renderer::new(&schema, &module, locale);
            let symbol = [
                IrFormatterArgument {
                    name: "code".into(),
                    value: "USD".into(),
                },
                IrFormatterArgument {
                    name: "accounting".into(),
                    value: accounting.to_string(),
                },
            ];
            for sample in case["samples"].as_array().unwrap() {
                let input = sample["input"].as_str().unwrap();
                for index in 0..8 {
                    let expected = if index < 6 {
                        sample["expected"].as_str().unwrap().to_owned()
                    } else {
                        renderer.format_currency(input, &symbol).unwrap()
                    };
                    let mut args =
                        BTreeMap::from([("value".into(), SampleValue::String(input.into()))]);
                    if index == 4 {
                        args.insert("mode".into(), SampleValue::String("receipt".into()));
                    }
                    assert_eq!(
                        renderer
                            .render_message(&format!("sample{index}"), &args)
                            .unwrap(),
                        expected,
                        "{locale}/{accounting}/{index}/{input}"
                    );
                    cases.push(serde_json::json!({"locale":locale,"index":index,"args":if index==4{serde_json::json!([input,"receipt"])}else{serde_json::json!([input])},"expected":expected}));
                }
            }
        }
        parity::assert_generated_samples(&schema_source, &locale_source, &locales, &cases);
    }
}
