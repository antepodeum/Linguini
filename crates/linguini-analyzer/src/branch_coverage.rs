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
    let mut diagnostics = validate_branch_sequence(&input.branches);
    let wildcard = input.branches.iter().position(|branch| branch.name == "_");
    let explicit_end = wildcard.unwrap_or(input.branches.len());
    let branch_names: BTreeSet<_> = input.branches[..explicit_end]
        .iter()
        .map(|branch| branch.name.as_str())
        .collect();
    let variants = input
        .variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect::<BTreeSet<_>>();
    let insertion = branch_insertion_span(&input.branches, input.span);

    for branch in &input.branches {
        if branch.name != "_" && !variants.contains(branch.name.as_str()) {
            diagnostics.push(
                Diagnostic::error(
                    format!(
                        "{} uses unknown variant `{}` for enum `{}`",
                        input.subject, branch.name, input.enum_name
                    ),
                    branch.span,
                )
                .with_code("linguini.unknown_enum_variant"),
            );
        }
    }

    if let Some(wildcard_index) = wildcard {
        if variants
            .iter()
            .all(|variant| branch_names.contains(variant))
        {
            diagnostics.push(
                Diagnostic::warning(
                    format!(
                        "{} has a redundant wildcard after covering every `{}` variant",
                        input.subject, input.enum_name
                    ),
                    input.branches[wildcard_index].span,
                )
                .as_lint("redundant_wildcard"),
            );
        }
        return diagnostics;
    }

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
                .with_code("linguini.incomplete_match")
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
    let mut diagnostics = validate_branch_sequence(branches);
    if branches
        .iter()
        .any(|branch| matches!(branch.name.as_str(), "other" | "_"))
    {
        diagnostics
    } else {
        let insertion = branch_insertion_span(branches, span);
        diagnostics.push(
            Diagnostic::error(
                format!("{subject} is missing required `other` branch"),
                span,
            )
            .with_code("linguini.incomplete_match")
            .with_quick_fix(QuickFix::replacement(
                "add `_` branch",
                Replacement {
                    span: insertion,
                    text: "\n_ => TODO".to_owned(),
                },
            )),
        );
        diagnostics
    }
}

pub(crate) fn validate_branch_sequence(branches: &[NamedSpan]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = std::collections::BTreeMap::<&str, Span>::new();
    let mut wildcard = None;

    for (index, branch) in branches.iter().enumerate() {
        if let Some(first) = seen.insert(branch.name.as_str(), branch.span) {
            diagnostics.push(
                Diagnostic::error(format!("duplicate branch `{}`", branch.name), branch.span)
                    .with_code("linguini.duplicate_branch")
                    .with_related(first, "first branch is here"),
            );
        }
        if branch.name == "_" && wildcard.is_none() {
            wildcard = Some(index);
        }
        if let Some(wildcard_index) = wildcard {
            if index > wildcard_index {
                diagnostics.push(
                    Diagnostic::warning(
                        format!("branch `{}` is unreachable after `_`", branch.name),
                        branch.span,
                    )
                    .as_lint("unreachable_arm")
                    .with_related(branches[wildcard_index].span, "wildcard is here"),
                );
            }
        }
    }

    if let Some(index) = wildcard {
        if index + 1 != branches.len() {
            diagnostics.push(
                Diagnostic::error("wildcard `_` must be the last branch", branches[index].span)
                    .with_code("linguini.wildcard_order"),
            );
        }
    }
    diagnostics
}

fn branch_insertion_span(branches: &[NamedSpan], fallback: Span) -> Span {
    branches
        .iter()
        .map(|branch| Span::new(branch.span.end, branch.span.end))
        .max_by_key(|span| span.start)
        .unwrap_or(Span::new(fallback.end, fallback.end))
}
