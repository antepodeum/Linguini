use super::{
    built_in_plural_rules, built_in_text_direction, compiled_currency_formatting,
    compiled_currency_fraction, compiled_date_formatting, compiled_number_formatting,
    compiled_plural_rules, CompiledPluralCategory, CompiledPluralRules, CLDR_DATA_MANIFEST_JSON,
};
use crate::PluralOperands;

fn always_matches(_: &PluralOperands) -> bool {
    true
}

fn matches_one(operands: &PluralOperands) -> bool {
    operands.i == 1 && operands.f == 0
}

#[test]
fn built_in_text_directions_are_generated_from_cldr_layout_data() {
    assert_eq!(built_in_text_direction("en"), Some("ltr"));
    assert_eq!(built_in_text_direction("ar"), Some("rtl"));
    assert_eq!(built_in_text_direction("ar-EG"), Some("rtl"));
}

#[test]
fn unknown_locale_does_not_silently_use_root_text_direction() {
    assert_eq!(built_in_text_direction("zz-ZZ"), None);
}

#[test]
fn compiled_cldr_data_falls_back_to_parent_locale_tags() {
    let portuguese = compiled_number_formatting("pt").expect("pt compiled");
    let brazil = compiled_number_formatting("pt-BR").expect("pt-BR falls back to pt");

    assert_eq!(brazil.decimal_symbol, portuguese.decimal_symbol);
    assert_eq!(brazil.group_symbol, portuguese.group_symbol);
    assert_eq!(
        compiled_plural_rules("pt-BR")
            .expect("pt-BR plural rules")
            .locale,
        "pt"
    );
    assert_eq!(
        built_in_text_direction("pt-BR"),
        built_in_text_direction("pt")
    );
}

#[test]
fn compiled_lookups_share_alias_likely_subtag_and_component_fallbacks() {
    assert_eq!(
        compiled_number_formatting("zh-TW")
            .expect("Traditional Chinese via likely script")
            .locale,
        "zh-Hant"
    );
    assert_eq!(
        compiled_number_formatting("iw-IL")
            .expect("deprecated Hebrew alias")
            .locale,
        "he"
    );
    assert_eq!(
        compiled_plural_rules("sh")
            .expect("Serbo-Croatian plural fallback")
            .locale,
        "sr"
    );
}

#[test]
fn compiled_plural_rules_need_no_runtime_json() {
    let english = compiled_plural_rules("en").expect("en compiled");
    let russian = compiled_plural_rules("ru").expect("ru compiled");

    assert_eq!(english.category_for("1").expect("en one"), "one");
    assert_eq!(english.category_for("2").expect("en other"), "other");
    assert_eq!(russian.category_for("1").expect("ru one"), "one");
    assert_eq!(russian.category_for("2").expect("ru few"), "few");
    assert_eq!(russian.category_for("5").expect("ru many"), "many");
    assert_eq!(russian.category_for("1.5").expect("ru other"), "other");
}

#[test]
fn explicit_categories_are_independent_of_other_category_order() {
    static CATEGORIES: &[CompiledPluralCategory] = &[
        CompiledPluralCategory {
            category: "other",
            matches: always_matches,
        },
        CompiledPluralCategory {
            category: "one",
            matches: matches_one,
        },
    ];
    let rules = CompiledPluralRules {
        locale: "test",
        categories: CATEGORIES,
    };

    assert_eq!(rules.category_for("1").expect("one"), "one");
    assert_eq!(rules.category_for("2").expect("other"), "other");
}

#[test]
fn compiled_plural_rules_come_from_checked_in_full_cldr_artifact() {
    let arabic = compiled_plural_rules("ar").expect("arabic generated from pinned CLDR artifact");

    assert_eq!(arabic.category_for("0").expect("ar zero"), "zero");
    assert_eq!(arabic.category_for("1").expect("ar one"), "one");
    assert_eq!(arabic.category_for("2").expect("ar two"), "two");
    assert_eq!(arabic.category_for("3").expect("ar few"), "few");
    assert_eq!(arabic.category_for("11").expect("ar many"), "many");
    assert_eq!(arabic.category_for("100").expect("ar other"), "other");
}

#[test]
fn checked_in_cldr_manifest_is_packaged_and_exposes_full_identity() {
    let manifest: serde_json::Value =
        serde_json::from_str(CLDR_DATA_MANIFEST_JSON).expect("valid CLDR manifest");

    assert_eq!(manifest["schema"], 1);
    assert_eq!(manifest["source"]["cldr_version"], "48.2.0");
    assert_eq!(
        manifest["source"]["commit"],
        "bb334e8d6250c9363e957e131bf7e6d08ec72f91"
    );
    assert_eq!(
        manifest["artifact"]["sha256"],
        "aac586640fc07211af62f9776e6a7aece371cb03e69a9026394c0e039a74c4c6"
    );
    assert_eq!(manifest["coverage"]["language_aliases"], 500);
    assert_eq!(manifest["coverage"]["parent_locales"], 199);
    assert_eq!(manifest["coverage"]["locale_rules"], 1);
    assert_eq!(manifest["coverage"]["likely_subtags"], 7_788);
    assert_eq!(manifest["coverage"]["extension_key_aliases"], 0);
    assert_eq!(manifest["coverage"]["extension_type_aliases"], 49);
    assert_eq!(manifest["coverage"]["locale_candidates"], 8_460);
    assert_eq!(manifest["coverage"]["number_locales"], 766);
    assert_eq!(manifest["coverage"]["numbering_systems"], 78);
    assert_eq!(manifest["coverage"]["currency_fraction_rules"], 75);
    assert_eq!(manifest["coverage"]["date_locales"], 765);
}

#[test]
fn built_in_plural_rule_sources_are_available_for_codegen_without_json() {
    let russian = built_in_plural_rules("ru").expect("ru built-in source rules");

    assert_eq!(russian.locale, "ru");
    assert!(russian
        .categories
        .iter()
        .any(|category| category.category == "one"));
    assert_eq!(russian.category_for("2").expect("ru few"), "few");
}

#[test]
fn compiled_formatting_data_is_typed_not_json() {
    let numbers = compiled_number_formatting("en").expect("numbers");
    let currency = compiled_currency_formatting("en").expect("currency");
    let dates = compiled_date_formatting("en").expect("dates");

    assert_eq!(numbers.decimal_symbol, ".");
    assert_eq!(numbers.numbering_system, "latn");
    assert_eq!(numbers.digits, "0123456789");
    assert_eq!(numbers.decimal_pattern.positive.primary_group_size, Some(3));
    assert_eq!(numbers.decimal_pattern.positive.max_fraction_digits, 3);
    let accounting = currency.accounting_pattern.expect("accounting pattern");
    assert_eq!(accounting.positive.prefix, "\u{a4}");
    assert_eq!(accounting.positive.min_fraction_digits, 2);
    let accounting_negative = accounting.negative.expect("negative accounting pattern");
    assert_eq!(accounting_negative.prefix, "(\u{a4}");
    assert_eq!(accounting_negative.suffix, ")");
    assert_eq!(dates.time_formats.short, "h:mm\u{202f}a");
    assert_eq!(dates.months.wide[0], "January");
    assert_eq!(dates.weekdays.abbreviated[0], "Sun");

    let persian = compiled_number_formatting("fa").expect("Persian numbers");
    assert_eq!(persian.numbering_system, "arabext");
    assert_eq!(persian.digits, "۰۱۲۳۴۵۶۷۸۹");
    assert_eq!(persian.decimal_symbol, "٫");
    assert_eq!(
        compiled_date_formatting("fa")
            .expect("Persian dates")
            .digits,
        persian.digits
    );

    let bengali = compiled_number_formatting("bn").expect("Bengali numbers");
    assert_eq!(bengali.numbering_system, "beng");
    assert_eq!(bengali.digits, "০১২৩৪৫৬৭৮৯");
}

#[test]
fn compiled_currency_fractions_apply_overrides_and_defaults() {
    let jpy = compiled_currency_fraction("jpy").expect("JPY rules");
    assert_eq!(jpy.digits, 0);
    assert_eq!(jpy.rounding, 0);

    let kwd = compiled_currency_fraction("KWD").expect("KWD rules");
    assert_eq!(kwd.digits, 3);

    let clf = compiled_currency_fraction("CLF").expect("CLF rules");
    assert_eq!(clf.digits, 4);

    let chf = compiled_currency_fraction("CHF").expect("CHF rules");
    assert_eq!(chf.digits, 2);
    assert_eq!(chf.cash_digits, 2);
    assert_eq!(chf.cash_rounding, 5);

    let default = compiled_currency_fraction("USD").expect("default USD rules");
    assert_eq!(default.digits, 2);
    assert_eq!(default.rounding, 0);
    assert_eq!(default.cash_digits, 2);
    assert_eq!(default.cash_rounding, 0);

    assert_eq!(compiled_currency_fraction("US"), None);
    assert_eq!(compiled_currency_fraction("12$"), None);
}
