use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct FormattingCoverage {
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
    locales: &[String],
) -> Result<(TokenStream, FormattingCoverage), String> {
    let mut number_arms = Vec::new();
    let mut currency_arms = Vec::new();
    let mut date_arms = Vec::new();
    let mut date_exclusions = Vec::new();

    for locale in locales {
        let numbers_path = numbers_main.join(locale).join("numbers.json");
        let numbers_value = read_json(&numbers_path)?;
        let numbers = extract_numbers(&numbers_value, locale)
            .map_err(|error| format!("{}: {error}", numbers_path.display()))?;
        number_arms.push(number_arm(locale, numbers));
        let currency = extract_currency(&numbers_value, locale)
            .map_err(|error| format!("{}: {error}", numbers_path.display()))?;
        currency_arms.push(currency_arm(locale, currency));

        let dates_path = dates_main.join(locale).join("ca-gregorian.json");
        let dates_value = read_json(&dates_path)?;
        match extract_dates(&dates_value, locale)
            .map_err(|error| format!("{}: {error}", dates_path.display()))?
        {
            Some(dates) => date_arms.push(date_arm(locale, dates)),
            None => date_exclusions.push(locale.clone()),
        }
    }
    require_exact_exclusions("date", &date_exclusions, &DATE_EXCLUSIONS)?;
    let currency_fractions = extract_currency_fractions(currency_data)?;
    let currency_fraction_rules = currency_fractions.len();
    let currency_fraction_arms = currency_fractions.iter().map(currency_fraction_arm);

    let coverage = FormattingCoverage {
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

            fn generated_currency_fraction(currency: &str) -> Option<CurrencyFractionData> {
                match currency {
                    #(#currency_fraction_arms)*
                    _ => None,
                }
            }

            fn generated_date_formatting(locale: &str) -> Option<DateFormatData> {
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

fn extract_numbers(value: &Value, locale: &str) -> Result<NumberData, String> {
    let main = required_field(value, "main", "root")?;
    let locale_value = required_field(main, locale, "main")?;
    let numbers = required_field(locale_value, "numbers", locale)?;
    let symbols = required_field(numbers, "symbols-numberSystem-latn", "numbers")?;
    let decimal_formats = required_field(numbers, "decimalFormats-numberSystem-latn", "numbers")?;
    let percent_formats = required_field(numbers, "percentFormats-numberSystem-latn", "numbers")?;
    let decimal_pattern = string_field(decimal_formats, "standard", "decimal formats")?;
    let percent_pattern = string_field(percent_formats, "standard", "percent formats")?;
    Ok(NumberData {
        decimal_symbol: string_field(symbols, "decimal", "latn symbols")?,
        group_symbol: string_field(symbols, "group", "latn symbols")?,
        decimal_pattern: parse_number_pattern(&decimal_pattern)?,
        percent_pattern: parse_number_pattern(&percent_pattern)?,
    })
}

fn extract_currency(value: &Value, locale: &str) -> Result<CurrencyData, String> {
    let main = required_field(value, "main", "root")?;
    let locale_value = required_field(main, locale, "main")?;
    let numbers = required_field(locale_value, "numbers", locale)?;
    let currency_formats = required_field(numbers, "currencyFormats-numberSystem-latn", "numbers")?;
    let standard = string_field(currency_formats, "standard", "currency formats")?;
    let accounting_pattern = currency_formats
        .get("accounting")
        .map(|value| {
            let pattern = value
                .as_str()
                .ok_or_else(|| "currency formats.accounting is not a string".to_owned())?;
            parse_number_pattern(pattern)
        })
        .transpose()?;
    Ok(CurrencyData {
        standard_pattern: parse_number_pattern(&standard)?,
        accounting_pattern,
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

fn parse_number_pattern(pattern: &str) -> Result<NumberPattern, String> {
    let parts = split_unquoted(pattern, ';')?;
    if parts.is_empty() || parts.len() > 2 {
        return Err(format!("invalid number pattern `{pattern}`"));
    }
    let positive = parse_number_pattern_part(parts[0])?;
    let negative = parts
        .get(1)
        .map(|part| parse_number_pattern_part(part))
        .transpose()?;
    Ok(NumberPattern { positive, negative })
}

fn parse_number_pattern_part(pattern: &str) -> Result<NumberPatternPart, String> {
    let number_start = find_unquoted_digit(pattern)
        .ok_or_else(|| format!("number pattern has no digit placeholder: `{pattern}`"))?;
    let number_end = number_skeleton_end(pattern, number_start);
    if pattern[..number_start].contains('*') || pattern[number_end..].contains('*') {
        return Err(format!(
            "number pattern uses unsupported padding syntax: `{pattern}`"
        ));
    }
    let prefix = unquote_affix(&pattern[..number_start])?;
    let suffix = unquote_affix(&pattern[number_end..])?;
    let number = &pattern[number_start..number_end];
    if number.contains(['@', 'E', '*'])
        || number
            .chars()
            .any(|character| matches!(character, '1'..='9'))
    {
        return Err(format!(
            "number pattern uses unsupported significant/scientific/padding/rounding syntax: `{pattern}`"
        ));
    }
    if number.matches('.').count() > 1 {
        return Err(format!("number pattern has multiple decimals: `{pattern}`"));
    }
    let (integer, fraction) = number.split_once('.').unwrap_or((number, ""));
    if integer.is_empty()
        || !integer
            .chars()
            .any(|character| matches!(character, '#' | '0'))
    {
        return Err(format!(
            "number pattern has no integer placeholders: `{pattern}`"
        ));
    }
    if integer
        .chars()
        .chain(fraction.chars())
        .any(|character| !matches!(character, '#' | '0' | ','))
    {
        return Err(format!(
            "number pattern has invalid skeleton syntax: `{pattern}`"
        ));
    }
    if fraction.contains(',') {
        return Err(format!(
            "number pattern groups fractional digits: `{pattern}`"
        ));
    }

    let group_sizes = integer
        .rsplit(',')
        .map(|group| {
            group
                .chars()
                .filter(|character| matches!(character, '#' | '0'))
                .count()
        })
        .collect::<Vec<_>>();

    Ok(NumberPatternPart {
        prefix,
        suffix,
        min_integer_digits: checked_u8(
            integer
                .chars()
                .filter(|character| *character == '0')
                .count(),
            "minimum integer digits",
            pattern,
        )?,
        min_fraction_digits: checked_u8(
            fraction
                .chars()
                .filter(|character| *character == '0')
                .count(),
            "minimum fraction digits",
            pattern,
        )?,
        max_fraction_digits: checked_u8(
            fraction
                .chars()
                .filter(|character| matches!(character, '#' | '0'))
                .count(),
            "maximum fraction digits",
            pattern,
        )?,
        primary_group_size: (group_sizes.len() > 1)
            .then(|| checked_u8(group_sizes[0], "primary group size", pattern))
            .transpose()?,
        secondary_group_size: (group_sizes.len() > 2)
            .then(|| checked_u8(group_sizes[1], "secondary group size", pattern))
            .transpose()?,
    })
}

fn split_unquoted(source: &str, separator: char) -> Result<Vec<&str>, String> {
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
        } else if character == separator && !quoted {
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

fn find_unquoted_digit(pattern: &str) -> Option<usize> {
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
        } else if !quoted && matches!(character, '#' | '0' | '@') {
            return Some(index);
        }
    }
    None
}

fn number_skeleton_end(pattern: &str, start: usize) -> usize {
    for (offset, character) in pattern[start..].char_indices() {
        if !matches!(
            character,
            '#' | '0'..='9' | '@' | ',' | '.' | 'E' | '+' | '*'
        ) {
            return start + offset;
        }
    }
    pattern.len()
}

fn unquote_affix(affix: &str) -> Result<String, String> {
    let mut result = String::with_capacity(affix.len());
    let mut quoted = false;
    let mut chars = affix.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\'' {
            result.push(character);
            continue;
        }
        if chars.peek() == Some(&'\'') {
            chars.next();
            result.push('\'');
        } else {
            quoted = !quoted;
        }
    }
    if quoted {
        Err(format!("unterminated quote in number affix `{affix}`"))
    } else {
        Ok(result)
    }
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
    use super::{
        extract_dates, extract_text_direction, parse_number_pattern, require_exact_exclusions,
    };
    use serde_json::json;

    #[test]
    fn number_pattern_parser_honors_quoted_affixes_and_subpatterns() {
        let pattern =
            parse_number_pattern("'pre;fix'#,##0.00' suffix';'negative'#,##0.00").expect("pattern");

        assert_eq!(pattern.positive.prefix, "pre;fix");
        assert_eq!(pattern.positive.suffix, " suffix");
        assert_eq!(pattern.positive.primary_group_size, Some(3));
        assert_eq!(pattern.positive.min_fraction_digits, 2);
        assert_eq!(
            pattern.negative.expect("negative subpattern").prefix,
            "negative"
        );
    }

    #[test]
    fn number_pattern_parser_rejects_unrepresentable_or_oversized_models() {
        assert!(parse_number_pattern("#E0").is_err());
        assert!(parse_number_pattern("@@@").is_err());
        assert!(parse_number_pattern("*x#,##0").is_err());
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
