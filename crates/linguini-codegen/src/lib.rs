//! Backend-independent models and semantic planning for code generators.
//!
//! Target backends own syntax, identifier escaping, runtime templates, and file emission.

mod date_symbols;
mod day_periods;
mod decision;
mod dependencies;
mod direct_dependencies;
mod formatters;
mod inline;
mod paths;
mod plurals;
mod requirements;
mod rounding;
mod types;

pub use date_symbols::{date_field_symbols, DateFieldSymbols};
pub use day_periods::{day_period_plan, DayPeriodPlan, DayPeriodSpan, ExactDayPeriod};
pub use decision::{Decision, DecisionArm};
pub use dependencies::{
    message_dependency_closure, MessageDependencyClosure, MessageDependencyError,
};
pub use direct_dependencies::{
    direct_message_dependencies, direct_symbol_dependencies, SemanticSymbolKind,
};
pub use requirements::{
    expression_has_string_result, formatter_requirements, module_uses_inline_functions,
    plural_required, FormatterRequirements,
};
pub use types::{resolve_type_name, TypeModel};

pub use formatters::{
    default_type_formatters, formatter_requests, parameter_formatters, FormatterRequest,
};

pub use inline::lower_inline_functions;

pub use paths::lexical_relative_path;
pub use plurals::{PluralDecisionPlan, PluralRequirements};
pub use rounding::NumberRoundingPlan;
