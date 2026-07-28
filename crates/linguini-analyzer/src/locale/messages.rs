use super::{ImplementedLocaleMessage, RequiredLocaleMessage};
use std::collections::BTreeMap;

pub(super) fn schema_message_map(
    messages: &[RequiredLocaleMessage],
) -> BTreeMap<&str, &RequiredLocaleMessage> {
    messages
        .iter()
        .map(|message| (message.name.as_str(), message))
        .collect()
}

pub(super) fn locale_message_map(
    messages: &[ImplementedLocaleMessage],
) -> BTreeMap<&str, &ImplementedLocaleMessage> {
    messages
        .iter()
        .map(|message| (message.name.as_str(), message))
        .collect()
}

pub(super) fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

pub(super) fn format_name_list(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn missing_message_stub_text(names: &[&str]) -> String {
    let mut root = StubNode::default();
    for name in names {
        let mut node = &mut root;
        for segment in name.split('.') {
            node = node.children.entry(segment).or_default();
        }
        node.message = true;
    }

    let mut output = String::new();
    if !root.children.is_empty() {
        output.push('\n');
        render_stub_children(&root, 0, &mut output);
    }
    output
}

#[derive(Default)]
struct StubNode<'a> {
    message: bool,
    children: BTreeMap<&'a str, StubNode<'a>>,
}

fn render_stub_children(node: &StubNode<'_>, indent: usize, output: &mut String) {
    let prefix = "  ".repeat(indent);
    for (name, child) in &node.children {
        if child.message {
            output.push_str(&format!("{prefix}{name} = TODO\n"));
        }
        if !child.children.is_empty() {
            output.push_str(&format!("{prefix}{name} {{\n"));
            render_stub_children(child, indent + 1, output);
            output.push_str(&format!("{prefix}}}\n"));
        }
    }
}
