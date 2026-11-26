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

impl DatePatternPlan {
    pub fn new(
        locale: &str,
        pattern: &str,
        calendar: &CalendarFormatData,
    ) -> Result<Self, DatePlanError> {
        let parsed = compile_date_pattern(locale, pattern)?;
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
