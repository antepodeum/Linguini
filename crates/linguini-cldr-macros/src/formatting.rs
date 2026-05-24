use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde_json::Value;
use std::fs;
use std::path::Path;
use syn::LitStr;

pub(crate) fn generate_text_direction_table(layout_main: &Path) -> Result<TokenStream, String> {
    let mut directions = Vec::new();
    let entries =
        fs::read_dir(layout_main).map_err(|error| format!("{}: {error}", layout_main.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", layout_main.display()))?;
        if !entry.path().is_dir() {
            continue;
        }
        let locale = entry.file_name().to_string_lossy().into_owned();
        let layout = entry.path().join("layout.json");
        if !layout.is_file() {
            continue;
        }
        let source = fs::read_to_string(&layout)
            .map_err(|error| format!("{}: {error}", layout.display()))?;
        if let Some(direction) = extract_text_direction(&source) {
            directions.push((locale, direction));
        }
    }
    directions.sort_by(|left, right| left.0.cmp(&right.0));

    let arms = directions
        .iter()
        .map(|(locale, direction)| quote! { #locale => Some(#direction), });

    Ok(quote! {
        fn generated_text_direction(locale: &str) -> Option<&'static str> {
            match locale {
                #(#arms)*
                _ => None,
            }
        }
    })
}

pub(crate) fn generate_formatting_tables(
    numbers_main: &Path,
    dates_main: &Path,
) -> Result<TokenStream, String> {
    let mut locales = Vec::new();
    for entry in fs::read_dir(numbers_main)
        .map_err(|error| format!("{}: {error}", numbers_main.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", numbers_main.display()))?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            locales.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    locales.sort();

    let mut number_arms = Vec::new();
    let mut currency_arms = Vec::new();
    let mut date_arms = Vec::new();

    for locale in locales {
        let numbers_path = numbers_main.join(&locale).join("numbers.json");
        let numbers_value = read_json(&numbers_path)?;
        if let Some(numbers) = extract_numbers(&numbers_value, &locale) {
            number_arms.push(number_arm(&locale, numbers));
        }
        if let Some(currency) = extract_currency(&numbers_value, &locale) {
            currency_arms.push(currency_arm(&locale, currency));
        }

        let dates_path = dates_main.join(&locale).join("ca-gregorian.json");
        if dates_path.is_file() {
            let dates_value = read_json(&dates_path)?;
            if let Some(dates) = extract_dates(&dates_value, &locale) {
                date_arms.push(date_arm(&locale, dates));
            }
        }
    }

    Ok(quote! {
        fn generated_number_formatting(locale: &str) -> Option<NumberFormatData> {
            match locale {
                #(#number_arms)*
                _ => None,
            }
        }

        fn generated_currency_formatting(locale: &str) -> Option<CurrencyFormatData> {
            match locale {
                #(#currency_arms)*
                _ => None,
            }
        }

        fn generated_date_formatting(locale: &str) -> Option<DateFormatData> {
            match locale {
                #(#date_arms)*
                _ => None,
            }
        }
    })
}

fn number_arm(locale: &str, numbers: NumberData) -> TokenStream {
    let decimal_symbol = numbers.decimal_symbol;
    let group_symbol = numbers.group_symbol;
    let decimal_pattern = number_pattern_tokens(&numbers.decimal_pattern);
    let percent_pattern = number_pattern_tokens(&numbers.percent_pattern);
    quote! {
        #locale => Some(NumberFormatData {
            locale: #locale,
            decimal_symbol: #decimal_symbol,
            group_symbol: #group_symbol,
            decimal_pattern: #decimal_pattern,
            percent_pattern: #percent_pattern,
        }),
    }
}

fn currency_arm(locale: &str, currency: CurrencyData) -> TokenStream {
    let standard_pattern = number_pattern_tokens(&currency.standard_pattern);
    let accounting_pattern = currency.accounting_pattern.as_ref().map_or_else(
        || quote! { None },
        |pattern| {
            let pattern = number_pattern_tokens(pattern);
            quote! { Some(#pattern) }
        },
    );
    quote! {
        #locale => Some(CurrencyFormatData {
            locale: #locale,
            standard_pattern: #standard_pattern,
            accounting_pattern: #accounting_pattern,
        }),
    }
}

fn date_arm(locale: &str, dates: DateData) -> TokenStream {
    let date_formats = widths_tokens(&dates.date_formats);
    let time_formats = widths_tokens(&dates.time_formats);
    let date_time_formats = widths_tokens(&dates.date_time_formats);
    let months = symbol_widths_tokens(&dates.months);
    let weekdays = symbol_widths_tokens(&dates.weekdays);
    quote! {
        #locale => Some(DateFormatData {
            locale: #locale,
            date_formats: #date_formats,
            time_formats: #time_formats,
            date_time_formats: #date_time_formats,
            months: #months,
            weekdays: #weekdays,
        }),
    }
}

struct NumberData {
    decimal_symbol: String,
    group_symbol: String,
    decimal_pattern: NumberPattern,
    percent_pattern: NumberPattern,
}

struct CurrencyData {
    standard_pattern: NumberPattern,
    accounting_pattern: Option<NumberPattern>,
}

struct DateData {
    date_formats: WidthData,
    time_formats: WidthData,
    date_time_formats: WidthData,
    months: SymbolWidthData,
    weekdays: SymbolWidthData,
}

struct WidthData {
    full: String,
    long: String,
    medium: String,
    short: String,
}

struct SymbolWidthData {
    wide: Vec<String>,
    abbreviated: Vec<String>,
}

fn read_json(path: &Path) -> Result<Value, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&source).map_err(|error| format!("{}: {error}", path.display()))
}

fn extract_numbers(value: &Value, locale: &str) -> Option<NumberData> {
    let numbers = value.get("main")?.get(locale)?.get("numbers")?;
    let symbols = numbers.get("symbols-numberSystem-latn")?;
    let decimal_formats = numbers.get("decimalFormats-numberSystem-latn")?;
    let percent_formats = numbers.get("percentFormats-numberSystem-latn")?;
    Some(NumberData {
        decimal_symbol: string_field(symbols, "decimal")?,
        group_symbol: string_field(symbols, "group")?,
        decimal_pattern: parse_number_pattern(&string_field(decimal_formats, "standard")?),
        percent_pattern: parse_number_pattern(&string_field(percent_formats, "standard")?),
    })
}

fn extract_currency(value: &Value, locale: &str) -> Option<CurrencyData> {
    let currency_formats = value
        .get("main")?
        .get(locale)?
        .get("numbers")?
        .get("currencyFormats-numberSystem-latn")?;
    Some(CurrencyData {
        standard_pattern: parse_number_pattern(&string_field(currency_formats, "standard")?),
        accounting_pattern: string_field(currency_formats, "accounting")
            .map(|pattern| parse_number_pattern(&pattern)),
    })
}

fn extract_dates(value: &Value, locale: &str) -> Option<DateData> {
    let gregorian = value
        .get("main")?
        .get(locale)?
        .get("dates")?
        .get("calendars")?
        .get("gregorian")?;
    Some(DateData {
        date_formats: extract_widths(gregorian.get("dateFormats")?)?,
        time_formats: extract_widths(gregorian.get("timeFormats")?)?,
        date_time_formats: extract_widths(gregorian.get("dateTimeFormats")?)?,
        months: extract_months(gregorian)?,
        weekdays: extract_weekdays(gregorian)?,
    })
}

fn extract_widths(value: &Value) -> Option<WidthData> {
    Some(WidthData {
        full: string_field(value, "full")?,
        long: string_field(value, "long")?,
        medium: string_field(value, "medium")?,
        short: string_field(value, "short")?,
    })
}

fn extract_months(gregorian: &Value) -> Option<SymbolWidthData> {
    let format = gregorian.get("months")?.get("format")?;
    Some(SymbolWidthData {
        wide: extract_numbered_symbols(format.get("wide")?, 1..=12)?,
        abbreviated: extract_numbered_symbols(format.get("abbreviated")?, 1..=12)?,
    })
}

fn extract_weekdays(gregorian: &Value) -> Option<SymbolWidthData> {
    let format = gregorian.get("days")?.get("format")?;
    let keys = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    Some(SymbolWidthData {
        wide: extract_named_symbols(format.get("wide")?, &keys)?,
        abbreviated: extract_named_symbols(format.get("abbreviated")?, &keys)?,
    })
}

fn extract_numbered_symbols(
    value: &Value,
    range: std::ops::RangeInclusive<u8>,
) -> Option<Vec<String>> {
    range
        .map(|index| string_field(value, &index.to_string()))
        .collect()
}

fn extract_named_symbols(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
    keys.iter().map(|key| string_field(value, key)).collect()
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_owned)
}

fn widths_tokens(value: &WidthData) -> TokenStream {
    let full = &value.full;
    let long = &value.long;
    let medium = &value.medium;
    let short = &value.short;
    quote! {
        FormatWidths {
            full: #full,
            long: #long,
            medium: #medium,
            short: #short,
        }
    }
}

fn symbol_widths_tokens(value: &SymbolWidthData) -> TokenStream {
    let wide = string_vec_tokens(&value.wide);
    let abbreviated = string_vec_tokens(&value.abbreviated);
    quote! {
        DateSymbolWidths {
            wide: #wide,
            abbreviated: #abbreviated,
        }
    }
}

fn string_vec_tokens(values: &[String]) -> TokenStream {
    let values = values
        .iter()
        .map(|value| LitStr::new(value, Span::call_site()));
    quote! { [#(#values),*] }
}

fn number_pattern_tokens(pattern: &NumberPattern) -> TokenStream {
    let positive = number_pattern_part_tokens(&pattern.positive);
    let negative = pattern.negative.as_ref().map_or_else(
        || quote! { None },
        |part| {
            let part = number_pattern_part_tokens(part);
            quote! { Some(#part) }
        },
    );
    quote! {
        NumberPattern {
            positive: #positive,
            negative: #negative,
        }
    }
}

fn number_pattern_part_tokens(part: &NumberPatternPart) -> TokenStream {
    let prefix = &part.prefix;
    let suffix = &part.suffix;
    let min_integer_digits = part.min_integer_digits;
    let min_fraction_digits = part.min_fraction_digits;
    let max_fraction_digits = part.max_fraction_digits;
    let primary_group_size = option_u8_tokens(part.primary_group_size);
    let secondary_group_size = option_u8_tokens(part.secondary_group_size);
    quote! {
        NumberPatternPart {
            prefix: #prefix,
            suffix: #suffix,
            min_integer_digits: #min_integer_digits,
            min_fraction_digits: #min_fraction_digits,
            max_fraction_digits: #max_fraction_digits,
            primary_group_size: #primary_group_size,
            secondary_group_size: #secondary_group_size,
        }
    }
}

fn option_u8_tokens(value: Option<u8>) -> TokenStream {
    value.map_or_else(|| quote! { None }, |value| quote! { Some(#value) })
}

struct NumberPattern {
    positive: NumberPatternPart,
    negative: Option<NumberPatternPart>,
}

struct NumberPatternPart {
    prefix: String,
    suffix: String,
    min_integer_digits: u8,
    min_fraction_digits: u8,
    max_fraction_digits: u8,
    primary_group_size: Option<u8>,
    secondary_group_size: Option<u8>,
}

fn parse_number_pattern(pattern: &str) -> NumberPattern {
    let mut parts = pattern.splitn(2, ';');
    let positive = parse_number_pattern_part(parts.next().unwrap_or(""));
    let negative = parts.next().map(parse_number_pattern_part);
    NumberPattern { positive, negative }
}

fn parse_number_pattern_part(pattern: &str) -> NumberPatternPart {
    let number_start = pattern.find(['#', '0', ',', '.']).unwrap_or(pattern.len());
    let number_end = pattern
        .rfind(['#', '0', ',', '.'])
        .map_or(number_start, |index| index + 1);
    let prefix = pattern[..number_start].to_owned();
    let suffix = pattern[number_end..].to_owned();
    let number = &pattern[number_start..number_end];
    let (integer, fraction) = number.split_once('.').unwrap_or((number, ""));

    let group_sizes = integer
        .rsplit(',')
        .map(|group| {
            group
                .chars()
                .filter(|character| matches!(character, '#' | '0'))
                .count()
        })
        .collect::<Vec<_>>();

    NumberPatternPart {
        prefix,
        suffix,
        min_integer_digits: integer
            .chars()
            .filter(|character| *character == '0')
            .count() as u8,
        min_fraction_digits: fraction
            .chars()
            .filter(|character| *character == '0')
            .count() as u8,
        max_fraction_digits: fraction
            .chars()
            .filter(|character| matches!(character, '#' | '0'))
            .count() as u8,
        primary_group_size: (group_sizes.len() > 1).then_some(group_sizes[0] as u8),
        secondary_group_size: (group_sizes.len() > 2).then_some(group_sizes[1] as u8),
    }
}

fn extract_text_direction(source: &str) -> Option<&'static str> {
    let key = "\"characterOrder\"";
    let key_start = source.find(key)?;
    let after_key = &source[key_start + key.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    if after_colon.starts_with("\"right-to-left\"") {
        Some("rtl")
    } else if after_colon.starts_with("\"left-to-right\"") {
        Some("ltr")
    } else {
        None
    }
}
