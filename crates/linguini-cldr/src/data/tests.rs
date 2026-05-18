use super::{
    built_in_plural_rules, built_in_text_direction, compiled_currency_formatting,
    compiled_date_formatting, compiled_number_formatting, compiled_plural_rules,
    load_currency_formatting_from_cache, load_date_formatting_from_cache,
    load_number_formatting_from_cache, load_plural_rules, load_plural_rules_from_cache,
    load_text_direction_from_cache,
};
use crate::fetch_cldr_from_dir;
use linguini_test_support::{temp_project_dir, TempProject};
use std::fs;

const PLURALS: &str = r#"
{
  "supplemental": {
    "plurals-type-cardinal": {
      "en": {
        "pluralRule-count-one": "i = 1 and v = 0 @integer 1",
        "pluralRule-count-other": " @integer 0, 2~16"
      },
      "ru": {
        "pluralRule-count-one": "v = 0 and i % 10 = 1 and i % 100 != 11",
        "pluralRule-count-few": "v = 0 and i % 10 = 2..4 and i % 100 != 12..14",
        "pluralRule-count-many": "v = 0 and i % 10 = 0 or v = 0 and i % 10 = 5..9 or v = 0 and i % 100 = 11..14",
        "pluralRule-count-other": ""
      }
    }
  }
}
"#;

const NUMBERS: &str = r##"
{
  "main": {
    "en": {
      "numbers": {
        "symbols-numberSystem-latn": { "decimal": ".", "group": "," },
        "decimalFormats-numberSystem-latn": { "standard": "#,##0.###" },
        "percentFormats-numberSystem-latn": { "standard": "#,##0%" },
        "currencyFormats-numberSystem-latn": {
          "standard": "CUR#,##0.00",
          "accounting": "(CUR#,##0.00)"
        }
      }
    }
  }
}
"##;

const GREGORIAN: &str = r#"
{
  "main": {
    "en": {
      "dates": {
        "calendars": {
          "gregorian": {
            "dateFormats": {
              "full": "EEEE, MMMM d, y",
              "long": "MMMM d, y",
              "medium": "MMM d, y",
              "short": "M/d/yy"
            },
            "timeFormats": {
              "full": "h:mm:ss a zzzz",
              "long": "h:mm:ss a z",
              "medium": "h:mm:ss a",
              "short": "h:mm a"
            },
            "dateTimeFormats": {
              "full": "{1}, {0}",
              "long": "{1}, {0}",
              "medium": "{1}, {0}",
              "short": "{1}, {0}"
            }
          }
        }
      }
    }
  }
}
"#;

const LAYOUT: &str = r#"
{
  "main": {
    "en": {
      "layout": {
        "orientation": {
          "characterOrder": "left-to-right",
          "lineOrder": "top-to-bottom"
        }
      }
    }
  }
}
"#;

#[test]
fn loads_plural_rules_for_locale_from_cldr_json() {
    let rules = load_plural_rules(PLURALS, "ru").expect("plural rules");

    assert_eq!(rules.locale, "ru");
    assert_eq!(rules.categories.len(), 4);
    assert!(rules
        .categories
        .iter()
        .any(|category| category.category == "few"));
}

#[test]
fn plural_categories_match_selected_cldr_examples() {
    let english = load_plural_rules(PLURALS, "en").expect("english");
    let russian = load_plural_rules(PLURALS, "ru").expect("russian");

    assert_eq!(english.category_for("1").expect("en one"), "one");
    assert_eq!(english.category_for("2").expect("en other"), "other");
    assert_eq!(english.category_for("1.0").expect("en decimal"), "other");
    assert_eq!(russian.category_for("1").expect("ru one"), "one");
    assert_eq!(russian.category_for("2").expect("ru few"), "few");
    assert_eq!(russian.category_for("5").expect("ru many"), "many");
    assert_eq!(russian.category_for("11").expect("ru many"), "many");
    assert_eq!(russian.category_for("1.5").expect("ru fraction"), "other");
}

#[test]
fn load_plural_rules_reports_missing_locale() {
    let error = load_plural_rules(PLURALS, "ar").expect_err("missing locale");

    assert_eq!(error, "missing plural rules for locale `ar`");
}

#[test]
fn loads_plural_rules_from_cache() {
    let project = temp_project_dir("load_plural_rules_from_cache");
    let source = project
        .path()
        .join("source/cldr-json/cldr-core/supplemental");
    fs::create_dir_all(&source).expect("source dir");
    fs::write(source.join("plurals.json"), PLURALS).expect("plural data");
    let cache = project.path().join(".linguini/cache");
    fetch_cldr_from_dir(project.path().join("source"), &cache).expect("fetch");

    let rules = load_plural_rules_from_cache(&cache, "en").expect("load rules");

    assert_eq!(rules.categories.len(), 2);
}

#[test]
fn loads_number_date_and_currency_formatting_from_cache() {
    let project = write_full_cache("load_formatting_from_cache");
    let cache = project.path().join(".linguini/cache");

    let numbers = load_number_formatting_from_cache(&cache, "en").expect("numbers");
    let currency = load_currency_formatting_from_cache(&cache, "en").expect("currency");
    let dates = load_date_formatting_from_cache(&cache, "en").expect("dates");
    let direction = load_text_direction_from_cache(&cache, "en").expect("direction");

    assert_eq!(numbers.decimal_symbol, ".");
    assert_eq!(numbers.decimal_pattern, "#,##0.###");
    assert_eq!(numbers.percent_pattern, "#,##0%");
    assert_eq!(currency.standard_pattern, "CUR#,##0.00");
    assert_eq!(
        currency.accounting_pattern.as_deref(),
        Some("(CUR#,##0.00)")
    );
    assert_eq!(dates.date_formats.short, "M/d/yy");
    assert_eq!(dates.time_formats.short, "h:mm a");
    assert_eq!(dates.date_time_formats.short, "{1}, {0}");
    assert_eq!(direction, "ltr");
}

#[test]
fn built_in_text_directions_are_generated_from_cldr_layout_data() {
    assert_eq!(built_in_text_direction("en"), Some("ltr"));
    assert_eq!(built_in_text_direction("ar"), Some("rtl"));
    assert_eq!(built_in_text_direction("ar-EG"), Some("rtl"));
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
        Some("(\u{a4}#,##0.00)")
    );
    assert_eq!(dates.time_formats.short, "h:mm a");
}

fn write_full_cache(name: &str) -> TempProject {
    let project = temp_project_dir(name);
    let supplemental = project
        .path()
        .join("source/cldr-json/cldr-core/supplemental");
    let numbers = project
        .path()
        .join("source/cldr-json/cldr-numbers-full/main/en");
    let dates = project
        .path()
        .join("source/cldr-json/cldr-dates-full/main/en");
    let layout = project
        .path()
        .join("source/cldr-json/cldr-misc-full/main/en");
    fs::create_dir_all(&supplemental).expect("supplemental dir");
    fs::create_dir_all(&numbers).expect("numbers dir");
    fs::create_dir_all(&dates).expect("dates dir");
    fs::create_dir_all(&layout).expect("layout dir");
    fs::write(supplemental.join("plurals.json"), PLURALS).expect("plural data");
    fs::write(numbers.join("numbers.json"), NUMBERS).expect("numbers data");
    fs::write(dates.join("ca-gregorian.json"), GREGORIAN).expect("calendar data");
    fs::write(layout.join("layout.json"), LAYOUT).expect("layout data");
    fetch_cldr_from_dir(
        project.path().join("source"),
        project.path().join(".linguini/cache"),
    )
    .expect("fetch");
    project
}
