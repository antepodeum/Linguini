use super::*;

#[test]
fn native_plural_input_boundaries_match_generated_intrinsics_functions_and_forms() {
    let locales = ["en", "ru", "fr", "ar", "ja"];
    let schema_source = concat!(
        "enum Item { item }\n",
        "sample0(count: Number)\nsample1(count: Number)\n",
        "sample2(item: Item, count: Number)\nsample3(item: Item, count: Number)\n",
        "sample4(count: Number)\n",
    );
    let choices =
        "zero => zero\none => one\ntwo => two\nfew => few\nmany => many\nother => other\n";
    let source = format!(
        "fn Choice(Plural) {{\n{choices}}}\n\
         fn Forward(value: Number) {{ _ => {{Choice(value)}} }}\n\
         impl Item {{ item {{\n{choices}\nform nom(Plural) {{\n{choices}}}\n}} }}\n\
         sample0 = {{Plural(count)}}\nsample1 = {{Forward(count)}}\n\
         sample2 = {{item(count)}}\nsample3 = {{item.nom(count)}}\n\
         sample4 = {{fn(Plural(count)) {{\n{choices}}}}}\n"
    );
    let schema = crate::project::ir::namespaced_module(
        lower_schema(&parse_schema(schema_source).unwrap()),
        "main",
    );
    let module = crate::project::ir::namespaced_module(
        lower_locale(&parse_locale(&source).unwrap()),
        "main",
    );
    // ECMA-262 WhiteSpace and LineTerminator, including BOM and excluding NEL.
    let whitespace = [
        '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}', '\u{00a0}',
        '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
        '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}',
        '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
    ];
    let mut inputs = Vec::new();
    for value in ["1", "1.0", "+1.20c3"] {
        inputs.push((value.to_owned(), Some(value)));
        for character in whitespace {
            for value_with_space in [
                format!("{character}{value}"),
                format!("{value}{character}"),
                format!("{character}{value}{character}"),
            ] {
                inputs.push((value_with_space, Some(value)));
            }
        }
    }
    for character in whitespace.into_iter().chain([
        '\u{0000}', '\u{001c}', '\u{0085}', '\u{180e}', '\u{200b}', '\u{2060}',
    ]) {
        inputs.push((format!("1{character}0"), None));
    }
    for character in [
        '\u{0000}', '\u{001c}', '\u{0085}', '\u{180e}', '\u{200b}', '\u{2060}',
    ] {
        for value in [
            format!("{character}1"),
            format!("1{character}"),
            format!("{character}1{character}"),
        ] {
            inputs.push((value, None));
        }
    }
    // Category passthrough happens before trimming; surrounding space cannot turn text into a number.
    for category in ["zero", "one", "two", "few", "many", "other"] {
        inputs.push((category.into(), Some(category)));
        inputs.push((format!(" {category} "), None));
        inputs.push((format!("\u{feff}{category}\u{feff}"), None));
    }
    let mut cases = Vec::new();
    for locale in locales {
        let renderer = renderer(&schema, &module, locale);
        let rules = built_in_plural_rules(locale).unwrap();
        for (value, canonical) in &inputs {
            let expected = canonical.map(|value| {
                if matches!(value, "zero" | "one" | "two" | "few" | "many" | "other") {
                    value.to_owned()
                } else {
                    rules.category_for(value).unwrap().to_owned()
                }
            });
            for index in 0..5 {
                let mut values =
                    BTreeMap::from([("count".into(), SampleValue::String(value.clone()))]);
                let args = if matches!(index, 2 | 3) {
                    values.insert("item".into(), SampleValue::String("item".into()));
                    serde_json::json!(["item", value])
                } else {
                    serde_json::json!([value])
                };
                let actual = renderer.render_message(&format!("main.sample{index}"), &values);
                if let Some(expected) = &expected {
                    assert_eq!(actual.unwrap(), *expected, "{locale}/{index}: {value:?}");
                } else {
                    assert!(
                        matches!(actual, Err(RenderError::InvalidNumber { .. })),
                        "{locale}/{index}: {value:?}: {actual:?}"
                    );
                }
                cases.push(serde_json::json!({
                    "locale":locale, "index":index, "args":args,
                    "expected":expected, "invalid":expected.is_none(),
                }));
            }
        }
    }
    super::super::parity::assert_generated_samples(schema_source, &source, &locales, &cases);
}
