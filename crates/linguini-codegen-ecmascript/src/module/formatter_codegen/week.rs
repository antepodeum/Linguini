//! Target projection of shared finite Gregorian week-format plans.

use super::{CompiledDatePattern, DateHelperRequirements, DateTables};
use minijinja::context;

pub(super) fn generate(
    locale: &str,
    dates: &linguini_cldr::DateFormatData,
    request: &super::super::formatters::FormatterRequest,
    name: &str,
    exported: bool,
    shared_tables: &mut DateTables,
) -> (String, DateHelperRequirements) {
    let plan = linguini_codegen::week_date_request_plan(locale, request)
        .expect("validated Gregorian week request")
        .expect("week request");
    let mut helpers = DateHelperRequirements::default();
    let mut patterns = Vec::new();
    for pattern in &plan.patterns {
        let compiled = super::project_date_parts(pattern, dates, name, locale, shared_tables);
        helpers.padding |= compiled.helpers.padding;
        helpers.digits |= compiled.helpers.digits;
        patterns.push(compiled);
    }
    let pattern_context = |pattern: &CompiledDatePattern| {
        context! {
            parts => &pattern.parts, tables => &pattern.tables, periods => &pattern.periods,
            clock => &pattern.clock, localize_result => pattern.localize_result,
        }
    };
    let mut choice = linguini_codegen::DateLiteralChoice::for_patterns(&plan.patterns);
    // Keep first-use remainder evaluation when a later branch introduces new arithmetic.
    let moduli_for = |rule: &linguini_cldr::PluralRule| {
        rule.conditions
            .iter()
            .flat_map(|condition| &condition.relations)
            .filter_map(|relation| relation.expression.modulo)
            .collect::<std::collections::BTreeSet<_>>()
    };
    let first_moduli = plan
        .selection
        .branches
        .first()
        .map(|branch| moduli_for(&branch.rule))
        .unwrap_or_default();
    if plan
        .selection
        .branches
        .iter()
        .any(|branch| !moduli_for(&branch.rule).is_subset(&first_moduli))
    {
        choice = None;
    }
    let mut defined = std::collections::BTreeSet::new();
    let branches = plan.selection.branches.iter().map(|branch| {
        let mut moduli = Vec::new();
        let conditions = branch.rule.conditions.iter().map(|condition| {
            condition.relations.iter().map(|relation| {
                let modulo_name = relation.expression.modulo.map(|modulus| {
                    let name = format!("weekNumberModulo{modulus}");
                    if defined.insert(modulus) {
                        moduli.push(context! { name => &name, binding => "weekNumber", modulo => modulus, bigint => false });
                    }
                    name
                });
                crate::plural::integer_relation_context(relation, "weekNumber", modulo_name)
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        context! { conditions, moduli, pattern => pattern_context(&patterns[branch.result]),
            value => choice.as_ref().map(|choice| &choice.values[branch.result]) }
    }).collect::<Vec<_>>();
    let factored = choice.as_ref().map(|choice| {
        let base = &patterns[0];
        let mut parts = base.parts[..choice.prefix.len()].to_vec();
        parts.push(context! { kind => "choice", branches => &branches,
        fallback => &choice.values[plan.selection.fallback] });
        parts.extend_from_slice(&base.parts[base.parts.len() - choice.suffix.len()..]);
        context! { parts, clock => &base.clock, periods => &base.periods,
        localize_result => base.helpers.digits && choice.can_localize_whole_result }
    });
    let output = crate::templates::render(
        "formatters/week_date",
        context! {
            name, exported, branches, factored, month_week => request.argument("skeleton") == Some("MMMMW"),
            month_anchor_offset => i16::from(plan.week.anchor_offset) - 1, patterns => patterns.iter().map(pattern_context).collect::<Vec<_>>(),
            fallback => pattern_context(&patterns[plan.selection.fallback]), digits => dates.digits,
            week_year => plan.patterns.iter().any(|pattern| pattern.fields.contains(&'Y')),
            previous => plan.week.previous_last_day.is_some(), next => plan.week.next_first_day.is_some(),
            anchor_offset => plan.week.anchor_offset, weekday_shift => plan.week.weekday_shift,
        },
    );
    (output, helpers)
}
