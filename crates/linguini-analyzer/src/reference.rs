use crate::{Diagnostic, NamedSpan};
use linguini_syntax::Span;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

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
    let mut diagnostics = Vec::new();
    let mut graph = BTreeMap::<&str, &ReferenceNode>::new();
    for node in nodes {
        match graph.entry(node.name.as_str()) {
            Entry::Vacant(entry) => {
                entry.insert(node);
            }
            Entry::Occupied(first) => diagnostics.push(
                Diagnostic::error(
                    format!("duplicate reference graph node `{}`", node.name),
                    node.span,
                )
                .with_code("linguini.duplicate_reference_node")
                .with_related(first.get().span, "first graph node is here"),
            ),
        }
    }

    let mut visited = BTreeSet::new();
    let mut finish = Vec::new();
    for name in graph.keys() {
        finish_order(name, &graph, &mut visited, &mut finish);
    }

    let mut reverse = graph
        .keys()
        .map(|name| (*name, Vec::<&str>::new()))
        .collect::<BTreeMap<_, _>>();
    for (source, node) in &graph {
        for reference in &node.references {
            if let Some(incoming) = reverse.get_mut(reference.name.as_str()) {
                incoming.push(source);
            }
        }
    }

    visited.clear();
    while let Some(name) = finish.pop() {
        if visited.contains(name) {
            continue;
        }
        let mut component = Vec::new();
        collect_component(name, &reverse, &mut visited, &mut component);
        component.sort_unstable();
        let self_cycle = component.len() == 1
            && graph[component[0]]
                .references
                .iter()
                .any(|reference| reference.name == component[0]);
        if component.len() == 1 && !self_cycle {
            continue;
        }

        let mut cycle = component.clone();
        cycle.push(component[0]);
        let component_names = component.iter().copied().collect::<BTreeSet<_>>();
        let mut diagnostic = Diagnostic::error(
            format!("cyclic reference `{}`", cycle.join(" -> ")),
            graph[component[0]].span,
        )
        .with_code("linguini.reference_cycle");
        for source in &component {
            for reference in &graph[source].references {
                if component_names.contains(reference.name.as_str()) {
                    diagnostic = diagnostic.with_related(
                        reference.span,
                        format!("`{source}` references `{}`", reference.name),
                    );
                }
            }
        }
        diagnostics.push(diagnostic);
    }
    diagnostics
}

fn finish_order<'a>(
    name: &'a str,
    graph: &BTreeMap<&'a str, &'a ReferenceNode>,
    visited: &mut BTreeSet<&'a str>,
    finish: &mut Vec<&'a str>,
) {
    if !visited.insert(name) {
        return;
    }
    if let Some(node) = graph.get(name) {
        for reference in &node.references {
            if let Some((target, _)) = graph.get_key_value(reference.name.as_str()) {
                finish_order(target, graph, visited, finish);
            }
        }
    }
    finish.push(name);
}

fn collect_component<'a>(
    name: &'a str,
    graph: &BTreeMap<&'a str, Vec<&'a str>>,
    visited: &mut BTreeSet<&'a str>,
    output: &mut Vec<&'a str>,
) {
    if !visited.insert(name) {
        return;
    }
    output.push(name);
    if let Some(sources) = graph.get(name) {
        for source in sources {
            collect_component(source, graph, visited, output);
        }
    }
}
