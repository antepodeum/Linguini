//! Resolve plural-dependent Gregorian week patterns during compilation.

use crate::{
    DatePatternError, DatePatternPlan, DatePlanError, GregorianWeekYearPlan, IntegerPluralPlan,
};
use linguini_cldr::{CalendarFormatData, PluralRules};
use std::collections::BTreeMap;

/// Exact `yw` and `MMMMW` families, specialized to reachable week numbers. Patterns and predicates
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
        Self::for_skeleton(locale, calendar, rules, "yw")
    }

    /// Resolve an exact year/week (`yw`) or month/week (`MMMMW`) family. Month-week
    /// bounds follow minimum days: week zero is impossible for one-day first weeks,
    /// and week six is impossible when the first week requires three or more days.
    pub fn for_skeleton(
        locale: &str,
        calendar: &CalendarFormatData,
        rules: &PluralRules,
        skeleton: &str,
    ) -> Result<Self, DatePlanError> {
        Self::compile(locale, calendar, Some(rules), skeleton)
    }

    fn compile(
        locale: &str,
        calendar: &CalendarFormatData,
        rules: Option<&PluralRules>,
        skeleton: &str,
    ) -> Result<Self, DatePlanError> {
        let missing = || {
            DatePlanError::Pattern(DatePatternError::MissingSkeleton {
                locale: locale.to_owned(),
                skeleton: skeleton.to_owned(),
            })
        };
        if calendar.calendar != "gregory" || !matches!(skeleton, "yw" | "MMMMW") {
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
        let fallback = lookup(&format!("{skeleton}-count-other"))
            .or_else(|| lookup(skeleton))
            .ok_or_else(missing)?;
        let (minimum, maximum) = if skeleton == "yw" {
            (1, 53)
        } else {
            (
                u64::from(week.anchor_offset == 6),
                u64::from((30 + week.anchor_offset) / 7 + 1),
            )
        };
        let Some(rules) = rules else {
            // Cardinal data cannot affect a family whose compiled variants are equal.
            // Prove that equality rather than selecting an assumed default category.
            let pattern = DatePatternPlan::for_week(locale, fallback, calendar)?;
            let prefix = format!("{skeleton}-count-");
            for &(name, source) in calendar.skeletons {
                if name.starts_with(&prefix)
                    && source != fallback
                    && DatePatternPlan::for_week(locale, source, calendar)? != pattern
                {
                    return Err(missing());
                }
            }
            return Ok(Self {
                week,
                patterns: vec![pattern],
                selection: IntegerPluralPlan {
                    minimum,
                    maximum,
                    branches: Vec::new(),
                    fallback: 0,
                },
            });
        };
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
            let source = lookup(&format!("{skeleton}-count-{category}")).unwrap_or(fallback);
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
            IntegerPluralPlan::for_rules(rules, minimum, maximum, |category| outcomes[category])
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

/// Resolve canonical `yw`/`MMMMW` requests without changing ordinary static date requests.
pub fn week_date_request_plan(
    locale: &str,
    request: &crate::FormatterRequest,
) -> Result<Option<WeekDatePlan>, DatePlanError> {
    if request.formatter.kind != linguini_ir::IrFormatterKind::Date
        || !matches!(request.argument("skeleton"), Some("yw" | "MMMMW"))
    {
        return Ok(None);
    }
    if request.argument("style").is_some() || request.argument("time_style").is_some() {
        return Err(DatePatternError::ConflictingSkeletonStyles.into());
    }
    let skeleton = request.argument("skeleton").expect("week skeleton");
    let missing = || {
        DatePlanError::Pattern(DatePatternError::MissingSkeleton {
            locale: locale.to_owned(),
            skeleton: skeleton.to_owned(),
        })
    };
    let calendar =
        linguini_cldr::compiled_calendar_formatting(locale, "gregory").ok_or_else(missing)?;
    let rules = linguini_cldr::built_in_plural_rules(locale);
    WeekDatePlan::compile(locale, &calendar, rules.as_ref(), skeleton).map(Some)
}

#[cfg(test)]
mod tests;
