use crate::cache::{CldrCacheError, CldrCacheResult, PLURALS_FILE};
use crate::plural::{parse_plural_rule, PluralOperands, PluralRule};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralRules {
    pub locale: String,
    pub categories: Vec<PluralCategoryRule>,
}

impl PluralRules {
    pub fn category_for(&self, sample: &str) -> Result<&str, String> {
        let operands = PluralOperands::parse(sample).map_err(|error| error.to_string())?;
        Ok(self.category_for_operands(&operands))
    }

    pub fn category_for_operands(&self, operands: &PluralOperands) -> &str {
        self.categories
            .iter()
            .find(|category| category.rule.matches(operands))
            .map(|category| category.category.as_str())
            .unwrap_or("other")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralCategoryRule {
    pub category: String,
    pub source: String,
    pub rule: PluralRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormatData {
    pub locale: String,
    pub decimal_symbol: String,
    pub group_symbol: String,
    pub decimal_pattern: String,
    pub percent_pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyFormatData {
    pub locale: String,
    pub standard_pattern: String,
    pub accounting_pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFormatData {
    pub locale: String,
    pub date_formats: FormatWidths,
    pub time_formats: FormatWidths,
    pub date_time_formats: FormatWidths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatWidths {
    pub full: String,
    pub long: String,
    pub medium: String,
    pub short: String,
}

pub fn load_plural_rules_from_cache(
    cache_root: impl AsRef<Path>,
    locale: &str,
) -> CldrCacheResult<PluralRules> {
    let path = cache_root.as_ref().join(PLURALS_FILE);
    let source = fs::read_to_string(&path).map_err(|source| CldrCacheError::Io {
        path: path.clone(),
        source,
    })?;
    load_plural_rules(&source, locale).map_err(|message| CldrCacheError::Parse { path, message })
}

pub fn load_number_formatting_from_cache(
    cache_root: impl AsRef<Path>,
    locale: &str,
) -> CldrCacheResult<NumberFormatData> {
    let path = locale_data_path(cache_root.as_ref(), locale, "numbers.json");
    let source = read_data_file(&path)?;
    let numbers = find_json_object(&source, "numbers").ok_or_else(|| CldrCacheError::Parse {
        path: path.clone(),
        message: format!("missing numbers data for locale `{locale}`"),
    })?;
    let symbols = find_json_object(numbers, "symbols-numberSystem-latn").ok_or_else(|| {
        CldrCacheError::Parse {
            path: path.clone(),
            message: "missing Latin number symbols".to_owned(),
        }
    })?;
    let decimal_formats = find_json_object(numbers, "decimalFormats-numberSystem-latn")
        .ok_or_else(|| CldrCacheError::Parse {
            path: path.clone(),
            message: "missing decimal format data".to_owned(),
        })?;
    let percent_formats = find_json_object(numbers, "percentFormats-numberSystem-latn")
        .ok_or_else(|| CldrCacheError::Parse {
            path: path.clone(),
            message: "missing percent format data".to_owned(),
        })?;

    Ok(NumberFormatData {
        locale: locale.to_owned(),
        decimal_symbol: required_string(symbols, "decimal", &path)?,
        group_symbol: required_string(symbols, "group", &path)?,
        decimal_pattern: required_string(decimal_formats, "standard", &path)?,
        percent_pattern: required_string(percent_formats, "standard", &path)?,
    })
}

pub fn load_currency_formatting_from_cache(
    cache_root: impl AsRef<Path>,
    locale: &str,
) -> CldrCacheResult<CurrencyFormatData> {
    let path = locale_data_path(cache_root.as_ref(), locale, "numbers.json");
    let source = read_data_file(&path)?;
    let numbers = find_json_object(&source, "numbers").ok_or_else(|| CldrCacheError::Parse {
        path: path.clone(),
        message: format!("missing numbers data for locale `{locale}`"),
    })?;
    let currency_formats = find_json_object(numbers, "currencyFormats-numberSystem-latn")
        .ok_or_else(|| CldrCacheError::Parse {
            path: path.clone(),
            message: "missing currency format data".to_owned(),
        })?;

    Ok(CurrencyFormatData {
        locale: locale.to_owned(),
        standard_pattern: required_string(currency_formats, "standard", &path)?,
        accounting_pattern: optional_string(currency_formats, "accounting"),
    })
}

pub fn load_date_formatting_from_cache(
    cache_root: impl AsRef<Path>,
    locale: &str,
) -> CldrCacheResult<DateFormatData> {
    let path = locale_data_path(cache_root.as_ref(), locale, "ca-gregorian.json");
    let source = read_data_file(&path)?;
    let gregorian =
        find_json_object(&source, "gregorian").ok_or_else(|| CldrCacheError::Parse {
            path: path.clone(),
            message: format!("missing Gregorian calendar data for locale `{locale}`"),
        })?;

    Ok(DateFormatData {
        locale: locale.to_owned(),
        date_formats: load_widths(gregorian, "dateFormats", &path)?,
        time_formats: load_widths(gregorian, "timeFormats", &path)?,
        date_time_formats: load_widths(gregorian, "dateTimeFormats", &path)?,
    })
}

pub fn load_plural_rules(source: &str, locale: &str) -> Result<PluralRules, String> {
    let locale_object = find_json_object(source, locale)
        .ok_or_else(|| format!("missing plural rules for locale `{locale}`"))?;
    let mut categories = Vec::new();

    for (key, value) in string_pairs(locale_object) {
        let Some(category) = key.strip_prefix("pluralRule-count-") else {
            continue;
        };
        let rule = parse_plural_rule(&value).map_err(|error| error.to_string())?;
        categories.push(PluralCategoryRule {
            category: category.to_owned(),
            source: value,
            rule,
        });
    }

    if categories.is_empty() {
        return Err(format!("missing plural categories for locale `{locale}`"));
    }

    Ok(PluralRules {
        locale: locale.to_owned(),
        categories,
    })
}

fn load_widths(source: &str, key: &str, path: &Path) -> CldrCacheResult<FormatWidths> {
    let object = find_json_object(source, key).ok_or_else(|| CldrCacheError::Parse {
        path: path.to_path_buf(),
        message: format!("missing `{key}`"),
    })?;

    Ok(FormatWidths {
        full: required_string(object, "full", path)?,
        long: required_string(object, "long", path)?,
        medium: required_string(object, "medium", path)?,
        short: required_string(object, "short", path)?,
    })
}

fn locale_data_path(cache_root: &Path, locale: &str, file_name: &str) -> std::path::PathBuf {
    cache_root
        .join("data/common/main")
        .join(locale)
        .join(file_name)
}

fn read_data_file(path: &Path) -> CldrCacheResult<String> {
    fs::read_to_string(path).map_err(|source| CldrCacheError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn required_string(source: &str, key: &str, path: &Path) -> CldrCacheResult<String> {
    optional_string(source, key).ok_or_else(|| CldrCacheError::Parse {
        path: path.to_path_buf(),
        message: format!("missing string field `{key}`"),
    })
}

fn optional_string(source: &str, key: &str) -> Option<String> {
    string_pairs(source)
        .into_iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value)
}

fn find_json_object<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let quoted_key = format!("\"{key}\"");
    let key_start = source.find(&quoted_key)?;
    let after_key = &source[key_start + quoted_key.len()..];
    let object_start = after_key.find('{')? + key_start + quoted_key.len();
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for index in object_start..source.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[object_start + 1..index]);
                }
            }
            _ => {}
        }
    }

    None
}

fn string_pairs(source: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut index = 0;

    while let Some((key, after_key)) = read_json_string(source, index) {
        let Some(colon) = source[after_key..].find(':') else {
            break;
        };
        let value_start = after_key + colon + 1;
        let Some((value, after_value)) = read_json_string(source, value_start) else {
            index = value_start;
            continue;
        };
        pairs.push((key, value));
        index = after_value;
    }

    pairs
}

fn read_json_string(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut index = start;
    while index < bytes.len() && bytes[index] != b'"' {
        index += 1;
    }
    if index == bytes.len() {
        return None;
    }

    index += 1;
    let mut value = String::new();
    let mut escaped = false;
    while index < bytes.len() {
        let character = source[index..].chars().next()?;
        index += character.len_utf8();

        if escaped {
            value.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some((value, index));
        } else {
            value.push(character);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        load_currency_formatting_from_cache, load_date_formatting_from_cache,
        load_number_formatting_from_cache, load_plural_rules, load_plural_rules_from_cache,
    };
    use crate::fetch_cldr_from_dir;
    use linguini_test_support::temp_project_dir;
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
            "symbols-numberSystem-latn": {
              "decimal": ".",
              "group": ","
            },
            "decimalFormats-numberSystem-latn": {
              "standard": "#,##0.###"
            },
            "percentFormats-numberSystem-latn": {
              "standard": "#,##0%"
            },
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
        let source = project.path().join("source/common/supplemental");
        fs::create_dir_all(&source).expect("source dir");
        fs::write(source.join("plurals.json"), PLURALS).expect("plural data");
        let cache = project.path().join(".linguini/cache");
        fetch_cldr_from_dir(project.path().join("source"), &cache).expect("fetch");

        let rules = load_plural_rules_from_cache(&cache, "en").expect("load rules");

        assert_eq!(rules.categories.len(), 2);
    }

    #[test]
    fn loads_number_date_and_currency_formatting_from_cache() {
        let project = temp_project_dir("load_formatting_from_cache");
        let supplemental = project.path().join("source/common/supplemental");
        let main = project.path().join("source/common/main/en");
        fs::create_dir_all(&supplemental).expect("supplemental dir");
        fs::create_dir_all(&main).expect("main dir");
        fs::write(supplemental.join("plurals.json"), PLURALS).expect("plural data");
        fs::write(main.join("numbers.json"), NUMBERS).expect("numbers data");
        fs::write(main.join("ca-gregorian.json"), GREGORIAN).expect("calendar data");
        let cache = project.path().join(".linguini/cache");
        fetch_cldr_from_dir(project.path().join("source"), &cache).expect("fetch");

        let numbers = load_number_formatting_from_cache(&cache, "en").expect("numbers");
        let currency = load_currency_formatting_from_cache(&cache, "en").expect("currency");
        let dates = load_date_formatting_from_cache(&cache, "en").expect("dates");

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
    }
}
