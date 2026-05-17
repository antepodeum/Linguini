mod branch_coverage;
mod diagnostic;
mod expression;
mod locale;
mod message_coverage;
mod reference;

pub use branch_coverage::{
    analyze_branch_coverage, require_other_branch, BranchCoverage, NamedSpan,
};
pub use diagnostic::{
    render_diagnostics, render_diagnostics_with_color, Diagnostic, DiagnosticSeverity, QuickFix,
    RelatedSpan, RenderError, Replacement,
};
pub use expression::{
    analyze_expressions, analyze_function_patterns, ExpressionAnalysis, FormProperty,
    FormSignature, FunctionSignature, MessageToAnalyze, Variable,
};
pub use locale::{
    analyze_locale_coverage, analyze_locale_coverage_with_options, analyze_locale_file,
    analyze_locale_message_coverage, analyze_locale_message_coverage_with_options,
    locale_public_messages, schema_public_messages, ImplementedLocaleMessage,
    LocaleCoverageOptions, RequiredLocaleMessage,
};
pub use message_coverage::{analyze_message_coverage, PublicMessage};
pub use reference::{detect_reference_cycles, ReferenceNode};

pub const CRATE_PURPOSE: &str = "semantic analysis";

#[cfg(test)]
mod tests;
