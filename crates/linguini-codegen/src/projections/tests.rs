use super::*;
use linguini_ir::{lower_locale_typed, lower_schema_typed};
use linguini_syntax::{parse_locale, parse_schema};

#[test]
fn namespace_projections_preserve_selected_groups_globals_and_declaration_order() {
    let schema = lower_schema_typed(
        &parse_schema(concat!(
            "enum Choice { a, b }\nroot\n",
            "top { keep { title } keeper { other } drop { hidden } empty {} }\nother { title }\n",
        ))
        .unwrap(),
    );
    let selected = select_schema_paths(&schema, &["top.keep".into()]);
    assert_eq!(selected.messages(), &schema.messages()[1..2]);
    assert_eq!(
        selected
            .groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["top", "top.keep"]
    );
    assert_eq!(selected.enums(), schema.enums());
    let entire = select_schema_paths(&schema, &["top".into()]);
    assert_eq!(entire.messages(), &schema.messages()[1..4]);
    assert_eq!(
        entire
            .groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["top", "top.keep", "top.keeper", "top.drop", "top.empty"]
    );
    let empty = select_schema_paths(&schema, &["top.empty".into()]);
    assert!(empty.messages().is_empty());
    assert_eq!(
        empty
            .groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["top", "top.empty"]
    );
    assert_eq!(
        select_schema_paths(&schema, &[]),
        schema.as_module().clone()
    );
    assert_eq!(top_level_message_namespaces(&schema), ["other", "top"]);
    let top = namespace_module(&schema, "top");
    assert_eq!(top.messages(), entire.messages());
    assert_eq!(top.groups(), entire.groups());
    let root = root_module(&schema);
    assert_eq!(root.messages(), &schema.messages()[..1]);
    assert_eq!(
        root.groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["top", "other"]
    );
    assert_eq!(root.enums(), schema.enums());
    let locale = lower_locale_typed(
        &parse_locale(concat!(
            "let prefix = Text\nroot = Root\n",
            "top { keep { title = Keep } keeper { other = Keeper } drop { hidden = Drop } }\nother { title = Other }\n",
        ))
        .unwrap(),
    );
    let projected = locale_module_for_schema(&locale, &selected);
    assert_eq!(projected.messages(), &locale.messages()[1..2]);
    assert_eq!(projected.variables(), locale.variables());
    assert_eq!(
        projected
            .groups()
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["top", "top.keep"]
    );
}
