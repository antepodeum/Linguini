use crate::cache::{CldrCacheError, CldrCacheResult, PLURALS_FILE};
use crate::plural::{parse_plural_rule, PluralRule};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralRules {
    pub locale: String,
    pub categories: Vec<PluralCategoryRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralCategoryRule {
    pub category: String,
    pub source: String,
    pub rule: PluralRule,
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
    use super::{load_plural_rules, load_plural_rules_from_cache};
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
            "pluralRule-count-many": "v = 0 and i % 10 = 0 or v = 0 and i % 10 = 5..9",
            "pluralRule-count-other": ""
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
}
