//! Resolve selected date fields, symbols, and shared day periods before target emission.

use std::collections::BTreeSet;

use linguini_cldr::CalendarFormatData;

use crate::{
    can_localize_date_result, compile_date_pattern, date_field_symbols, day_period_plan,
    DateFieldSymbols, DatePatternError, DatePatternPart, DayPeriodPlan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatePlanError {
    Pattern(DatePatternError),
    MissingFieldSymbols {
        locale: String,
        symbol: char,
        width: usize,
    },
}

impl std::fmt::Display for DatePlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern(error) => write!(f, "{error}"),
            Self::MissingFieldSymbols {
                locale,
                symbol,
                width,
            } => {
                write!(
                    f,
                    "missing pinned date symbols for `{symbol}` with width {width} in `{locale}`"
                )
            }
        }
    }
}

impl std::error::Error for DatePlanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Pattern(error) => Some(error),
            Self::MissingFieldSymbols { .. } => None,
        }
    }
}

impl From<DatePatternError> for DatePlanError {
    fn from(error: DatePatternError) -> Self {
        Self::Pattern(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFieldPlan {
    pub symbol: char,
    pub width: usize,
    pub symbols: DateFieldSymbols,
    /// Index into the containing pattern's deduplicated day-period plans.
    pub period: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFormatPart {
    Literal(String),
    Field(DateFieldPlan),
}

/// Selected symbol values and semantic field demand, without target bindings or syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatePatternPlan {
    pub parts: Vec<DateFormatPart>,
    pub periods: Vec<DayPeriodPlan>,
    pub fields: BTreeSet<char>,
    pub can_localize_whole_result: bool,
}

/// Shared field evaluation around one literal choice. Variants must differ only in text;
/// fields keep their original order, and digit-localization proofs include every alternative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateLiteralChoice {
    pub prefix: Vec<DateFormatPart>,
    pub values: Vec<String>,
    pub suffix: Vec<DateFormatPart>,
    pub can_localize_whole_result: bool,
}

impl DateLiteralChoice {
    pub fn for_patterns(patterns: &[DatePatternPlan]) -> Option<Self> {
        let first = patterns.first()?;
        if patterns.len() < 2
            || patterns.iter().all(|pattern| pattern.parts == first.parts)
            || patterns
                .iter()
                .any(|pattern| pattern.periods != first.periods)
        {
            return None;
        }
        let shortest = patterns.iter().map(|pattern| pattern.parts.len()).min()?;
        let prefix = (0..shortest)
            .take_while(|&index| {
                patterns
                    .iter()
                    .all(|pattern| pattern.parts[index] == first.parts[index])
            })
            .count();
        let suffix = (0..shortest - prefix)
            .take_while(|&index| {
                patterns.iter().all(|pattern| {
                    pattern.parts[pattern.parts.len() - 1 - index]
                        == first.parts[first.parts.len() - 1 - index]
                })
            })
            .count();
        let mut values = Vec::with_capacity(patterns.len());
        for pattern in patterns {
            let mut value = String::new();
            for part in &pattern.parts[prefix..pattern.parts.len() - suffix] {
                match part {
                    DateFormatPart::Literal(text) => value.push_str(text),
                    DateFormatPart::Field(_) => return None,
                }
            }
            values.push(value);
        }
        Some(Self {
            prefix: first.parts[..prefix].to_vec(),
            values,
            suffix: first.parts[first.parts.len() - suffix..].to_vec(),
            can_localize_whole_result: patterns
                .iter()
                .all(|pattern| pattern.can_localize_whole_result),
        })
    }
}

impl DatePatternPlan {
    pub fn new(
        locale: &str,
        pattern: &str,
        calendar: &CalendarFormatData,
    ) -> Result<Self, DatePlanError> {
        Self::from_parts(locale, compile_date_pattern(locale, pattern)?, calendar)
    }

    pub(crate) fn for_week(
        locale: &str,
        pattern: &str,
        calendar: &CalendarFormatData,
    ) -> Result<Self, DatePlanError> {
        Self::from_parts(
            locale,
            crate::date_pattern::compile_week_pattern(locale, pattern)?,
            calendar,
        )
    }

    fn from_parts(
        locale: &str,
        parsed: Vec<DatePatternPart>,
        calendar: &CalendarFormatData,
    ) -> Result<Self, DatePlanError> {
        let mut parts = Vec::with_capacity(parsed.len());
        let mut periods = Vec::new();
        let mut fields = BTreeSet::new();
        let mut textual_symbols = Vec::new();
        for part in &parsed {
            let (symbol, width) = match part {
                DatePatternPart::Literal(value) => {
                    parts.push(DateFormatPart::Literal(value.clone()));
                    continue;
                }
                DatePatternPart::Field { symbol, width } => (*symbol, *width),
            };
            fields.insert(symbol);
            let symbols = date_field_symbols(calendar, symbol, width).ok_or_else(|| {
                DatePlanError::MissingFieldSymbols {
                    locale: locale.to_owned(),
                    symbol,
                    width,
                }
            })?;
            textual_symbols.extend(
                [symbols.am, symbols.pm, symbols.ad, symbols.bc]
                    .into_iter()
                    .flatten(),
            );
            if let Some(names) = &symbols.names {
                textual_symbols.extend(names.iter().copied());
            }
            let period = day_period_plan(calendar, symbol, width).map(|plan| {
                if let Some(index) = periods.iter().position(|existing| existing == &plan) {
                    return index;
                }
                textual_symbols.extend(plan.spans.iter().map(|span| span.value));
                textual_symbols.extend(plan.exact.iter().map(|period| period.value));
                periods.push(plan);
                periods.len() - 1
            });
            parts.push(DateFormatPart::Field(DateFieldPlan {
                symbol,
                width,
                symbols,
                period,
            }));
        }
        Ok(Self {
            can_localize_whole_result: can_localize_date_result(&parsed, textual_symbols),
            parts,
            periods,
            fields,
        })
    }
}

#[cfg(test)]
mod tests;
