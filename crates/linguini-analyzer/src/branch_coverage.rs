use crate::Diagnostic;
use linguini_syntax::Span;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedSpan {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchCoverage<'a> {
    pub subject: &'a str,
    pub enum_name: &'a str,
    pub variants: Vec<NamedSpan>,
    pub branches: Vec<NamedSpan>,
    pub span: Span,
}

impl NamedSpan {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

pub fn analyze_branch_coverage(input: BranchCoverage<'_>) -> Vec<Diagnostic> {
    let branch_names: BTreeSet<_> = input
        .branches
        .iter()
        .map(|branch| branch.name.as_str())
        .collect();
    let mut diagnostics = Vec::new();

    for variant in input.variants {
        if !branch_names.contains(variant.name.as_str()) {
            diagnostics.push(
                Diagnostic::error(
                    format!(
                        "{} for enum `{}` is missing branch `{}`",
                        input.subject, input.enum_name, variant.name
                    ),
                    input.span,
                )
                .with_related(variant.span, "enum variant is declared here"),
            );
        }
    }

    diagnostics
}

pub fn require_other_branch(subject: &str, branches: &[NamedSpan], span: Span) -> Vec<Diagnostic> {
    if branches.iter().any(|branch| branch.name == "other") {
        Vec::new()
    } else {
        vec![Diagnostic::error(
            format!("{subject} is missing required `other` branch"),
            span,
        )]
    }
}
