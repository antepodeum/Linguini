//! Input-shape proofs shared by every caller of a generated precision helper.

use std::collections::BTreeMap;

use minijinja::{context, Value};

pub(super) struct PrecisionHelpers {
    pub rounding: Vec<Value>,
    pub padding: Vec<Value>,
    pub variable_rounding: bool,
    pub fraction_bounded: bool,
}

impl PrecisionHelpers {
    pub fn for_plans(plans: &[Value], maximum_digits: usize) -> Self {
        let rounding = groups(plans, "max_significant");
        let fixed = rounding.len() == 1
            && rounding[0].1 == Some(1)
            && (1..=maximum_digits).contains(&digits(&rounding[0].0, "max_significant"));
        let variable_rounding = !fixed && !rounding.is_empty();
        let fraction_bounded = rounding.iter().all(|(plan, width)| {
            width.is_some() && digits(plan, "max_significant") <= maximum_digits
        });
        Self {
            rounding: rounding.into_iter().map(|(plan, integer_width)| {
                let maximum = digits(&plan, "max_significant");
                context! { plan, integer_width, fixed_fraction => fixed.then_some(maximum.saturating_sub(1)), name => format!("roundTo{maximum}SignificantDigits") }
            }).collect(),
            padding: groups(plans, "min_significant").into_iter().map(|(plan, integer_width)| {
                context! { plan, normalized => integer_width.is_some() }
            }).collect(),
            variable_rounding,
            fraction_bounded,
        }
    }
}

/// A non-normalized caller invalidates the proof for the entire shared helper.
/// String keys retain the established helper ordering.
fn groups(plans: &[Value], key: &str) -> Vec<(Value, Option<usize>)> {
    let mut groups = BTreeMap::<String, (Value, Option<usize>)>::new();
    for plan in plans {
        let precision = plan.get_attr(key).expect("compiled precision field");
        if precision.is_none()
            || (key == "min_significant"
                && !plan.get_attr("fixed_significant_zero").unwrap().is_none())
        {
            continue;
        }
        let width = if !plan.get_attr("exponent_digits").unwrap().is_none()
            && !plan.get_attr("max_significant").unwrap().is_none()
        {
            Some(digits(plan, "scientific_carry_width"))
        } else {
            None
        };
        let entry = groups
            .entry(precision.to_string())
            .or_insert_with(|| (plan.clone(), Some(0)));
        entry.1 = entry
            .1
            .zip(width)
            .map(|(previous, current)| previous.max(current));
    }
    groups.into_values().collect()
}

fn digits(plan: &Value, key: &str) -> usize {
    plan.get_attr(key)
        .expect("compiled digit field")
        .as_usize()
        .expect("nonnegative digit count")
}
