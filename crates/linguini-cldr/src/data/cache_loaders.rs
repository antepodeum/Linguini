use super::json::{
    find_json_object, optional_string, read_data_file, required_string, string_pairs,
};
use super::{
    CurrencyFormatData, DateFormatData, FormatWidths, NumberFormatData, PluralCategoryRule,
    PluralRules,
};
use crate::cache::{CldrCacheError, CldrCacheResult, PLURALS_FILE};
use crate::plural::parse_plural_rule;
use std::fs;
use std::path::{Path, PathBuf};

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

pub(super) fn locale_data_path(cache_root: &Path, locale: &str, file_name: &str) -> PathBuf {
    let domain = match file_name {
        "numbers.json" => "cldr-numbers-full",
        "ca-gregorian.json" => "cldr-dates-full",
        _ => "cldr-core",
    };
    cache_root
        .join("data/cldr-json")
        .join(domain)
        .join("main")
        .join(locale)
        .join(file_name)
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
