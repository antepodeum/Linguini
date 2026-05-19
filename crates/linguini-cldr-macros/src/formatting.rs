use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;
use std::fs;
use std::path::Path;

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
    let decimal_pattern = numbers.decimal_pattern;
    let percent_pattern = numbers.percent_pattern;
    quote! {
        #locale => Some(NumberFormatData {
            locale: #locale.to_owned(),
            decimal_symbol: #decimal_symbol.to_owned(),
            group_symbol: #group_symbol.to_owned(),
            decimal_pattern: #decimal_pattern.to_owned(),
            percent_pattern: #percent_pattern.to_owned(),
        }),
    }
}

fn currency_arm(locale: &str, currency: CurrencyData) -> TokenStream {
    let standard_pattern = currency.standard_pattern;
    let accounting_pattern = option_string_tokens(currency.accounting_pattern.as_deref());
    quote! {
        #locale => Some(CurrencyFormatData {
            locale: #locale.to_owned(),
            standard_pattern: #standard_pattern.to_owned(),
            accounting_pattern: #accounting_pattern,
        }),
    }
}

fn date_arm(locale: &str, dates: DateData) -> TokenStream {
    let date_formats = widths_tokens(&dates.date_formats);
    let time_formats = widths_tokens(&dates.time_formats);
    let date_time_formats = widths_tokens(&dates.date_time_formats);
    quote! {
        #locale => Some(DateFormatData {
            locale: #locale.to_owned(),
            date_formats: #date_formats,
            time_formats: #time_formats,
            date_time_formats: #date_time_formats,
        }),
    }
}

struct NumberData {
    decimal_symbol: String,
    group_symbol: String,
    decimal_pattern: String,
    percent_pattern: String,
}

struct CurrencyData {
    standard_pattern: String,
    accounting_pattern: Option<String>,
}

struct DateData {
    date_formats: WidthData,
    time_formats: WidthData,
    date_time_formats: WidthData,
}

struct WidthData {
    full: String,
    long: String,
    medium: String,
    short: String,
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
        decimal_pattern: string_field(decimal_formats, "standard")?,
        percent_pattern: string_field(percent_formats, "standard")?,
    })
}

fn extract_currency(value: &Value, locale: &str) -> Option<CurrencyData> {
    let currency_formats = value
        .get("main")?
        .get(locale)?
        .get("numbers")?
        .get("currencyFormats-numberSystem-latn")?;
    Some(CurrencyData {
        standard_pattern: string_field(currency_formats, "standard")?,
        accounting_pattern: string_field(currency_formats, "accounting"),
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

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_owned)
}

fn option_string_tokens(value: Option<&str>) -> TokenStream {
    value.map_or_else(
        || quote! { None },
        |value| quote! { Some(#value.to_owned()) },
    )
}

fn widths_tokens(value: &WidthData) -> TokenStream {
    let full = &value.full;
    let long = &value.long;
    let medium = &value.medium;
    let short = &value.short;
    quote! {
        FormatWidths {
            full: #full.to_owned(),
            long: #long.to_owned(),
            medium: #medium.to_owned(),
            short: #short.to_owned(),
        }
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
