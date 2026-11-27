use super::*;
use crate::project::sources::project_locale_fallbacks;
use linguini_syntax::parse_locale;

#[test]
fn module_merge_preserves_all_declaration_origins() {
    let mut target = namespaced_module(
        lower_locale(&parse_locale("section { first = First }\n").expect("first locale")),
        "shop",
    );
    let source = namespaced_module(
        lower_locale(&parse_locale("other { second = Second }\n").expect("second locale")),
        "shop",
    );

    merge_module(&mut target, source).expect("disjoint fixture merge");

    assert_eq!(
        target
            .origins()
            .iter()
            .map(|origin| origin.name.as_str())
            .collect::<Vec<_>>(),
        [
            "shop.section",
            "shop.section.first",
            "shop.other",
            "shop.other.second",
        ]
    );
    assert_eq!(
        target
            .groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.section", "shop.other"]
    );
}

#[test]
fn module_merge_rejects_duplicate_symbols_instead_of_duplicating_them() {
    let target = namespaced_module(
        lower_locale(&parse_locale("section { first = First }\n").expect("first locale")),
        "shop",
    );
    let conflicting = namespaced_module(
        lower_locale(&parse_locale("section { second = Second }\n").expect("second locale")),
        "shop",
    );

    let mut merged = target;
    let error = merge_module(&mut merged, conflicting)
        .expect_err("duplicate group symbols must fail visibly");
    assert!(error.to_string().contains("duplicate group symbol"));
    assert!(error.to_string().contains("shop.section"));
    // The rejected source contributes nothing to the target.
    assert_eq!(merged.groups().len(), 1);
    assert!(merged
        .messages()
        .iter()
        .all(|message| message.name != "shop.section.second"));
}

#[test]
fn fallback_merge_keeps_only_inserted_symbol_provenance() {
    let mut target = namespaced_module(
        lower_locale(&parse_locale("present = Primary\n").expect("primary locale")),
        "shop",
    );
    let fallback = namespaced_module(
        lower_locale(
            &parse_locale("present = Default\nextra { nested = Fallback }\n")
                .expect("fallback locale"),
        ),
        "shop",
    );

    merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

    assert_eq!(
        target
            .origins()
            .iter()
            .map(|origin| origin.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.present", "shop.extra", "shop.extra.nested"]
    );
    assert_eq!(
        target
            .messages()
            .iter()
            .map(|message| message.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.present", "shop.extra.nested"]
    );
    assert_eq!(
        target
            .groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.extra"]
    );
}

#[test]
fn fallback_merge_keeps_new_nested_groups_without_duplicate_parent_provenance() {
    let mut target = namespaced_module(
        lower_locale(&parse_locale("section { primary = Primary }\n").expect("primary")),
        "shop",
    );
    let fallback = namespaced_module(
        lower_locale(
            &parse_locale("section { primary = Default\n secondary { item = Fallback } }\n")
                .expect("fallback"),
        ),
        "shop",
    );

    merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

    assert_eq!(
        target
            .groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.section", "shop.section.secondary"]
    );
    assert_eq!(
        target
            .messages()
            .iter()
            .map(|message| message.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.section.primary", "shop.section.secondary.item"]
    );
    assert_eq!(
        target
            .origins()
            .iter()
            .map(|origin| origin.name.as_str())
            .collect::<Vec<_>>(),
        [
            "shop.section",
            "shop.section.primary",
            "shop.section.secondary",
            "shop.section.secondary.item",
        ]
    );
}

#[test]
fn fallback_merge_blocks_cross_kind_parent_collision() {
    let mut target = namespaced_module(
        lower_locale(&parse_locale("section { title = Primary }\n").expect("primary")),
        "shop",
    );
    let fallback = namespaced_module(
        lower_locale(&parse_locale("section = Fallback\n").expect("fallback")),
        "shop",
    );

    merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

    assert_eq!(target.groups().len(), 1);
    assert_eq!(target.messages().len(), 1);
    assert_eq!(target.messages()[0].name, "shop.section.title");
    assert_eq!(
        target
            .origins()
            .iter()
            .map(|origin| origin.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.section", "shop.section.title"]
    );
}

#[test]
fn fallback_merge_blocks_reverse_cross_kind_parent_collision() {
    let mut target = namespaced_module(
        lower_locale(&parse_locale("section = Primary\n").expect("primary")),
        "shop",
    );
    let fallback = namespaced_module(
        lower_locale(&parse_locale("section { child = Fallback }\n").expect("fallback")),
        "shop",
    );

    merge_module_fallback(&mut target, fallback).expect("fallback fixture merge");

    assert!(target.groups().is_empty());
    assert_eq!(
        target
            .messages()
            .iter()
            .map(|message| message.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.section"]
    );
    assert_eq!(
        target
            .origins()
            .iter()
            .map(|origin| origin.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.section"]
    );
}

#[test]
fn project_fallbacks_follow_cldr_parents_and_likely_scripts() {
    let locales = ["en", "en-001", "en-AU", "zh-Hant", "zh-TW"]
        .map(str::to_owned)
        .to_vec();

    let plan = ProjectLocalePlan::new(locales.iter().map(String::as_str));
    assert_eq!(
        project_locale_fallbacks(&plan, "en-AU", "en"),
        ["en-001", "en"]
    );
    assert_eq!(
        project_locale_fallbacks(&plan, "zh-TW", "en"),
        ["zh-Hant", "en"]
    );
}

#[test]
fn project_fallbacks_match_canonical_aliases() {
    let locales = ["en", "he", "iw-IL"].map(str::to_owned).to_vec();
    let plan = ProjectLocalePlan::new(locales.iter().map(String::as_str));

    assert_eq!(project_locale_fallbacks(&plan, "iw-IL", "en"), ["he", "en"]);
}

#[test]
fn exact_declared_spelling_excludes_its_canonical_sibling_from_fallbacks() {
    let locales = ["en", "he", "he-IL", "iw-IL"];
    let plan = ProjectLocalePlan::new(locales);
    assert_eq!(project_locale_fallbacks(&plan, "iw-IL", "en"), ["he", "en"]);
    assert_eq!(project_locale_fallbacks(&plan, "he-IL", "en"), ["he", "en"]);
}
