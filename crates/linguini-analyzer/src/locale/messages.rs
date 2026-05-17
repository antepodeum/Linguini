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
    let mut output = String::new();
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut top_level = Vec::new();

    for name in names {
        if let Some((group, message)) = name.split_once('.') {
            groups.entry(group).or_default().push(message);
        } else {
            top_level.push(*name);
        }
    }

    if !top_level.is_empty() || !groups.is_empty() {
        output.push('\n');
    }

    for name in top_level {
        output.push_str(&format!("{name} = TODO\n"));
    }

    for (group, messages) in groups {
        output.push_str(&format!("{group} {{\n"));
        for message in messages {
            output.push_str(&format!("  {message} = TODO\n"));
        }
        output.push_str("}\n");
    }

    output
}
