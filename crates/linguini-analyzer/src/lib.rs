mod application;
mod branch_coverage;
mod diagnostic;
mod expression;
mod locale;
mod message_coverage;
mod reference;

pub use application::{
    analyze_unused_messages, ApplicationBinding, ApplicationBindingProvenance,
    ApplicationDynamicReference, ApplicationDynamicReferenceKind, ApplicationImportBinding,
    ApplicationImportBindingId, ApplicationReference, ApplicationReferenceKind, ApplicationUsage,
};
pub use branch_coverage::{
    analyze_branch_coverage, require_other_branch, BranchCoverage, NamedSpan,
};
pub use diagnostic::{
    render_diagnostics, render_diagnostics_with_color, Diagnostic, DiagnosticCategory,
    DiagnosticSeverity, QuickFix, QuickFixAction, RelatedSpan, RenderError, Replacement,
};
pub use expression::{
    analyze_expressions, analyze_function_patterns, analyze_project_expressions,
    ExpressionAnalysis, FormProperty, FormSignature, FunctionSignature, MessageToAnalyze, Variable,
};
pub use locale::{
    analyze_locale_coverage, analyze_locale_coverage_with_options, analyze_locale_file,
    analyze_locale_message_coverage, analyze_locale_message_coverage_with_options,
    locale_public_messages, schema_public_messages, ImplementedLocaleMessage,
    LocaleCoverageOptions, RequiredLocaleMessage,
};
pub use message_coverage::{analyze_message_coverage, PublicMessage};
pub use reference::{detect_reference_cycles, ReferenceNode};

#[cfg(test)]
mod tests;
