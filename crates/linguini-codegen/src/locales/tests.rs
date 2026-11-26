use super::ProjectLocalePlan;

#[test]
fn fallback_preserves_exact_names_alias_precedence_and_base_order() {
    let plan = ProjectLocalePlan::new([
        "en", "en-001", "en-AU", "zh-Hant", "zh-TW", "he-IL", "iw-IL", "he",
    ]);
    for (requested, base, expected) in [
        ("en-AU", Some("en"), vec!["en-AU", "en-001", "en"]),
        ("en-NZ", Some("en"), vec!["en-001", "en"]),
        ("zh-TW", Some("en"), vec!["zh-TW", "zh-Hant", "en"]),
        ("IW-il", Some("en"), vec!["iw-IL", "he", "en"]),
        ("HE-il", None, vec!["he-IL", "he"]),
        ("fr-CA", None, vec![]),
        ("fr-CA", Some("en"), vec!["en"]),
        ("not a tag", Some("en"), vec!["en"]),
    ] {
        assert_eq!(
            plan.fallback_chain(requested, base),
            expected,
            "{requested}"
        );
    }
    let alias = ProjectLocalePlan::new(["iw-IL", "he-IL", "he"]);
    assert_eq!(alias.fallback_chain("he-IL", None), ["he-IL", "he"]);
    assert_eq!(alias.fallback_chain("iw-IL", None), ["iw-IL", "he"]);
    let canonical_only = ProjectLocalePlan::new(["he-IL", "he"]);
    assert_eq!(
        canonical_only.fallback_chain("iw-IL", None),
        ["he-IL", "he"]
    );
}

#[test]
fn resolution_table_includes_pinned_parents_aliases_scripts_and_custom_tags() {
    let plan = ProjectLocalePlan::new(["en", "en-001", "zh-Hant", "he", "en-XA", "x-private"]);
    let resolutions = plan.resolutions();
    for (input, expected) in [
        ("en-au", "en-001"),
        ("en-latn", "en"),
        ("zh-tw", "zh-Hant"),
        ("iw", "he"),
        ("en-xa", "en-XA"),
        ("x-private", "x-private"),
    ] {
        assert_eq!(
            resolutions.get(input).map(String::as_str),
            Some(expected),
            "{input}"
        );
    }
    assert!(!resolutions.contains_key("fr-ca"));
    assert!(resolutions
        .keys()
        .all(|key| *key == key.to_ascii_lowercase()));
    assert_eq!(resolutions, plan.resolutions());
    assert!(ProjectLocalePlan::new([]).resolutions().is_empty());
}

#[test]
fn composition_replaces_whole_declarations_and_preserves_order_and_source_history() {
    use super::compose_locale_modules;
    use linguini_ir::{lower_locale_typed, IrSymbolKind, LocaleIr};
    use linguini_syntax::{parse_locale_in, SourceId};

    let base = lower_locale_typed(
        &parse_locale_in(
            concat!(
                "enum Choice { base }\nlet label = Base\n",
                "impl Fruit { apple { label = Base } }\n",
                "fn Ready() {\n _ => Base\n}\n",
                "/// Base section\nsection {\n title = Base\n child = Child\n}\n",
            ),
            SourceId(1),
        )
        .unwrap(),
    );
    let regional = lower_locale_typed(
        &parse_locale_in(
            concat!(
                "enum Choice { regional }\nlet label = Regional\n",
                "impl Fruit { pear { label = Regional } }\n",
                "fn Ready() {\n _ => Regional\n}\n",
                "/// Regional section\nsection { title = Regional }\nregional = Extra\n",
            ),
            SourceId(2),
        )
        .unwrap(),
    );
    let selected = lower_locale_typed(
        &parse_locale_in(
            "let label = Selected\n/// Selected section\nsection { title = Selected }\n",
            SourceId(3),
        )
        .unwrap(),
    );
    let originals = [base.clone(), regional.clone(), selected.clone()];
    let merged = compose_locale_modules([&base, &regional, &selected]);
    assert_eq!(merged.enums(), regional.enums());
    assert_eq!(merged.variables(), selected.variables());
    assert_eq!(merged.forms(), regional.forms());
    assert_eq!(
        merged.forms()[0]
            .variants
            .iter()
            .map(|variant| variant.name.as_str())
            .collect::<Vec<_>>(),
        ["pear"]
    );
    assert_eq!(merged.functions(), regional.functions());
    assert_eq!(merged.groups(), selected.groups());
    assert_eq!(
        merged
            .messages()
            .iter()
            .map(|message| message.name.as_str())
            .collect::<Vec<_>>(),
        ["section.title", "section.child", "regional"]
    );
    assert_eq!(merged.messages()[0], selected.messages()[0]);
    assert_eq!(merged.messages()[1], base.messages()[1]);
    assert_eq!(merged.messages()[2], regional.messages()[1]);
    assert!(merged.type_aliases().is_empty());
    for (kind, name, selected_override) in [
        (IrSymbolKind::Enum, "Choice", false),
        (IrSymbolKind::Variable, "label", true),
        (IrSymbolKind::Form, "Fruit", false),
        (IrSymbolKind::Function, "Ready", false),
        (IrSymbolKind::Group, "section", true),
        (IrSymbolKind::Message, "section.title", true),
    ] {
        let history = merged
            .origins()
            .iter()
            .filter(|origin| origin.kind == kind && origin.name == name)
            .map(|origin| (origin.span.source, origin.is_override))
            .collect::<Vec<_>>();
        let mut expected = vec![(SourceId(1), false), (SourceId(2), true)];
        if selected_override {
            expected.push((SourceId(3), true));
        }
        assert_eq!(history, expected, "{name}");
    }
    assert_eq!(
        merged.origins().len(),
        base.origins().len() + regional.origins().len() + selected.origins().len()
    );
    assert_eq!([base.clone(), regional, selected], originals);
    assert_eq!(compose_locale_modules([&base]), base);
    assert_eq!(compose_locale_modules([]), LocaleIr::default());
}
