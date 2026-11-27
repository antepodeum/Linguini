//! Resolve plural-dependent Gregorian year/week patterns during compilation.

use crate::{
    DatePatternError, DatePatternPlan, DatePlanError, GregorianWeekYearPlan, IntegerPluralPlan,
};
use linguini_cldr::{CalendarFormatData, PluralRules};
use std::collections::BTreeMap;

/// The exact `yw` family, specialized to week numbers 1–53. Patterns and numeric predicates
/// contain only live outcomes; backends never need a CLDR category/pattern interpreter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekDatePlan {
    pub week: GregorianWeekYearPlan,
    pub patterns: Vec<DatePatternPlan>,
    pub selection: IntegerPluralPlan<usize>,
}

impl WeekDatePlan {
    /// Resolve category variants with `other` fallback, sharing semantically equal patterns.
    /// An incomplete or invalid family reports the unavailable `yw` request.
    pub fn new(
        locale: &str,
        calendar: &CalendarFormatData,
        rules: &PluralRules,
    ) -> Result<Self, DatePlanError> {
        let missing = || {
            DatePlanError::Pattern(DatePatternError::MissingSkeleton {
                locale: locale.to_owned(),
                skeleton: "yw".to_owned(),
            })
        };
        if calendar.calendar != "gregory" {
            return Err(missing());
        }
        let week = GregorianWeekYearPlan::for_locale(locale).ok_or_else(|| {
            DatePlanError::Pattern(DatePatternError::MissingWeekData {
                locale: locale.to_owned(),
            })
        })?;
        let lookup = |name: &str| {
            calendar
                .skeletons
                .iter()
                .find(|(candidate, _)| *candidate == name)
                .map(|(_, pattern)| *pattern)
        };
        let fallback = lookup("yw-count-other")
            .or_else(|| lookup("yw"))
            .ok_or_else(missing)?;
        let mut patterns = Vec::new();
        let mut outcomes = BTreeMap::new();
        for category in rules
            .categories
            .iter()
            .map(|category| category.category.as_str())
            .chain(std::iter::once("other"))
        {
            if outcomes.contains_key(category) {
                continue;
            }
            let source = lookup(&format!("yw-count-{category}")).unwrap_or(fallback);
            let pattern = DatePatternPlan::for_week(locale, source, calendar)?;
            let index = patterns
                .iter()
                .position(|existing| existing == &pattern)
                .unwrap_or_else(|| {
                    patterns.push(pattern);
                    patterns.len() - 1
                });
            outcomes.insert(category, index);
        }
        let mut selection =
            IntegerPluralPlan::for_rules(rules, 1, 53, |category| outcomes[category])
                .ok_or_else(missing)?;
        let mut live = selection
            .branches
            .iter()
            .map(|branch| branch.result)
            .chain(std::iter::once(selection.fallback))
            .collect::<Vec<_>>();
        live.sort_unstable();
        live.dedup();
        let mut retained = Vec::with_capacity(live.len());
        for (index, pattern) in patterns.into_iter().enumerate() {
            if live.binary_search(&index).is_ok() {
                retained.push(pattern);
            }
        }
        for branch in &mut selection.branches {
            branch.result = live
                .binary_search(&branch.result)
                .expect("live week pattern");
        }
        selection.fallback = live
            .binary_search(&selection.fallback)
            .expect("live fallback pattern");
        Ok(Self {
            week,
            patterns: retained,
            selection,
        })
    }
}

/// Resolve a canonical `yw` request without changing ordinary static date requests.
pub fn week_date_request_plan(
    locale: &str,
    request: &crate::FormatterRequest,
) -> Result<Option<WeekDatePlan>, DatePlanError> {
    if request.formatter.kind != linguini_ir::IrFormatterKind::Date
        || request.argument("skeleton") != Some("yw")
    {
        return Ok(None);
    }
    if request.argument("style").is_some() || request.argument("time_style").is_some() {
        return Err(DatePatternError::ConflictingSkeletonStyles.into());
    }
    let missing = || {
        DatePlanError::Pattern(DatePatternError::MissingSkeleton {
            locale: locale.to_owned(),
            skeleton: "yw".to_owned(),
        })
    };
    let calendar =
        linguini_cldr::compiled_calendar_formatting(locale, "gregory").ok_or_else(missing)?;
    let rules = linguini_cldr::built_in_plural_rules(locale).ok_or_else(missing)?;
    WeekDatePlan::new(locale, &calendar, &rules).map(Some)
}

#[cfg(test)]
mod tests;
