mod affix;

use affix::{affix_tokens, compile_affix, CompiledAffix, NumberAffixSymbols};
use linguini_cldr::{NumberPadding, NumberPaddingPosition};
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct FormattingCoverage {
    pub(crate) numbering_systems: usize,
    pub(crate) text_direction_locales: usize,
    pub(crate) number_locales: usize,
    pub(crate) currency_locales: usize,
    pub(crate) currency_fraction_rules: usize,
    pub(crate) date_locales: usize,
    pub(crate) text_direction_exclusions: Vec<String>,
    pub(crate) date_exclusions: Vec<String>,
}

pub(crate) const TEXT_DIRECTION_EXCLUSIONS: [&str; 2] = ["mn-Mong", "mn-Mong-MN"];
pub(crate) const DATE_EXCLUSIONS: [&str; 1] = ["haw"];

pub(crate) fn generate_text_direction_table(
    layout_main: &Path,
    locales: &[String],
) -> Result<(TokenStream, usize, Vec<String>), String> {
    let mut directions = Vec::new();
    let mut exclusions = Vec::new();
    for locale in locales {
        let layout = layout_main.join(locale).join("layout.json");
        let value = read_json(&layout)?;
        let direction = extract_text_direction(&value, locale)
            .map_err(|error| format!("{}: {error}", layout.display()))?;
        match direction {
            "left-to-right" => directions.push((locale.clone(), "ltr")),
            "right-to-left" => directions.push((locale.clone(), "rtl")),
            "top-to-bottom" => exclusions.push(locale.clone()),
            unsupported => {
                return Err(format!(
                    "{}: unsupported character order `{unsupported}` for locale `{locale}`",
                    layout.display()
                ))
            }
        }
    }
    require_exact_exclusions("text direction", &exclusions, &TEXT_DIRECTION_EXCLUSIONS)?;

    let arms = directions
        .iter()
        .map(|(locale, direction)| quote! { #locale => Some(#direction), });

    Ok((
        quote! {
            fn generated_text_direction(locale: &str) -> Option<&'static str> {
                match locale {
                    #(#arms)*
                    _ => None,
                }
            }
        },
        directions.len(),
        exclusions,
    ))
}

fn require_exact_exclusions(
    kind: &str,
    actual: &[String],
    expected: &[&str],
) -> Result<(), String> {
    if actual
        .iter()
        .map(String::as_str)
        .eq(expected.iter().copied())
    {
        Ok(())
    } else {
        Err(format!(
            "CLDR {kind} exclusions changed: got {actual:?}, expected {expected:?}"
        ))
    }
}

pub(crate) fn generate_formatting_tables(
    numbers_main: &Path,
    dates_main: &Path,
    currency_data: &Path,
    numbering_systems_path: &Path,
    locales: &[String],
) -> Result<(TokenStream, FormattingCoverage), String> {
    let mut number_arms = Vec::new();
    let mut currency_arms = Vec::new();
    let mut symbol_arms = Vec::new();
    let mut date_arms = Vec::new();
    let mut date_exclusions = Vec::new();
    let numbering_systems = extract_numbering_systems(numbering_systems_path)?;

    for locale in locales {
        let numbers_path = numbers_main.join(locale).join("numbers.json");
        let numbers_value = read_json(&numbers_path)?;
        let numbers = extract_numbers(&numbers_value, locale, &numbering_systems)
            .map_err(|error| format!("{}: {error}", numbers_path.display()))?;
        let digits = numbers.digits.clone();
        number_arms.push(number_arm(locale, numbers));
        let currency = extract_currency(&numbers_value, locale)
            .map_err(|error| format!("{}: {error}", numbers_path.display()))?;
        currency_arms.push(currency_arm(locale, currency));
        let path = numbers_main.join(locale).join("currencies.json");
        let value = read_json(&path)?;
        let symbols = value["main"][locale]["numbers"]["currencies"]
            .as_object()
            .ok_or_else(|| format!("{}: missing currencies object", path.display()))?;
        let entries = symbols.iter().map(|(code, value)| {
            let symbol = value["symbol"].as_str().unwrap_or(code);
            quote! { (#code, #symbol) }
        });
        symbol_arms.push(quote! { #locale => Some(&[#(#entries),*]), });

        let dates_path = dates_main.join(locale).join("ca-gregorian.json");
        let dates_value = read_json(&dates_path)?;
        match extract_dates(&dates_value, locale)
            .map_err(|error| format!("{}: {error}", dates_path.display()))?
        {
            Some(dates) => date_arms.push(date_arm(locale, dates, &digits)),
            None => date_exclusions.push(locale.clone()),
        }
    }
    require_exact_exclusions("date", &date_exclusions, &DATE_EXCLUSIONS)?;
    let currency_fractions = extract_currency_fractions(currency_data)?;
    let currency_fraction_rules = currency_fractions.len();
    let currency_fraction_arms = currency_fractions.iter().map(currency_fraction_arm);

    let coverage = FormattingCoverage {
        numbering_systems: numbering_systems.len(),
        text_direction_locales: 0,
        number_locales: number_arms.len(),
        currency_locales: currency_arms.len(),
        currency_fraction_rules,
        date_locales: date_arms.len(),
        text_direction_exclusions: Vec::new(),
        date_exclusions,
    };
    Ok((
        quote! {
            fn generated_number_formatting(locale: &str) -> Option<&'static NumberFormatData> {
                match locale {
                    #(#number_arms)*
                    _ => None,
                }
            }

            fn generated_currency_formatting(locale: &str) -> Option<&'static CurrencyFormatData> {
                match locale {
                    #(#currency_arms)*
                    _ => None,
                }
            }

            fn generated_currency_symbols(locale: &str) -> Option<&'static [(&'static str, &'static str)]> {
                match locale { #(#symbol_arms)* _ => None }
            }

            fn generated_currency_fraction(currency: &str) -> Option<CurrencyFractionData> {
                match currency {
                    #(#currency_fraction_arms)*
                    _ => None,
                }
            }

            fn generated_date_formatting(locale: &str) -> Option<&'static DateFormatData> {
                match locale {
                    #(#date_arms)*
                    _ => None,
                }
            }
        },
        coverage,
    ))
}

fn currency_fraction_arm((currency, rule): &(String, CurrencyFractionRule)) -> TokenStream {
    let digits = rule.digits;
    let rounding = rule.rounding;
    let cash_digits = rule.cash_digits;
    let cash_rounding = rule.cash_rounding;
    quote! {
        #currency => Some(CurrencyFractionData {
            digits: #digits,
            rounding: #rounding,
            cash_digits: #cash_digits,
            cash_rounding: #cash_rounding,
        }),
    }
}

fn number_arm(locale: &str, numbers: NumberData) -> TokenStream {
    let numbering_system = numbers.numbering_system;
    let digits = numbers.digits;
    let decimal_symbol = numbers.decimal_symbol;
    let group_symbol = numbers.group_symbol;
    let minus_sign = numbers.minus_sign;
    let plus_sign = numbers.plus_sign;
    let exponential_symbol = numbers.exponential_symbol;
    let minimum_grouping_digits = numbers.minimum_grouping_digits;
    let decimal_pattern = number_pattern_tokens(&numbers.decimal_pattern);
    let percent_pattern = number_pattern_tokens(&numbers.percent_pattern);
    let scientific_pattern = number_pattern_tokens(&numbers.scientific_pattern);
    quote! {
        #locale => {
            static DATA: NumberFormatData = NumberFormatData {
                locale: #locale,
                numbering_system: #numbering_system,
                digits: #digits,
                decimal_symbol: #decimal_symbol,
                group_symbol: #group_symbol,
                minus_sign: #minus_sign,
                plus_sign: #plus_sign,
                exponential_symbol: #exponential_symbol,
                minimum_grouping_digits: #minimum_grouping_digits,
                decimal_pattern: #decimal_pattern,
                percent_pattern: #percent_pattern,
                scientific_pattern: #scientific_pattern,
            };
            Some(&DATA)
        },
    }
}

fn currency_arm(locale: &str, currency: CurrencyData) -> TokenStream {
    let decimal_symbol = currency.decimal_symbol;
    let group_symbol = currency.group_symbol;
    let standard_pattern = number_pattern_tokens(&currency.standard_pattern);
    let accounting_pattern =
        optional_number_pattern_tokens(currency.accounting_pattern.as_ref(), false);
    let standard_alpha_next_to_number =
        optional_number_pattern_tokens(currency.standard_alpha_next_to_number.as_ref(), true);
    let accounting_alpha_next_to_number =
        optional_number_pattern_tokens(currency.accounting_alpha_next_to_number.as_ref(), true);
    quote! {
        #locale => {
            static DATA: CurrencyFormatData = CurrencyFormatData {
                locale: #locale,
                decimal_symbol: #decimal_symbol,
                group_symbol: #group_symbol,
                standard_pattern: #standard_pattern,
                accounting_pattern: #accounting_pattern,
                standard_alpha_next_to_number: #standard_alpha_next_to_number,
                accounting_alpha_next_to_number: #accounting_alpha_next_to_number,
            };
            Some(&DATA)
        },
    }
}

fn optional_number_pattern_tokens(
    pattern: Option<&NumberPattern>,
    by_reference: bool,
) -> TokenStream {
    pattern.map_or_else(
        || quote! { None },
        |pattern| {
            let pattern = number_pattern_tokens(pattern);
            let pattern = if by_reference {
                quote! { &#pattern }
            } else {
                pattern
            };
            quote! { Some(#pattern) }
        },
    )
}

fn date_arm(locale: &str, dates: DateData, digits: &str) -> TokenStream {
    let date_formats = widths_tokens(&dates.date_formats);
    let time_formats = widths_tokens(&dates.time_formats);
    let date_time_formats = widths_tokens(&dates.date_time_formats);
    let months = symbol_widths_tokens(&dates.months);
    let weekdays = symbol_widths_tokens(&dates.weekdays);
    quote! {
        #locale => {
            static DATA: DateFormatData = DateFormatData {
                locale: #locale,
                digits: #digits,
                date_formats: #date_formats,
                time_formats: #time_formats,
                date_time_formats: #date_time_formats,
                months: #months,
                weekdays: #weekdays,
            };
            Some(&DATA)
        },
    }
}

struct NumberData {
    numbering_system: String,
    digits: String,
    decimal_symbol: String,
    group_symbol: String,
    minus_sign: String,
    plus_sign: String,
    exponential_symbol: String,
    minimum_grouping_digits: u8,
    decimal_pattern: NumberPattern,
    percent_pattern: NumberPattern,
    scientific_pattern: NumberPattern,
}

struct CurrencyData {
    decimal_symbol: String,
    group_symbol: String,
    standard_pattern: NumberPattern,
    accounting_pattern: Option<NumberPattern>,
    standard_alpha_next_to_number: Option<NumberPattern>,
    accounting_alpha_next_to_number: Option<NumberPattern>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CurrencyFractionRule {
    digits: u8,
    rounding: u16,
    cash_digits: u8,
    cash_rounding: u16,
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

fn extract_numbering_systems(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let value = read_json(path)?;
    let systems = required_field(
        required_field(&value, "supplemental", "root")?,
        "numberingSystems",
        "supplemental",
    )?
    .as_object()
    .ok_or_else(|| "supplemental.numberingSystems is not an object".to_owned())?;
    let mut numeric = BTreeMap::new();
    for (name, value) in systems {
        if value.get("_type").and_then(Value::as_str) != Some("numeric") {
            continue;
        }
        let digits = string_field(value, "_digits", &format!("numbering system {name}"))?;
        if digits.chars().count() != 10 {
            return Err(format!(
                "numbering system `{name}` has {} digits, expected 10",
                digits.chars().count()
            ));
        }
        numeric.insert(name.clone(), digits);
    }
    if !numeric.contains_key("latn") {
        return Err("numeric numbering systems omit required `latn`".to_owned());
    }
    Ok(numeric)
}

fn extract_numbers(
    value: &Value,
    locale: &str,
    numbering_systems: &BTreeMap<String, String>,
) -> Result<NumberData, String> {
    let main = required_field(value, "main", "root")?;
    let locale_value = required_field(main, locale, "main")?;
    let numbers = required_field(locale_value, "numbers", locale)?;
    let numbering_system = string_field(numbers, "defaultNumberingSystem", "numbers")?;
    let digits = numbering_systems.get(&numbering_system).ok_or_else(|| {
        format!("default numbering system `{numbering_system}` is not a numeric system")
    })?;
    let symbols = required_field(
        numbers,
        &format!("symbols-numberSystem-{numbering_system}"),
        "numbers",
    )?;
    let decimal_formats = required_field(
        numbers,
        &format!("decimalFormats-numberSystem-{numbering_system}"),
        "numbers",
    )?;
    let percent_formats = required_field(
        numbers,
        &format!("percentFormats-numberSystem-{numbering_system}"),
        "numbers",
    )?;
    let scientific_formats = required_field(
        numbers,
        &format!("scientificFormats-numberSystem-{numbering_system}"),
        "numbers",
    )?;
    let decimal_pattern = string_field(decimal_formats, "standard", "decimal formats")?;
    let percent_pattern = string_field(percent_formats, "standard", "percent formats")?;
    let scientific_pattern = string_field(scientific_formats, "standard", "scientific formats")?;
    let affix_symbols = NumberAffixSymbols::extract(symbols)?;
    let minimum_grouping_digits = string_field(numbers, "minimumGroupingDigits", "numbers")?
        .parse::<std::num::NonZeroU8>()
        .map_err(|_| "numbers.minimumGroupingDigits must be an integer in 1..=255".to_owned())?
        .get();
    Ok(NumberData {
        numbering_system,
        digits: digits.clone(),
        decimal_symbol: string_field(symbols, "decimal", "number symbols")?,
        group_symbol: string_field(symbols, "group", "number symbols")?,
        minus_sign: affix_symbols.minus.clone(),
        plus_sign: affix_symbols.plus.clone(),
        exponential_symbol: string_field(symbols, "exponential", "number symbols")?,
        minimum_grouping_digits,
        decimal_pattern: parse_number_pattern(&decimal_pattern, &affix_symbols)?,
        percent_pattern: parse_number_pattern(&percent_pattern, &affix_symbols)?,
        scientific_pattern: parse_number_pattern(&scientific_pattern, &affix_symbols)?,
    })
}

fn extract_currency(value: &Value, locale: &str) -> Result<CurrencyData, String> {
    let main = required_field(value, "main", "root")?;
    let locale_value = required_field(main, locale, "main")?;
    let numbers = required_field(locale_value, "numbers", locale)?;
    let numbering_system = string_field(numbers, "defaultNumberingSystem", "numbers")?;
    let currency_formats = required_field(
        numbers,
        &format!("currencyFormats-numberSystem-{numbering_system}"),
        "numbers",
    )?;
    let standard = string_field(currency_formats, "standard", "currency formats")?;
    let symbols = required_field(
        numbers,
        &format!("symbols-numberSystem-{numbering_system}"),
        "numbers",
    )?;
    let affix_symbols = NumberAffixSymbols::extract(symbols)?;
    let optional_pattern = |key: &str| {
        currency_formats
            .get(key)
            .map(|value| {
                let pattern = value
                    .as_str()
                    .ok_or_else(|| format!("currency formats.{key} is not a string"))?;
                parse_number_pattern(pattern, &affix_symbols)
            })
            .transpose()
    };
    let accounting_pattern = optional_pattern("accounting")?;
    let standard_alpha_next_to_number = optional_pattern("standard-alphaNextToNumber")?;
    let accounting_alpha_next_to_number = optional_pattern("accounting-alphaNextToNumber")?;
    Ok(CurrencyData {
        decimal_symbol: string_field(
            symbols,
            if symbols.get("currencyDecimal").is_some() {
                "currencyDecimal"
            } else {
                "decimal"
            },
            "number symbols",
        )?,
        group_symbol: string_field(
            symbols,
            if symbols.get("currencyGroup").is_some() {
                "currencyGroup"
            } else {
                "group"
            },
            "number symbols",
        )?,
        standard_pattern: parse_number_pattern(&standard, &affix_symbols)?,
        accounting_pattern,
        standard_alpha_next_to_number,
        accounting_alpha_next_to_number,
    })
}

fn extract_currency_fractions(path: &Path) -> Result<Vec<(String, CurrencyFractionRule)>, String> {
    let value = read_json(path)?;
    let fractions = required_field(
        required_field(
            required_field(&value, "supplemental", "root")?,
            "currencyData",
            "supplemental",
        )?,
        "fractions",
        "supplemental.currencyData",
    )?
    .as_object()
    .ok_or_else(|| "supplemental.currencyData.fractions is not an object".to_owned())?;

    if !fractions.contains_key("DEFAULT") {
        return Err("currency fractions are missing the CLDR `DEFAULT` rule".to_owned());
    }

    let mut rules = Vec::with_capacity(fractions.len());
    for (currency, value) in fractions {
        if currency != "DEFAULT"
            && (currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_uppercase()))
        {
            return Err(format!(
                "currency fraction key `{currency}` is not `DEFAULT` or an uppercase ISO code"
            ));
        }
        let context = format!("currency fractions.{currency}");
        let digits = unsigned_u8_field(value, "_digits", &context)?;
        let rounding = unsigned_u16_field(value, "_rounding", &context)?;
        let cash_digits =
            optional_unsigned_u8_field(value, "_cashDigits", &context)?.unwrap_or(digits);
        let cash_rounding =
            optional_unsigned_u16_field(value, "_cashRounding", &context)?.unwrap_or(rounding);
        rules.push((
            currency.clone(),
            CurrencyFractionRule {
                digits,
                rounding,
                cash_digits,
                cash_rounding,
            },
        ));
    }
    rules.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(rules)
}

fn unsigned_u8_field(value: &Value, key: &str, context: &str) -> Result<u8, String> {
    optional_unsigned_u8_field(value, key, context)?
        .ok_or_else(|| format!("{context}: missing `{key}`"))
}

fn optional_unsigned_u8_field(
    value: &Value,
    key: &str,
    context: &str,
) -> Result<Option<u8>, String> {
    optional_unsigned_field(value, key, context)?
        .map(|source| {
            source
                .parse::<u8>()
                .map_err(|_| format!("{context}.{key} is not a valid u8"))
        })
        .transpose()
}

fn unsigned_u16_field(value: &Value, key: &str, context: &str) -> Result<u16, String> {
    optional_unsigned_u16_field(value, key, context)?
        .ok_or_else(|| format!("{context}: missing `{key}`"))
}

fn optional_unsigned_u16_field(
    value: &Value,
    key: &str,
    context: &str,
) -> Result<Option<u16>, String> {
    optional_unsigned_field(value, key, context)?
        .map(|source| {
            source
                .parse::<u16>()
                .map_err(|_| format!("{context}.{key} is not a valid u16"))
        })
        .transpose()
}

fn optional_unsigned_field<'a>(
    value: &'a Value,
    key: &str,
    context: &str,
) -> Result<Option<&'a str>, String> {
    value
        .get(key)
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("{context}.{key} is not a string"))
        })
        .transpose()
}

fn extract_dates(value: &Value, locale: &str) -> Result<Option<DateData>, String> {
    let main = required_field(value, "main", "root")?;
    let locale_value = required_field(main, locale, "main")?;
    let dates = required_field(locale_value, "dates", locale)?;
    let calendars = required_field(dates, "calendars", "dates")?;
    let gregorian = required_field(calendars, "gregorian", "calendars")?;
    let date_formats = extract_date_widths(
        required_field(gregorian, "dateFormats", "gregorian")?,
        locale,
    )?;
    let time_formats = extract_widths(
        required_field(gregorian, "timeFormats", "gregorian")?,
        "timeFormats",
    )?;
    let date_time_formats = extract_widths(
        required_field(gregorian, "dateTimeFormats", "gregorian")?,
        "dateTimeFormats",
    )?;
    let months = extract_months(gregorian)?;
    let weekdays = extract_weekdays(gregorian)?;

    Ok(date_formats.map(|date_formats| DateData {
        date_formats,
        time_formats,
        date_time_formats,
        months,
        weekdays,
    }))
}

fn extract_widths(value: &Value, context: &str) -> Result<WidthData, String> {
    Ok(WidthData {
        full: string_field(value, "full", context)?,
        long: string_field(value, "long", context)?,
        medium: string_field(value, "medium", context)?,
        short: string_field(value, "short", context)?,
    })
}

fn extract_date_widths(value: &Value, locale: &str) -> Result<Option<WidthData>, String> {
    let full = string_field(value, "full", "dateFormats")?;
    let long = string_field(value, "long", "dateFormats")?;
    let medium = string_field(value, "medium", "dateFormats")?;
    let short = required_field(value, "short", "dateFormats")?;
    if let Some(short) = short.as_str() {
        return Ok(Some(WidthData {
            full,
            long,
            medium,
            short: short.to_owned(),
        }));
    }

    let object = short
        .as_object()
        .ok_or_else(|| "dateFormats.short is neither a string nor an object".to_owned())?;
    if locale == "haw"
        && object.len() == 2
        && object.get("_value").and_then(Value::as_str) == Some("d/M/yy")
        && object.get("_numbers").and_then(Value::as_str) == Some("M=romanlow")
    {
        Ok(None)
    } else {
        Err(format!(
            "dateFormats.short has unsupported structured value for locale `{locale}`"
        ))
    }
}

fn extract_months(gregorian: &Value) -> Result<SymbolWidthData, String> {
    let months = required_field(gregorian, "months", "gregorian")?;
    let format = required_field(months, "format", "months")?;
    Ok(SymbolWidthData {
        wide: extract_numbered_symbols(
            required_field(format, "wide", "months.format")?,
            1..=12,
            "months.format.wide",
        )?,
        abbreviated: extract_numbered_symbols(
            required_field(format, "abbreviated", "months.format")?,
            1..=12,
            "months.format.abbreviated",
        )?,
    })
}

fn extract_weekdays(gregorian: &Value) -> Result<SymbolWidthData, String> {
    let days = required_field(gregorian, "days", "gregorian")?;
    let format = required_field(days, "format", "days")?;
    let keys = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    Ok(SymbolWidthData {
        wide: extract_named_symbols(
            required_field(format, "wide", "days.format")?,
            &keys,
            "days.format.wide",
        )?,
        abbreviated: extract_named_symbols(
            required_field(format, "abbreviated", "days.format")?,
            &keys,
            "days.format.abbreviated",
        )?,
    })
}

fn extract_numbered_symbols(
    value: &Value,
    range: std::ops::RangeInclusive<u8>,
    context: &str,
) -> Result<Vec<String>, String> {
    range
        .map(|index| string_field(value, &index.to_string(), context))
        .collect()
}

fn extract_named_symbols(
    value: &Value,
    keys: &[&str],
    context: &str,
) -> Result<Vec<String>, String> {
    keys.iter()
        .map(|key| string_field(value, key, context))
        .collect()
}

fn required_field<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("{context}: missing `{key}`"))
}

fn string_field(value: &Value, key: &str, context: &str) -> Result<String, String> {
    required_field(value, key, context)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{context}.{key} is not a string"))
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
    let values = values.iter();
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
    let prefix = affix_tokens(&part.prefix);
    let suffix = affix_tokens(&part.suffix);
    let min_integer_digits = part.min_integer_digits;
    let max_integer_digits = option_u8_tokens(part.max_integer_digits);
    let min_fraction_digits = part.min_fraction_digits;
    let max_fraction_digits = part.max_fraction_digits;
    let decimal_separator_always = part.decimal_separator_always;
    let primary_group_size = option_u8_tokens(part.primary_group_size);
    let secondary_group_size = option_u8_tokens(part.secondary_group_size);
    let min_significant_digits = option_u8_tokens(part.min_significant_digits);
    let max_significant_digits = option_u8_tokens(part.max_significant_digits);
    let rounding_increment = part
        .rounding_increment
        .as_ref()
        .map_or_else(|| quote! { None }, |increment| quote! { Some(#increment) });
    let exponent_digits = option_u8_tokens(part.exponent_digits);
    let exponent_sign_always = part.exponent_sign_always;
    let scale = part.scale;
    let padding = part.padding.as_ref().map_or_else(
        || quote! { None },
        |padding| {
            let character = padding.character;
            let width = padding.width;
            let position = match padding.position {
                NumberPaddingPosition::BeforePrefix => {
                    quote! { NumberPaddingPosition::BeforePrefix }
                }
                NumberPaddingPosition::AfterPrefix => {
                    quote! { NumberPaddingPosition::AfterPrefix }
                }
                NumberPaddingPosition::BeforeSuffix => {
                    quote! { NumberPaddingPosition::BeforeSuffix }
                }
                NumberPaddingPosition::AfterSuffix => {
                    quote! { NumberPaddingPosition::AfterSuffix }
                }
            };
            quote! { Some(NumberPadding { character: #character, width: #width, position: #position }) }
        },
    );
    quote! {
        NumberPatternPart {
            prefix: #prefix,
            suffix: #suffix,
            min_integer_digits: #min_integer_digits,
            max_integer_digits: #max_integer_digits,
            min_fraction_digits: #min_fraction_digits,
            max_fraction_digits: #max_fraction_digits,
            decimal_separator_always: #decimal_separator_always,
            primary_group_size: #primary_group_size,
            secondary_group_size: #secondary_group_size,
            min_significant_digits: #min_significant_digits,
            max_significant_digits: #max_significant_digits,
            rounding_increment: #rounding_increment,
            exponent_digits: #exponent_digits,
            exponent_sign_always: #exponent_sign_always,
            scale: #scale,
            padding: #padding,
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
    prefix: CompiledAffix,
    suffix: CompiledAffix,
    min_integer_digits: u8,
    max_integer_digits: Option<u8>,
    min_fraction_digits: u8,
    max_fraction_digits: u8,
    decimal_separator_always: bool,
    primary_group_size: Option<u8>,
    secondary_group_size: Option<u8>,
    min_significant_digits: Option<u8>,
    max_significant_digits: Option<u8>,
    rounding_increment: Option<String>,
    exponent_digits: Option<u8>,
    exponent_sign_always: bool,
    scale: u16,
    padding: Option<NumberPadding>,
}

struct PaddingMarker {
    index: usize,
    character: char,
    width: u16,
}

fn parse_number_pattern(
    pattern: &str,
    symbols: &NumberAffixSymbols,
) -> Result<NumberPattern, String> {
    let parts = split_number_subpatterns(pattern)?;
    if parts.is_empty() || parts.len() > 2 {
        return Err(format!("invalid number pattern `{pattern}`"));
    }
    let positive = parse_number_pattern_part(parts[0], symbols)?;
    let negative = parts
        .get(1)
        .filter(|part| !part.is_empty())
        .map(|part| parse_number_pattern_part(part, symbols))
        .transpose()?;
    Ok(NumberPattern { positive, negative })
}

fn parse_number_pattern_part(
    pattern: &str,
    symbols: &NumberAffixSymbols,
) -> Result<NumberPatternPart, String> {
    let original = pattern;
    let (pattern, padding_marker) = extract_padding(pattern)?;
    let number_start = find_number_skeleton(&pattern)
        .ok_or_else(|| format!("number pattern has no numeric skeleton: `{pattern}`"))?;
    let number_end = number_skeleton_end(&pattern, number_start);
    let (prefix, prefix_scale) = compile_affix(&pattern[..number_start], symbols)?;
    let (suffix, suffix_scale) = compile_affix(&pattern[number_end..], symbols)?;
    let number = &pattern[number_start..number_end];
    let padding = padding_marker
        .map(|marker| classify_padding(marker, number_start, number_end, pattern.len(), original))
        .transpose()?;
    let (mantissa, exponent) = number
        .split_once('E')
        .map_or((number, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    if mantissa.contains('E') || exponent.is_some_and(|value| value.contains('E')) {
        return Err(format!(
            "number pattern has multiple exponents: `{original}`"
        ));
    }
    let (exponent_digits, exponent_sign_always) = parse_exponent(exponent, original)?;
    if mantissa.matches('.').count() > 1 {
        return Err(format!(
            "number pattern has multiple decimals: `{original}`"
        ));
    }
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let leading_decimal = integer.is_empty() && mantissa.starts_with('.');
    if !leading_decimal
        && !integer
            .chars()
            .any(|character| matches!(character, '#' | '0'..='9' | '@'))
    {
        return Err(format!(
            "number pattern has no integer placeholders: `{original}`"
        ));
    }
    if integer
        .chars()
        .chain(fraction.chars())
        .any(|character| !matches!(character, '#' | '0'..='9' | '@' | ','))
    {
        return Err(format!(
            "number pattern has invalid skeleton syntax: `{original}`"
        ));
    }
    if fraction.contains(',') {
        return Err(format!(
            "number pattern groups fractional digits: `{original}`"
        ));
    }

    let group_sizes = integer
        .rsplit(',')
        .map(|group| {
            group
                .chars()
                .filter(|character| matches!(character, '#' | '0'..='9' | '@'))
                .count()
        })
        .collect::<Vec<_>>();
    // A leading separator defines the rightmost group without a prefix group.
    // Empty interior/trailing groups still have no numeric interpretation.
    if !integer.is_empty()
        && (group_sizes[0] == 0 || group_sizes[..group_sizes.len() - 1].contains(&0))
    {
        return Err(format!(
            "number pattern has an empty integer group: `{original}`"
        ));
    }
    if exponent.is_some() && group_sizes.len() > 1 {
        return Err(format!(
            "number pattern groups a scientific mantissa: `{original}`"
        ));
    }

    let (min_significant_digits, max_significant_digits) =
        parse_significant_digits(mantissa, original)?;
    let significant = min_significant_digits.is_some();
    if !significant {
        validate_fixed_digit_order(integer, fraction, original)?;
    }
    let rounding_increment = parse_rounding_increment(integer, fraction, significant);
    let scale = match (prefix_scale, suffix_scale) {
        (1, scale) | (scale, 1) => scale,
        _ => {
            return Err(format!(
                "number pattern has multiple scaling symbols: `{original}`"
            ))
        }
    };

    Ok(NumberPatternPart {
        prefix,
        suffix,
        min_integer_digits: checked_u8(
            integer.chars().filter(char::is_ascii_digit).count()
                + usize::from(leading_decimal && fraction.is_empty()),
            "minimum integer digits",
            original,
        )?,
        max_integer_digits: (exponent.is_some() && !significant)
            .then(|| {
                checked_u8(
                    integer
                        .chars()
                        .filter(|character| matches!(character, '#' | '0'..='9'))
                        .count(),
                    "maximum integer digits",
                    original,
                )
            })
            .transpose()?,
        min_fraction_digits: checked_u8(
            fraction
                .chars()
                .filter(char::is_ascii_digit)
                .count()
                .max(usize::from(leading_decimal && !fraction.is_empty())),
            "minimum fraction digits",
            original,
        )?,
        max_fraction_digits: checked_u8(
            fraction
                .chars()
                .filter(|character| matches!(character, '#' | '0'..='9'))
                .count(),
            "maximum fraction digits",
            original,
        )?,
        decimal_separator_always: mantissa.ends_with('.'),
        primary_group_size: (group_sizes.len() > 1)
            .then(|| checked_u8(group_sizes[0], "primary group size", original))
            .transpose()?,
        secondary_group_size: (group_sizes.len() > 2)
            .then(|| checked_u8(group_sizes[1], "secondary group size", original))
            .transpose()?,
        min_significant_digits,
        max_significant_digits,
        rounding_increment,
        exponent_digits,
        exponent_sign_always,
        scale,
        padding,
    })
}

fn parse_significant_digits(
    mantissa: &str,
    pattern: &str,
) -> Result<(Option<u8>, Option<u8>), String> {
    let Some(start) = mantissa.find('@') else {
        return Ok((None, None));
    };
    if mantissa.contains('.') || mantissa.chars().any(|character| character.is_ascii_digit()) {
        return Err(format!(
            "number pattern mixes significant and fixed digit syntax: `{pattern}`"
        ));
    }
    let mut minimum = 0;
    let mut maximum = 0;
    let mut optional = false;
    // Leading hashes only position grouping; precision starts at the first required digit.
    for character in mantissa[start..].chars() {
        match character {
            '@' if !optional => minimum += 1,
            '#' => optional = true,
            ',' => continue,
            _ => {
                return Err(format!(
                    "number pattern has invalid significant digit order: `{pattern}`"
                ))
            }
        }
        maximum += 1;
    }
    Ok((
        Some(checked_u8(minimum, "minimum significant digits", pattern)?),
        Some(checked_u8(maximum, "maximum significant digits", pattern)?),
    ))
}

fn validate_fixed_digit_order(integer: &str, fraction: &str, pattern: &str) -> Result<(), String> {
    let mut required = false;
    for character in integer.chars() {
        if character.is_ascii_digit() {
            required = true;
        } else if character == '#' && required {
            return Err(format!(
                "number pattern has an optional integer digit after a required digit: `{pattern}`"
            ));
        }
    }
    let mut optional = false;
    for character in fraction.chars() {
        if character == '#' {
            optional = true;
        } else if character.is_ascii_digit() && optional {
            return Err(format!(
                "number pattern has a required fraction digit after an optional digit: `{pattern}`"
            ));
        }
    }
    Ok(())
}

fn parse_exponent(exponent: Option<&str>, pattern: &str) -> Result<(Option<u8>, bool), String> {
    let Some(exponent) = exponent else {
        return Ok((None, false));
    };
    let (sign_always, digits) = exponent
        .strip_prefix('+')
        .map_or((false, exponent), |digits| (true, digits));
    if digits.is_empty() || !digits.chars().all(|character| character == '0') {
        return Err(format!(
            "number pattern has invalid exponent syntax: `{pattern}`"
        ));
    }
    Ok((
        Some(checked_u8(
            digits.len(),
            "minimum exponent digits",
            pattern,
        )?),
        sign_always,
    ))
}

fn parse_rounding_increment(integer: &str, fraction: &str, significant: bool) -> Option<String> {
    if significant
        || !integer
            .chars()
            .chain(fraction.chars())
            .any(|character| matches!(character, '1'..='9'))
    {
        return None;
    }
    let digits = integer
        .chars()
        .chain(fraction.chars())
        .filter(char::is_ascii_digit)
        .collect::<String>();
    Some(digits.trim_start_matches('0').to_owned())
}

fn extract_padding(pattern: &str) -> Result<(String, Option<PaddingMarker>), String> {
    let mut output = String::with_capacity(pattern.len());
    let mut marker = None;
    let mut quoted = false;
    let mut chars = pattern.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' {
            output.push(character);
            if chars.peek() == Some(&'\'') {
                output.push(chars.next().expect("peeked apostrophe"));
            } else {
                quoted = !quoted;
            }
        } else if character == '*' && !quoted {
            if marker.is_some() {
                return Err(format!(
                    "number pattern has multiple padding escapes: `{pattern}`"
                ));
            }
            let pad = chars.next().ok_or_else(|| {
                format!("number pattern has missing padding character: `{pattern}`")
            })?;
            marker = Some(PaddingMarker {
                index: output.len(),
                character: pad,
                width: visible_pattern_width(pattern)?,
            });
        } else {
            output.push(character);
        }
    }
    if quoted {
        return Err(format!("unterminated quote in number pattern `{pattern}`"));
    }
    Ok((output, marker))
}

fn visible_pattern_width(pattern: &str) -> Result<u16, String> {
    let mut width = 0usize;
    let mut quoted = false;
    let mut chars = pattern.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                width += 1;
            } else {
                quoted = !quoted;
            }
        } else if character == '*' && !quoted {
            chars.next().ok_or_else(|| {
                format!("number pattern has missing padding character: `{pattern}`")
            })?;
        } else {
            width += 1;
        }
    }
    if quoted {
        return Err(format!("unterminated quote in number pattern `{pattern}`"));
    }
    u16::try_from(width)
        .map_err(|_| format!("format width exceeds u16 in number pattern `{pattern}`"))
}

fn classify_padding(
    marker: PaddingMarker,
    number_start: usize,
    number_end: usize,
    pattern_length: usize,
    pattern: &str,
) -> Result<NumberPadding, String> {
    let PaddingMarker {
        index,
        character,
        width,
    } = marker;
    let position = if index == 0 {
        NumberPaddingPosition::BeforePrefix
    } else if index == number_start {
        NumberPaddingPosition::AfterPrefix
    } else if index == number_end {
        NumberPaddingPosition::BeforeSuffix
    } else if index == pattern_length {
        NumberPaddingPosition::AfterSuffix
    } else {
        return Err(format!(
            "number pattern pads inside a component: `{pattern}`"
        ));
    };
    Ok(NumberPadding {
        character,
        width,
        position,
    })
}

fn split_number_subpatterns(source: &str) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut chars = source.char_indices().peekable();
    while let Some((index, character)) = chars.next() {
        if character == '\'' {
            if chars
                .peek()
                .is_some_and(|(_, next_character)| *next_character == '\'')
            {
                chars.next();
            } else {
                quoted = !quoted;
            }
        } else if character == '*' && !quoted {
            // A padding escape makes its next code point opaque, even an apostrophe or `;`.
            chars.next();
        } else if character == ';' && !quoted {
            parts.push(&source[start..index]);
            start = index + character.len_utf8();
        }
    }
    if quoted {
        return Err(format!("unterminated quote in number pattern `{source}`"));
    }
    parts.push(&source[start..]);
    Ok(parts)
}

fn find_number_skeleton(pattern: &str) -> Option<usize> {
    let mut quoted = false;
    let mut chars = pattern.char_indices().peekable();
    while let Some((index, character)) = chars.next() {
        if character == '\'' {
            if chars
                .peek()
                .is_some_and(|(_, next_character)| *next_character == '\'')
            {
                chars.next();
            } else {
                quoted = !quoted;
            }
        } else if !quoted && matches!(character, '#' | '0'..='9' | '@' | '.' | ',') {
            return Some(index);
        }
    }
    None
}

fn number_skeleton_end(pattern: &str, start: usize) -> usize {
    for (offset, character) in pattern[start..].char_indices() {
        let exponent_sign = character == '+' && pattern[..start + offset].ends_with('E');
        if !matches!(character, '#' | '0'..='9' | '@' | ',' | '.' | 'E' | '*') && !exponent_sign {
            return start + offset;
        }
    }
    pattern.len()
}

fn checked_u8(value: usize, field: &str, pattern: &str) -> Result<u8, String> {
    u8::try_from(value).map_err(|_| format!("{field} exceeds u8 in number pattern `{pattern}`"))
}

fn extract_text_direction<'a>(value: &'a Value, locale: &str) -> Result<&'a str, String> {
    let main = required_field(value, "main", "root")?;
    let locale_value = required_field(main, locale, "main")?;
    let layout = required_field(locale_value, "layout", locale)?;
    let orientation = required_field(layout, "orientation", "layout")?;
    required_field(orientation, "characterOrder", "layout.orientation")?
        .as_str()
        .ok_or_else(|| "layout.orientation.characterOrder is not a string".to_owned())
}

#[cfg(test)]
mod tests {
    mod currency_affix_cases {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codegen/currency-affixes.rs"
        ));
    }
    mod optional_integer_cases {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codegen/optional-integer.rs"
        ));
    }
    use super::{
        extract_dates, extract_text_direction, require_exact_exclusions, NumberPaddingPosition,
    };
    use serde_json::json;

    fn literal_affix(text: &str) -> super::CompiledAffix {
        if text.is_empty() {
            Vec::new()
        } else {
            vec![linguini_cldr::NumberAffixPart::Literal(text.to_owned())]
        }
    }

    fn owned_affix(affix: linguini_cldr::NumberAffix) -> super::CompiledAffix {
        use linguini_cldr::{NumberAffix, NumberAffixPart};
        match affix {
            NumberAffix::Literal(text) => literal_affix(text),
            NumberAffix::CurrencySymbol => vec![NumberAffixPart::CurrencySymbol],
            NumberAffix::CurrencyCode => vec![NumberAffixPart::CurrencyCode],
            NumberAffix::Parts(parts) => parts
                .iter()
                .map(|part| match part {
                    NumberAffixPart::Literal(text) => NumberAffixPart::Literal((*text).to_owned()),
                    NumberAffixPart::CurrencySymbol => NumberAffixPart::CurrencySymbol,
                    NumberAffixPart::CurrencyCode => NumberAffixPart::CurrencyCode,
                })
                .collect(),
        }
    }

    #[test]
    fn optional_integer_patterns_preserve_the_shared_numeric_contract() {
        let mut samples = 0;
        for case in optional_integer_cases::CASES {
            let (numbers, expected) = case.formatting();
            let symbols = super::NumberAffixSymbols {
                minus: numbers.minus_sign.into(),
                plus: numbers.plus_sign.into(),
                percent: "%".into(),
                per_mille: "‰".into(),
            };
            let actual = super::parse_number_pattern(case.pattern, &symbols)
                .unwrap()
                .positive;
            assert_eq!(
                actual.min_integer_digits, expected.positive.min_integer_digits,
                "{}",
                case.name
            );
            assert_eq!(
                actual.min_fraction_digits, expected.positive.min_fraction_digits,
                "{}",
                case.name
            );
            assert_eq!(
                actual.max_fraction_digits, expected.positive.max_fraction_digits,
                "{}",
                case.name
            );
            assert_eq!(
                actual.decimal_separator_always, expected.positive.decimal_separator_always,
                "{}",
                case.name
            );
            assert_eq!(
                actual.primary_group_size, expected.positive.primary_group_size,
                "{}",
                case.name
            );
            assert_eq!(actual.scale, expected.positive.scale, "{}", case.name);
            assert_eq!(actual.padding, expected.positive.padding, "{}", case.name);
            assert_eq!(
                actual.prefix,
                owned_affix(expected.positive.prefix),
                "{}",
                case.name
            );
            assert_eq!(
                actual.suffix,
                owned_affix(expected.positive.suffix),
                "{}",
                case.name
            );
            samples += case.samples.len();
        }
        assert_eq!(samples, 228);
    }

    #[test]
    fn currency_affix_patterns_compile_the_shared_execution_contract() {
        for case in currency_affix_cases::CASES {
            let parsed = parse_number_pattern(case.pattern).unwrap();
            let label = format!(
                "{}: {} ({} / {})",
                case.name, case.pattern, case.positive, case.negative_text
            );
            assert_eq!(parsed.positive.prefix, owned_affix(case.prefix), "{label}");
            assert_eq!(parsed.positive.suffix, owned_affix(case.suffix), "{label}");
            if let Some((prefix, suffix)) = case.negative {
                let negative = parsed.negative.unwrap();
                assert_eq!(negative.prefix, owned_affix(prefix), "{label}");
                assert_eq!(negative.suffix, owned_affix(suffix), "{label}");
            } else {
                assert!(parsed.negative.is_none(), "{label}");
            }
        }
    }

    fn parse_number_pattern(pattern: &str) -> Result<super::NumberPattern, String> {
        super::parse_number_pattern(
            pattern,
            &super::NumberAffixSymbols {
                minus: "-".into(),
                plus: "+".into(),
                percent: "%".into(),
                per_mille: "‰".into(),
            },
        )
    }

    #[test]
    fn scientific_integer_and_precision_metadata_matches_the_icu_corpus() {
        let corpus: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/codegen/scientific-widths.json"
        ))
        .unwrap();
        for case in corpus["cases"].as_array().unwrap() {
            let pattern = case["pattern"].as_str().unwrap();
            let part = parse_number_pattern(pattern).unwrap().positive;
            assert_eq!(
                serde_json::json!({
                    "min_integer": part.min_integer_digits,
                    "max_integer": part.max_integer_digits,
                    "min_fraction": part.min_fraction_digits,
                    "max_fraction": part.max_fraction_digits,
                    "min_significant": part.min_significant_digits,
                    "max_significant": part.max_significant_digits,
                    "rounding_increment": part.rounding_increment,
                    "exponent_digits": part.exponent_digits,
                    "exponent_sign_always": part.exponent_sign_always,
                    "decimal_separator_always": part.decimal_separator_always,
                }),
                serde_json::Value::Object(
                    case.as_object()
                        .unwrap()
                        .iter()
                        .filter(|(key, _)| !matches!(key.as_str(), "pattern" | "samples"))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect()
                ),
                "{pattern}"
            );
        }
    }

    #[test]
    fn currency_symbols_resolve_overrides_and_validate_present_values() {
        let mut input = json!({"main": {"en": {"numbers": {
            "defaultNumberingSystem": "latn",
            "symbols-numberSystem-latn": {"decimal": ".", "group": ",", "minusSign": "-", "plusSign": "+", "percentSign": "%", "perMille": "‰"},
            "currencyFormats-numberSystem-latn": {"standard": "¤#,##0.00"}
        }}}});
        let ordinary = super::extract_currency(&input, "en").unwrap();
        assert_eq!(ordinary.decimal_symbol, ".");
        assert_eq!(ordinary.group_symbol, ",");
        assert!(ordinary.standard_alpha_next_to_number.is_none());
        assert!(ordinary.accounting_alpha_next_to_number.is_none());
        input["main"]["en"]["numbers"]["currencyFormats-numberSystem-latn"]
            ["standard-alphaNextToNumber"] = json!("¤ #,##0.00");
        input["main"]["en"]["numbers"]["currencyFormats-numberSystem-latn"]
            ["accounting-alphaNextToNumber"] = json!("¤ #,##0.00;(¤ #,##0.00)");
        let variants = super::extract_currency(&input, "en").unwrap();
        assert_eq!(
            variants
                .standard_alpha_next_to_number
                .unwrap()
                .positive
                .prefix,
            vec![
                linguini_cldr::NumberAffixPart::CurrencySymbol,
                linguini_cldr::NumberAffixPart::Literal(" ".to_owned())
            ]
        );
        assert!(variants
            .accounting_alpha_next_to_number
            .unwrap()
            .negative
            .is_some());
        for key in ["standard-alphaNextToNumber", "accounting-alphaNextToNumber"] {
            for invalid in [
                json!(null),
                json!(1),
                json!(false),
                json!([]),
                json!({}),
                json!("¤0..0"),
            ] {
                let mut malformed = input.clone();
                malformed["main"]["en"]["numbers"]["currencyFormats-numberSystem-latn"][key] =
                    invalid;
                assert!(super::extract_currency(&malformed, "en").is_err(), "{key}");
            }
        }
        input["main"]["en"]["numbers"]["symbols-numberSystem-latn"]["currencyDecimal"] = json!(":");
        input["main"]["en"]["numbers"]["symbols-numberSystem-latn"]["currencyGroup"] = json!(";");
        let currency = super::extract_currency(&input, "en").unwrap();
        assert_eq!(currency.decimal_symbol, ":");
        assert_eq!(currency.group_symbol, ";");
        for key in ["currencyDecimal", "currencyGroup"] {
            for invalid in [json!(null), json!(1), json!(false), json!([]), json!({})] {
                let mut malformed = input.clone();
                malformed["main"]["en"]["numbers"]["symbols-numberSystem-latn"][key] = invalid;
                assert!(super::extract_currency(&malformed, "en")
                    .err()
                    .unwrap()
                    .contains(key));
            }
        }
    }

    #[test]
    fn number_extraction_requires_a_positive_minimum_grouping_digit_count() {
        let mut input = json!({"main": {"en": {"numbers": {
            "defaultNumberingSystem": "latn", "minimumGroupingDigits": "2",
            "symbols-numberSystem-latn": {"decimal": ".", "group": ",", "minusSign": "-", "plusSign": "+", "exponential": "E", "percentSign": "%", "perMille": "‰"},
            "decimalFormats-numberSystem-latn": {"standard": "#,##0.###"},
            "percentFormats-numberSystem-latn": {"standard": "#,##0%"},
            "scientificFormats-numberSystem-latn": {"standard": "#E0"}
        }}}});
        let systems = std::collections::BTreeMap::from([("latn".into(), "0123456789".into())]);
        let mut missing_exponent = input.clone();
        missing_exponent["main"]["en"]["numbers"]["symbols-numberSystem-latn"]
            .as_object_mut()
            .unwrap()
            .remove("exponential");
        assert!(super::extract_numbers(&missing_exponent, "en", &systems)
            .err()
            .unwrap()
            .contains("exponential"));
        assert_eq!(
            super::extract_numbers(&input, "en", &systems)
                .unwrap()
                .minimum_grouping_digits,
            2
        );
        for (text, exponent, prefix, suffix) in [
            ("#E0", Some(1), "", ""),
            ("[#E0]", Some(1), "[", "]"),
            ("#", None, "", ""),
        ] {
            let mut sample = input.clone();
            sample["main"]["en"]["numbers"]["scientificFormats-numberSystem-latn"]["standard"] =
                json!(text);
            let result = super::extract_numbers(&sample, "en", &systems).unwrap();
            let pattern = result.scientific_pattern.positive;
            assert_eq!(pattern.exponent_digits, exponent, "{text}");
            assert_eq!(pattern.min_integer_digits, 0, "{text}");
            assert_eq!(pattern.max_fraction_digits, 0, "{text}");
            assert_eq!(pattern.prefix, literal_affix(prefix), "{text}");
            assert_eq!(pattern.suffix, literal_affix(suffix), "{text}");
        }
        let mut missing_scientific = input.clone();
        missing_scientific["main"]["en"]["numbers"]
            .as_object_mut()
            .unwrap()
            .remove("scientificFormats-numberSystem-latn");
        assert!(super::extract_numbers(&missing_scientific, "en", &systems)
            .err()
            .unwrap()
            .contains("scientificFormats-numberSystem-latn"));
        for invalid in [
            json!(null),
            json!(1),
            json!(false),
            json!([]),
            json!({}),
            json!("#E"),
        ] {
            let mut sample = input.clone();
            sample["main"]["en"]["numbers"]["scientificFormats-numberSystem-latn"]["standard"] =
                invalid;
            assert!(super::extract_numbers(&sample, "en", &systems).is_err());
        }
        for value in ["0", "256", "-1", "auto", ""] {
            input["main"]["en"]["numbers"]["minimumGroupingDigits"] = json!(value);
            assert!(
                super::extract_numbers(&input, "en", &systems).is_err(),
                "{value}"
            );
        }
    }

    #[test]
    fn affix_symbols_are_compiled_with_quote_provenance_and_scaling() {
        let symbols = super::NumberAffixSymbols {
            minus: "\u{200e}−".into(),
            plus: "\u{200e}+".into(),
            percent: "٪".into(),
            per_mille: "؉".into(),
        };
        let parse = |pattern| super::parse_number_pattern(pattern, &symbols).unwrap();
        let implicit = parse("0%");
        assert_eq!(implicit.positive.suffix, literal_affix("٪"));
        assert_eq!(implicit.positive.scale, 100);
        assert!(implicit.negative.is_none());
        let explicit = parse("+0‰;-0‰");
        assert_eq!(explicit.positive.prefix, literal_affix("\u{200e}+"));
        assert_eq!(explicit.positive.suffix, literal_affix("؉"));
        assert_eq!(explicit.positive.scale, 1000);
        assert_eq!(
            explicit.negative.unwrap().prefix,
            literal_affix("\u{200e}−")
        );
        let quoted = parse("'-+%'0'‰''';'minus'0'%' ");
        assert_eq!(quoted.positive.prefix, literal_affix("-+%"));
        assert_eq!(quoted.positive.suffix, literal_affix("‰'"));
        assert_eq!(quoted.positive.scale, 1);
        assert_eq!(quoted.negative.unwrap().suffix, literal_affix("% "));
        assert!(super::parse_number_pattern("%0‰", &symbols).is_err());
        assert!(super::parse_number_pattern("0%%", &symbols).is_err());
    }

    #[test]
    fn literal_currency_marks_remain_distinct_from_substitution_slots() {
        for (literal, placeholder) in [("'¤'0", "¤0"), ("0'¤'", "0¤"), ("'¤'¤0", "¤'¤'0")] {
            let literal = parse_number_pattern(literal).unwrap();
            let placeholder = parse_number_pattern(placeholder).unwrap();
            assert_ne!(
                super::number_pattern_part_tokens(&literal.positive).to_string(),
                super::number_pattern_part_tokens(&placeholder.positive).to_string(),
                "quoted currency marks must not become substitution slots"
            );
        }
    }

    #[test]
    fn currency_affix_parts_preserve_literal_and_marker_order() {
        use linguini_cldr::NumberAffixPart::{CurrencySymbol, Literal};
        for (source, expected) in [
            ("'¤'¤0", vec![Literal("¤".to_owned()), CurrencySymbol]),
            ("¤'¤'0", vec![CurrencySymbol, Literal("¤".to_owned())]),
            ("'a''¤'¤0", vec![Literal("a'¤".to_owned()), CurrencySymbol]),
            (
                "¤x¤0",
                vec![CurrencySymbol, Literal("x".to_owned()), CurrencySymbol],
            ),
            ("'¤¤¤¤¤'0", vec![Literal("¤¤¤¤¤".to_owned())]),
        ] {
            assert_eq!(
                parse_number_pattern(source).unwrap().positive.prefix,
                expected,
                "{source}"
            );
        }
    }

    #[test]
    fn unsupported_currency_display_widths_fail_instead_of_repeating_symbols() {
        for width in 3..=6 {
            let marker = "¤".repeat(width);
            for source in [format!("{marker}0"), format!("0{marker}")] {
                let error = parse_number_pattern(&source).err().unwrap();
                assert!(
                    error.contains("unsupported currency marker width"),
                    "{source}: {error}"
                );
            }
        }
    }

    #[test]
    fn number_pattern_parser_honors_quoted_affixes_and_subpatterns() {
        let pattern =
            parse_number_pattern("'pre;fix'#,##0.00' suffix';'negative'#,##0.00").expect("pattern");

        assert_eq!(pattern.positive.prefix, literal_affix("pre;fix"));
        assert_eq!(pattern.positive.suffix, literal_affix(" suffix"));
        assert_eq!(pattern.positive.primary_group_size, Some(3));
        assert_eq!(pattern.positive.min_fraction_digits, 2);
        assert_eq!(
            pattern.negative.expect("negative subpattern").prefix,
            literal_affix("negative")
        );
    }

    #[test]
    fn number_suffix_plus_signs_are_affixes_after_the_complete_skeleton() {
        let symbols = super::NumberAffixSymbols {
            minus: "−".into(),
            plus: "＋".into(),
            percent: "٪".into(),
            per_mille: "‰".into(),
        };
        for (source, suffix, exponent, sign_always, scale) in [
            ("0.00+", "＋", None, false, 1),
            ("0E0+", "＋", Some(1), false, 1),
            ("0E+00+", "＋", Some(2), true, 1),
            ("0+%", "＋٪", None, false, 100),
            ("0'+'", "+", None, false, 1),
        ] {
            let pattern = super::parse_number_pattern(source, &symbols).unwrap();
            assert_eq!(pattern.positive.suffix, literal_affix(suffix), "{source}");
            assert_eq!(pattern.positive.exponent_digits, exponent, "{source}");
            assert_eq!(
                pattern.positive.exponent_sign_always, sign_always,
                "{source}"
            );
            assert_eq!(pattern.positive.scale, scale, "{source}");
        }
        let pattern = super::parse_number_pattern("0.00+;0.00-", &symbols).unwrap();
        assert_eq!(pattern.positive.suffix, literal_affix("＋"));
        assert_eq!(pattern.negative.unwrap().suffix, literal_affix("−"));
        for invalid in ["0E+", "0E++00", "0E+0E0"] {
            assert!(
                super::parse_number_pattern(invalid, &symbols).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn trailing_subpattern_separators_preserve_quotes_padding_and_spaces() {
        for source in ["0;", "0.00;", "'pre;'0';tail';", "*;0;", "0 ;"] {
            let pattern = parse_number_pattern(source).unwrap();
            assert!(pattern.negative.is_none(), "{source}");
        }
        let quoted = parse_number_pattern("'pre;'0';tail';").unwrap();
        assert_eq!(quoted.positive.prefix, literal_affix("pre;"));
        assert_eq!(quoted.positive.suffix, literal_affix(";tail"));
        let padded = parse_number_pattern("*;0;").unwrap();
        assert_eq!(padded.positive.padding.unwrap().character, ';');
        let spaced = parse_number_pattern("0 ;").unwrap();
        assert_eq!(spaced.positive.suffix, literal_affix(" "));
        let negative = parse_number_pattern("0; -0 ").unwrap().negative.unwrap();
        assert_eq!(negative.prefix, literal_affix(" -"));
        assert_eq!(negative.suffix, literal_affix(" "));
        // A second separator is not an empty trailing negative subpattern.
        for invalid in ["0;;", "0;-0;", ";"] {
            assert!(parse_number_pattern(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn numeric_affix_metacharacters_require_quotes() {
        let symbols = super::NumberAffixSymbols {
            minus: "-".into(),
            plus: "+".into(),
            percent: "%".into(),
            per_mille: "‰".into(),
        };
        for character in "#@0123456789.,*;".chars() {
            let affix = format!("tail{character}");
            assert!(super::compile_affix(&affix, &symbols).is_err(), "{affix}");
        }
        for source in [
            "0 tail#",
            "0 tail@",
            "0 tail5",
            "0 tail,",
            "0 tail.",
            "руб.*;0.0'кг'",
        ] {
            assert!(parse_number_pattern(source).is_err(), "{source}");
        }
        let literal = "@#0123456789.,*;";
        let pattern = parse_number_pattern(&format!("'{literal}'0'{literal}'")).unwrap();
        assert_eq!(pattern.positive.prefix, literal_affix(literal));
        assert_eq!(pattern.positive.suffix, literal_affix(literal));
    }

    #[test]
    fn leading_separators_preserve_group_widths_and_reject_malformed_skeletons() {
        for (source, primary, secondary) in [
            (",0", 1, None),
            (",##0", 3, None),
            (",#,#0", 2, Some(1)),
            ("'literal.,'#,##0", 3, None),
        ] {
            let parsed = parse_number_pattern(source).unwrap().positive;
            assert_eq!(parsed.primary_group_size, Some(primary), "{source}");
            assert_eq!(parsed.secondary_group_size, secondary, "{source}");
        }
        for source in [
            ",0E0",
            ",,0",
            ",0,",
            ",0,,0",
            ",.0",
            "..0",
            ".,0",
            "'literal.,'0,##0",
        ] {
            let error = parse_number_pattern(source)
                .err()
                .unwrap_or_else(|| panic!("invalid or unsupported pattern accepted: {source}"));
            assert!(error.contains(source), "{source}: {error}");
        }
    }

    #[test]
    fn number_pattern_parser_models_significant_scientific_padding_and_rounding() {
        let significant = parse_number_pattern("@@##E+00")
            .expect("significant scientific")
            .positive;
        assert_eq!(significant.min_significant_digits, Some(2));
        assert_eq!(significant.max_significant_digits, Some(4));
        assert_eq!(significant.exponent_digits, Some(2));
        assert!(significant.exponent_sign_always);

        let rounded = parse_number_pattern("#,##0.05").expect("rounding").positive;
        assert_eq!(rounded.rounding_increment.as_deref(), Some("5"));
        assert_eq!(rounded.max_fraction_digits, 2);
        assert_eq!(rounded.min_fraction_digits, 2);

        for (pattern, increment, integer_digits) in
            [("5", "5", 1), ("50", "50", 2), ("#,#50", "50", 2)]
        {
            let rounded = parse_number_pattern(pattern)
                .expect("integer rounding")
                .positive;
            assert_eq!(
                rounded.rounding_increment.as_deref(),
                Some(increment),
                "{pattern}"
            );
            assert_eq!(rounded.min_integer_digits, integer_digits, "{pattern}");
            assert_eq!(rounded.max_fraction_digits, 0, "{pattern}");
            assert_eq!(rounded.prefix, literal_affix(""), "{pattern}");
        }

        let padded = parse_number_pattern("USD *x#,##0")
            .expect("padding")
            .positive;
        let padding = padded.padding.expect("padding model");
        assert!(matches!(
            padding.position,
            NumberPaddingPosition::AfterPrefix
        ));
        assert_eq!(padding.character, 'x');
        assert_eq!(padding.width, 9);

        assert_eq!(
            parse_number_pattern("#,##0%")
                .expect("percent")
                .positive
                .scale,
            100
        );
        assert_eq!(
            parse_number_pattern("#,##0\u{2030}")
                .expect("per mille")
                .positive
                .scale,
            1000
        );
    }

    #[test]
    fn trailing_decimal_separator_is_retained_in_compiled_number_metadata() {
        for (ordinary, visible) in [("0", "0."), ("#,##0", "#,##0."), ("0E+00", "0.E+00")] {
            let ordinary = parse_number_pattern(ordinary).unwrap();
            let visible = parse_number_pattern(visible).unwrap();
            let ordinary = super::number_pattern_part_tokens(&ordinary.positive).to_string();
            let visible = super::number_pattern_part_tokens(&visible.positive).to_string();
            assert_ne!(
                ordinary, visible,
                "decimal visibility must survive compilation"
            );
            assert!(visible.contains("decimal_separator_always : true"));
            assert!(ordinary.contains("decimal_separator_always : false"));
        }
        for pattern in ["0.0", "0.##", "0'.'"] {
            let parsed = parse_number_pattern(pattern).unwrap();
            assert!(super::number_pattern_part_tokens(&parsed.positive)
                .to_string()
                .contains("decimal_separator_always : false"));
        }
    }

    #[test]
    fn significant_precision_ignores_leading_grouping_placeholders() {
        for (pattern, minimum, maximum, grouping) in [
            ("#,#@#", 1, 2, Some(3)),
            ("###@@##", 2, 4, None),
            ("#,#@@##", 2, 4, Some(5)),
            ("##@@###E+00", 2, 5, None),
            ("'prefix#@'###@@##'suffix@#'", 2, 4, None),
        ] {
            let parsed = parse_number_pattern(pattern).unwrap().positive;
            assert_eq!(parsed.min_significant_digits, Some(minimum), "{pattern}");
            assert_eq!(parsed.max_significant_digits, Some(maximum), "{pattern}");
            assert_eq!(parsed.primary_group_size, grouping, "{pattern}");
        }
        let pattern = format!("{}@", "#".repeat(512));
        let parsed = parse_number_pattern(&pattern).unwrap().positive;
        assert_eq!(parsed.min_significant_digits, Some(1));
        assert_eq!(parsed.max_significant_digits, Some(1));
        for pattern in ["@".repeat(256), format!("@{}", "#".repeat(255))] {
            assert!(parse_number_pattern(&pattern).is_err(), "{pattern}");
        }
    }

    #[test]
    fn number_digit_order_and_grouping_are_validated_before_compilation() {
        for pattern in [
            "0#", "5#", "#0#", "0,#0", "0.#0", "0.#5", "0.0#0", "@@#@", "@#,@", "@.##", "0,,0",
            "0,", "#,##0E0", "@,@@E0",
        ] {
            for source in [pattern.to_owned(), format!("'pre'{pattern}'tail'")] {
                let error = parse_number_pattern(&source)
                    .err()
                    .unwrap_or_else(|| panic!("invalid pattern accepted: {source}"));
                assert!(error.contains(&source), "{error}");
            }
        }
        for pattern in [
            "###",
            "#.##",
            "#,##0.00##",
            "#,#50.05##",
            "00,00",
            "@@##E+00",
        ] {
            assert!(parse_number_pattern(pattern).is_ok(), "{pattern}");
        }
    }

    #[test]
    fn short_number_skeletons_follow_the_fixed_and_significant_grammars() {
        // Construct valid languages from their productions, then check every short word.
        let mut valid = std::collections::BTreeMap::new();
        for leading in 0..=6 {
            for required in 0..=6 - leading {
                if leading + required > 0 {
                    valid.insert(
                        format!("{}{}", "#".repeat(leading), "0".repeat(required)),
                        None,
                    );
                }
                if required == 0 {
                    continue;
                }
                for trailing in 0..=6 - leading - required {
                    valid.insert(
                        format!(
                            "{}{}{}",
                            "#".repeat(leading),
                            "@".repeat(required),
                            "#".repeat(trailing)
                        ),
                        Some((required as u8, (required + trailing) as u8)),
                    );
                }
            }
        }
        let mut checked = 0;
        for length in 1..=6 {
            for mut encoded in 0..3_usize.pow(length) {
                let word: String = (0..length)
                    .map(|_| {
                        let digit = b"#0@"[encoded % 3];
                        encoded /= 3;
                        digit as char
                    })
                    .collect();
                let parsed = parse_number_pattern(&word);
                assert_eq!(parsed.is_ok(), valid.contains_key(&word), "{word}");
                if let Ok(pattern) = parsed {
                    let limits = pattern
                        .positive
                        .min_significant_digits
                        .zip(pattern.positive.max_significant_digits);
                    assert_eq!(limits, valid[&word], "{word}");
                }
                checked += 1;
            }
        }
        assert_eq!(checked, 1_092);
    }

    #[test]
    fn number_padding_escapes_special_characters_and_preserves_quoted_affixes() {
        for (pattern, character, position, prefix, suffix, width) in [
            (
                "*'pre0.0tail",
                '\'',
                NumberPaddingPosition::BeforePrefix,
                "pre",
                "tail",
                10,
            ),
            (
                "pre*;0.0tail",
                ';',
                NumberPaddingPosition::AfterPrefix,
                "pre",
                "tail",
                10,
            ),
            (
                "pre0.0*💠tail",
                '💠',
                NumberPaddingPosition::BeforeSuffix,
                "pre",
                "tail",
                10,
            ),
            (
                "pre0.0tail**",
                '*',
                NumberPaddingPosition::AfterSuffix,
                "pre",
                "tail",
                10,
            ),
            (
                "'pre*''fix'*;0.0'ta;il'",
                ';',
                NumberPaddingPosition::AfterPrefix,
                "pre*'fix",
                "ta;il",
                16,
            ),
            (
                "*'0.0;'-'0.0",
                '\'',
                NumberPaddingPosition::BeforePrefix,
                "",
                "",
                3,
            ),
            (
                "pre*;0.0tail;-0.0",
                ';',
                NumberPaddingPosition::AfterPrefix,
                "pre",
                "tail",
                10,
            ),
            (
                "'руб.'*;0.0'кг'",
                ';',
                NumberPaddingPosition::AfterPrefix,
                "руб.",
                "кг",
                9,
            ),
        ] {
            let parsed = parse_number_pattern(pattern)
                .unwrap_or_else(|error| panic!("{pattern:?}: {error}"));
            assert_eq!(parsed.positive.prefix, literal_affix(prefix), "{pattern}");
            assert_eq!(parsed.positive.suffix, literal_affix(suffix), "{pattern}");
            let padding = parsed.positive.padding.unwrap();
            assert_eq!(padding.character, character, "{pattern}");
            assert_eq!(padding.position, position, "{pattern}");
            assert_eq!(padding.width, width, "{pattern}");
            assert_eq!(
                parsed.negative.is_some(),
                pattern.ends_with(";-0.0") || pattern.ends_with(";'-'0.0"),
                "{pattern}"
            );
        }
    }

    #[test]
    fn number_padding_rejects_positions_inside_affixes_and_the_skeleton() {
        for pattern in [
            "pre*xfix0.0tail",
            "pre0.0ta*xil",
            "р*xуб'.'0.0кг",
            "руб'.'0.0к*xг",
            "0.*x0",
            "0*x.0",
        ] {
            let error = parse_number_pattern(pattern)
                .err()
                .unwrap_or_else(|| panic!("padding inside a component must fail: {pattern}"));
            assert!(
                error.contains("pads inside a component"),
                "{pattern}: {error}"
            );
        }
    }

    #[test]
    fn number_pattern_parser_rejects_invalid_or_oversized_models() {
        assert!(parse_number_pattern("#E").is_err());
        assert!(parse_number_pattern("@@0").is_err());
        assert!(parse_number_pattern("*x*y#,##0").is_err());
        for pattern in ["0*", "pre0.0tail*", "*;*;0"] {
            assert!(parse_number_pattern(pattern).is_err(), "{pattern}");
        }
        assert!(parse_number_pattern(&format!("0.{}", "0".repeat(256))).is_err());
        assert!(parse_number_pattern("'unterminated#,##0").is_err());
    }

    #[test]
    fn text_direction_uses_json_structure_not_substring_order() {
        let value = json!({
            "decoy": {"characterOrder": "right-to-left"},
            "main": {
                "en": {
                    "layout": {
                        "orientation": {
                            "lineOrder": "top-to-bottom",
                            "characterOrder": "left-to-right"
                        }
                    }
                }
            }
        });

        assert_eq!(
            extract_text_direction(&value, "en").expect("direction"),
            "left-to-right"
        );
    }

    #[test]
    fn known_date_exclusion_still_validates_every_other_required_field() {
        let mut value = json!({
            "main": {
                "haw": {
                    "dates": {
                        "calendars": {
                            "gregorian": {
                                "dateFormats": {
                                    "full": "EEEE, d MMMM y",
                                    "long": "d MMMM y",
                                    "medium": "d MMM y",
                                    "short": {
                                        "_value": "d/M/yy",
                                        "_numbers": "M=romanlow"
                                    }
                                },
                                "timeFormats": {
                                    "full": "h:mm:ss a zzzz",
                                    "long": "h:mm:ss a z",
                                    "medium": "h:mm:ss a",
                                    "short": "h:mm a"
                                },
                                "dateTimeFormats": {
                                    "full": "{1} {0}",
                                    "long": "{1} {0}",
                                    "medium": "{1} {0}",
                                    "short": "{1} {0}"
                                },
                                "months": {
                                    "format": {
                                        "wide": {
                                            "1": "1", "2": "2", "3": "3", "4": "4",
                                            "5": "5", "6": "6", "7": "7", "8": "8",
                                            "9": "9", "10": "10", "11": "11", "12": "12"
                                        },
                                        "abbreviated": {
                                            "1": "1", "2": "2", "3": "3", "4": "4",
                                            "5": "5", "6": "6", "7": "7", "8": "8",
                                            "9": "9", "10": "10", "11": "11", "12": "12"
                                        }
                                    }
                                },
                                "days": {
                                    "format": {
                                        "wide": {
                                            "sun": "sun", "mon": "mon", "tue": "tue",
                                            "wed": "wed", "thu": "thu", "fri": "fri", "sat": "sat"
                                        },
                                        "abbreviated": {
                                            "sun": "sun", "mon": "mon", "tue": "tue",
                                            "wed": "wed", "thu": "thu", "fri": "fri", "sat": "sat"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        assert!(extract_dates(&value, "haw")
            .expect("known structured override")
            .is_none());
        value["main"]["haw"]["dates"]["calendars"]["gregorian"]
            .as_object_mut()
            .expect("gregorian object")
            .remove("months");
        let error = extract_dates(&value, "haw")
            .err()
            .expect("missing required field must not be hidden");
        assert!(error.contains("missing `months`"));
    }

    #[test]
    fn exclusion_manifest_must_match_actual_generation() {
        let expected = ["haw"];
        require_exact_exclusions("date", &["haw".to_owned()], &expected).expect("exact exclusion");
        assert!(require_exact_exclusions("date", &[], &expected).is_err());
        assert!(
            require_exact_exclusions("date", &["haw".to_owned(), "new".to_owned()], &expected)
                .is_err()
        );
    }
}
