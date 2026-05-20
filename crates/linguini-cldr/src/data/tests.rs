use super::{
    built_in_plural_rules, built_in_text_direction, compiled_currency_formatting,
    compiled_date_formatting, compiled_number_formatting, compiled_plural_rules,
};

#[test]
fn built_in_text_directions_are_generated_from_cldr_layout_data() {
    assert_eq!(built_in_text_direction("en"), Some("ltr"));
    assert_eq!(built_in_text_direction("ar"), Some("rtl"));
    assert_eq!(built_in_text_direction("ar-EG"), Some("rtl"));
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
fn compiled_plural_rules_are_generated_from_full_cldr_at_cargo_build_time() {
    let arabic =
        compiled_plural_rules("ar").expect("arabic generated from CLDR at cargo build time");

    assert_eq!(arabic.category_for("0").expect("ar zero"), "zero");
    assert_eq!(arabic.category_for("1").expect("ar one"), "one");
    assert_eq!(arabic.category_for("2").expect("ar two"), "two");
    assert_eq!(arabic.category_for("3").expect("ar few"), "few");
    assert_eq!(arabic.category_for("11").expect("ar many"), "many");
    assert_eq!(arabic.category_for("100").expect("ar other"), "other");
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
    assert_eq!(
        currency.accounting_pattern.as_deref(),
        Some("\u{a4}#,##0.00;(\u{a4}#,##0.00)")
    );
    assert_eq!(dates.time_formats.short, "h:mm\u{202f}a");
}
