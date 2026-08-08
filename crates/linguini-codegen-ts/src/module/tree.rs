use std::collections::BTreeMap;

use linguini_ir::{IrMessage, IrModule};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageTree {
    /// Documentation attached to the declared namespace represented by this node.
    ///
    /// The root node has no declaration and therefore normally has no docs. Nodes that are
    /// inferred from dotted message paths keep an empty docs vector; explicit schema groups
    /// populate it from their `IrGroup` declaration.
    pub docs: Vec<String>,
    pub messages: Vec<MessageTreeMessage>,
    pub children: BTreeMap<String, MessageTree>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageTreeMessage {
    pub property: String,
    pub signature: IrMessage,
}

pub fn nested_message_tree(module: &IrModule) -> MessageTree {
    let mut root = MessageTree::default();
    for group in &module.groups {
        let parts = group.name.split('.').collect::<Vec<_>>();
        insert_group(&mut root, &parts, &group.docs);
    }
    for message in &module.messages {
        let parts = message.name.split('.').collect::<Vec<_>>();
        if parts.len() > 1 {
            insert_message(&mut root, &parts, message.clone());
        }
    }
    // Explicit empty groups are namespace reservations, not runtime values. Pruning them here
    // keeps both generated locale modules and schema-owned declarations from claiming leaves
    // that cannot be read from a locale module.
    retain_message_groups(&mut root);
    root
}

fn insert_group(node: &mut MessageTree, parts: &[&str], docs: &[String]) {
    match parts {
        [] => {}
        [group] => {
            let child = node.children.entry((*group).to_owned()).or_default();
            if !docs.is_empty() {
                child.docs = docs.to_owned();
            }
        }
        [group, rest @ ..] => {
            insert_group(
                node.children.entry((*group).to_owned()).or_default(),
                rest,
                docs,
            );
        }
    }
}

fn insert_message(node: &mut MessageTree, parts: &[&str], signature: IrMessage) {
    match parts {
        [] => {}
        [property] => node.messages.push(MessageTreeMessage {
            property: (*property).to_owned(),
            signature,
        }),
        [namespace, rest @ ..] => {
            insert_message(
                node.children.entry((*namespace).to_owned()).or_default(),
                rest,
                signature,
            );
        }
    }
}

fn retain_message_groups(node: &mut MessageTree) -> bool {
    node.children
        .retain(|_, child| retain_message_groups(child));
    !node.messages.is_empty() || !node.children.is_empty()
}
