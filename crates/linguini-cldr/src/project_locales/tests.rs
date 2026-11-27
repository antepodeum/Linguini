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
