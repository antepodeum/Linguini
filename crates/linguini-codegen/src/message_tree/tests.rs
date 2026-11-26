use super::*;
use linguini_ir::lower_schema_typed;
use linguini_syntax::{parse_schema_in, SourceId};

#[test]
fn nested_tree_preserves_docs_message_order_and_source_identity_without_empty_values() {
    let schema = lower_schema_typed(
        &parse_schema_in(
            concat!(
                "root\n/// Top docs\ntop {\n second\n first\n",
                "/// Nested docs\n nested { leaf }\n empty {}\n}\nreserved { child {} }\n",
            ),
            SourceId(13),
        )
        .unwrap(),
    );
    let tree = nested_message_tree(&schema);
    assert!(
        tree.messages.is_empty(),
        "root messages are emitted independently"
    );
    assert_eq!(
        tree.children.keys().map(String::as_str).collect::<Vec<_>>(),
        ["top"]
    );
    let top = &tree.children["top"];
    assert_eq!(top.docs, ["Top docs"]);
    assert_eq!(
        top.messages
            .iter()
            .map(|entry| entry.property.as_str())
            .collect::<Vec<_>>(),
        ["second", "first"]
    );
    assert_eq!(
        top.children.keys().map(String::as_str).collect::<Vec<_>>(),
        ["nested"]
    );
    let nested = &top.children["nested"];
    assert_eq!(nested.docs, ["Nested docs"]);
    assert_eq!(nested.messages[0].signature, schema.messages()[3]);
    assert_eq!(nested.messages[0].signature.name, "top.nested.leaf");
    assert_eq!(
        schema
            .origins()
            .iter()
            .find(|origin| origin.name == nested.messages[0].signature.name)
            .unwrap()
            .span
            .source,
        SourceId(13)
    );
}
