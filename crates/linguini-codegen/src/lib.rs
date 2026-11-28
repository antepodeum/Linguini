//! Backend-independent models and semantic planning for code generators.
//!
//! Target backends own syntax, identifier escaping, runtime templates, and file emission.

mod date_pattern;
mod date_plan;
mod date_symbols;
mod day_periods;
mod decision;
mod dependencies;
mod digits;
mod direct_dependencies;
mod dispatch;
mod form_demand;
mod formatters;
mod inline;
mod integer_plurals;
mod locales;
mod message_tree;
mod numbers;
mod paths;
mod plurals;
mod projections;
mod provenance;
mod requirements;
mod rounding;
mod types;
mod week_date;
mod week_year;

pub use date_pattern::{
    compile_date_pattern, date_request_pattern, DatePatternError, DatePatternPart,
};
pub use date_plan::{
    DateFieldPlan, DateFormatPart, DateLiteralChoice, DatePatternPlan, DatePlanError,
};
pub use date_symbols::{date_field_symbols, DateFieldSymbols};
pub use day_periods::{day_period_plan, DayPeriodPlan, DayPeriodSpan, ExactDayPeriod};
pub use decision::{Decision, DecisionArm};
pub use dependencies::{
    message_dependency_closure, prune_unused_form_attributes, MessageDependencyClosure,
    MessageDependencyError,
};
pub use digits::{can_localize_date_result, contiguous_decimal_digit_zero, decimal_digit_symbols};
pub use direct_dependencies::{
    direct_message_dependencies, direct_symbol_dependencies, SemanticSymbolKind,
};
pub use dispatch::{form_dispatch_plan, function_dispatch_plan};
pub use form_demand::independent_form_attribute_names;
pub use requirements::{
    expression_has_string_result, formatter_requirements, module_uses_inline_functions,
    plural_required, FormatterRequirements, StringResultFacts,
};
pub use types::{expression_value_type, resolve_type_name, TypeModel};
pub use week_date::{week_date_request_plan, WeekDatePlan};
pub use week_year::GregorianWeekYearPlan;

pub use formatters::{
    default_type_formatters, formatter_requests, parameter_formatters, FormatterRequest,
};

pub use inline::lower_inline_functions;
pub use integer_plurals::{IntegerPluralBranch, IntegerPluralPlan};
pub use locales::{compose_locale_modules, ProjectLocalePlan};
pub use numbers::{
    NumberAffixPlan, NumberFormatPlan, NumberGroupingPlan, NumberPrecisionPlan,
    NumberScientificPlan,
};

pub use paths::lexical_relative_path;
pub use plurals::{PluralDecisionPlan, PluralRequirements};
pub use rounding::NumberRoundingPlan;

pub use message_tree::{nested_message_tree, MessageTree, MessageTreeMessage};
pub use projections::{
    locale_module_for_schema, namespace_module, root_module, select_schema_paths,
    top_level_message_namespaces,
};

pub use provenance::{semantic_symbol_source_ids, semantic_symbol_span};
