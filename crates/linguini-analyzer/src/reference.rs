use crate::{Diagnostic, NamedSpan};
use linguini_syntax::Span;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceNode {
    pub name: String,
    pub references: Vec<NamedSpan>,
    pub span: Span,
}

impl ReferenceNode {
    pub fn new(name: impl Into<String>, references: Vec<NamedSpan>, span: Span) -> Self {
        Self {
            name: name.into(),
            references,
            span,
        }
    }
}

pub fn detect_reference_cycles(nodes: &[ReferenceNode]) -> Vec<Diagnostic> {
    let graph: BTreeMap<_, _> = nodes
        .iter()
        .map(|node| (node.name.as_str(), node.references.as_slice()))
        .collect();
    let spans: BTreeMap<_, _> = nodes
        .iter()
        .map(|node| (node.name.as_str(), node.span))
        .collect();
    let mut diagnostics = Vec::new();
    let mut reported = BTreeSet::new();

    for node in nodes {
        let mut path = Vec::new();
        visit_cycle(
            node.name.as_str(),
            node.name.as_str(),
            &graph,
            &spans,
            &mut path,
            &mut reported,
            &mut diagnostics,
        );
    }

    diagnostics
}

fn visit_cycle(
    start: &str,
    current: &str,
    graph: &BTreeMap<&str, &[NamedSpan]>,
    spans: &BTreeMap<&str, Span>,
    path: &mut Vec<String>,
    reported: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if path.iter().any(|item| item == current) {
        if current == start && reported.insert(start.to_owned()) {
            let mut cycle = path.clone();
            cycle.push(start.to_owned());
            let span = spans.get(start).copied().unwrap_or(Span::new(0, 0));
            diagnostics.push(Diagnostic::error(
                format!("cyclic reference `{}`", cycle.join(" -> ")),
                span,
            ));
        }
        return;
    }

    path.push(current.to_owned());
    if let Some(references) = graph.get(current) {
        for reference in *references {
            if graph.contains_key(reference.name.as_str()) {
                visit_cycle(
                    start,
                    reference.name.as_str(),
                    graph,
                    spans,
                    path,
                    reported,
                    diagnostics,
                );
            }
        }
    }
    path.pop();
}
