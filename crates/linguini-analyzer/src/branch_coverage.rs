use crate::{Diagnostic, QuickFix, Replacement};
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
    let insertion = branch_insertion_span(&input.branches, input.span);
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
                .with_related(variant.span, "enum variant is declared here")
                .with_quick_fix(QuickFix::replacement(
                    format!("add branch `{}`", variant.name),
                    Replacement {
                        span: insertion,
                        text: format!("\n{} => TODO", variant.name),
                    },
                )),
            );
        }
    }

    diagnostics
}

pub fn require_other_branch(subject: &str, branches: &[NamedSpan], span: Span) -> Vec<Diagnostic> {
    if branches.iter().any(|branch| branch.name == "other") {
        Vec::new()
    } else {
        let insertion = branch_insertion_span(branches, span);
        vec![Diagnostic::error(
            format!("{subject} is missing required `other` branch"),
            span,
        )
        .with_quick_fix(QuickFix::replacement(
            "add `other` branch",
            Replacement {
                span: insertion,
                text: "\nother => TODO".to_owned(),
            },
        ))]
    }
}

fn branch_insertion_span(branches: &[NamedSpan], fallback: Span) -> Span {
    branches
        .iter()
        .map(|branch| Span::new(branch.span.end, branch.span.end))
        .max_by_key(|span| span.start)
        .unwrap_or(Span::new(fallback.end, fallback.end))
}
