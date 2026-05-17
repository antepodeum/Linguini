use std::collections::BTreeMap;

use linguini_ir::{IrMessage, IrModule};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageTree {
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
    for message in &module.messages {
        let parts = message.name.split('.').collect::<Vec<_>>();
        if parts.len() > 1 {
            insert_message(&mut root, &parts, message.clone());
        }
    }
    root
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
