use super::*;

#[test]
fn percent_styles_preserve_pinned_results_through_every_source_call_path() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/percent-styles.json"
    )))
    .unwrap();
    let schema_source = concat!(
        "type Ratio = Decimal @number(style = \"percent\")\ntype Rate = Ratio\n",
        "enum Mode { receipt }\n",
        "sample0(value: Rate)\nsample1(value: Decimal)\nsample2(value: Decimal)\n",
        "sample3(value: Decimal)\nsample4(value: Decimal, mode: Mode)\nsample5(value: Ratio)\n",
        "sample6(value: Ratio)\nsample7(value: Decimal)\n"
    );
    let locale_source = concat!(
        "fn Echo(value: String) { _ => {value} }\n",
        "impl Mode { receipt { form label(value: Decimal) { _ => {value @number(style = \"percent\")} } } }\n",
        "sample0 = {value}\nsample1 = {value @number(style = \"percent\")}\n",
        "sample2 = {Echo(value @number(style = \"percent\"))}\n",
        "sample3 = {fn(copy: value @number(style = \"percent\")) { _ => {copy} }}\n",
        "sample4 = {mode.label(value)}\nsample5 = {fn(copy: value) { _ => {copy} }}\n",
        "sample6 = {value @number(style = \"decimal\")}\nsample7 = {value}\n"
    );
    let schema = lower_schema(&parse_schema(schema_source).unwrap());
    let module = lower_locale(&parse_locale(locale_source).unwrap());
    let mut cases = Vec::new();
    let locales = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["locale"].as_str().unwrap())
        .collect::<Vec<_>>();
    for case in corpus["cases"].as_array().unwrap() {
        let locale = case["locale"].as_str().unwrap();
        let renderer = Renderer::new(&schema, &module, locale);
        for sample in case["samples"].as_array().unwrap() {
            let input = sample["input"].as_str().unwrap();
            for index in 0..8 {
                let mut args =
                    BTreeMap::from([("value".into(), SampleValue::String(input.into()))]);
                if index == 4 {
                    args.insert("mode".into(), SampleValue::String("receipt".into()));
                }
                let expected = if index < 6 {
                    sample["expected"].as_str().unwrap().to_owned()
                } else {
                    renderer.format_number(input, &[]).unwrap()
                };
                assert_eq!(
                    renderer
                        .render_message(&format!("sample{index}"), &args)
                        .unwrap(),
                    expected,
                    "{locale}/{index}/{input}"
                );
                cases.push(serde_json::json!({"locale":locale,"index":index,"args":if index==4{serde_json::json!([input,"receipt"])}else{serde_json::json!([input])},"expected":expected}));
            }
        }
    }
    parity::assert_generated_samples(schema_source, locale_source, &locales, &cases);
}
