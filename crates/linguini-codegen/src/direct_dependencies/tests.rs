use super::*;
use linguini_ir::{lower_locale, lower_schema};
use linguini_syntax::{parse_locale, parse_schema};

#[test]
fn direct_edges_select_alias_forms_and_calls_without_their_transitive_internals() {
    let schema = lower_schema(
        &parse_schema("enum Fruit { apple }\ntype Item = Fruit\nroot(item: Item, count: Number)\n")
            .unwrap(),
    );
    let locale = lower_locale(
        &parse_locale(concat!(
            "let base = Base\nlet leaf = {base}\nlet unused = Unused\n",
            "fn Render(value: String) { _ => {leaf} {value} }\n",
            "impl Fruit { apple { form nom(Plural) {\n one => apple\n _ => apples\n} } }\n",
            "root = {Render(item.nom(count))}\n",
        ))
        .unwrap(),
    );
    assert_eq!(
        direct_message_dependencies(
            &schema,
            &locale,
            &schema.messages()[0],
            &locale.messages()[0]
        ),
        [
            (SemanticSymbolKind::LocaleForm, "Fruit".to_owned()),
            (SemanticSymbolKind::LocaleFunction, "Render".to_owned())
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(
        direct_symbol_dependencies(
            &schema,
            &locale,
            SemanticSymbolKind::LocaleFunction,
            "Render"
        ),
        [(SemanticSymbolKind::LocaleVariable, "leaf".to_owned())]
            .into_iter()
            .collect()
    );
    assert_eq!(
        direct_symbol_dependencies(&schema, &locale, SemanticSymbolKind::LocaleVariable, "leaf"),
        [(SemanticSymbolKind::LocaleVariable, "base".to_owned())]
            .into_iter()
            .collect()
    );
}

#[test]
fn direct_edges_respect_lexical_scope_and_simultaneous_inline_inputs() {
    let schema = lower_schema(&parse_schema("enum Fruit { apple }\ntype Item = Fruit\nroot(item: Item, other: String, count: Number)\n").unwrap());
    let locale = lower_locale(
        &parse_locale(concat!(
            "let unused = Unused global\n",
            "impl Fruit { apple { form nom(Plural) {\n one => apple\n _ => apples\n} } }\n",
            "root = {fn(item: other, next: item) { _ => {next.nom(count)} }}\n",
        ))
        .unwrap(),
    );
    let expected = [(SemanticSymbolKind::LocaleForm, "Fruit".to_owned())]
        .into_iter()
        .collect();
    assert_eq!(
        direct_message_dependencies(
            &schema,
            &locale,
            &schema.messages()[0],
            &locale.messages()[0]
        ),
        expected
    );
    let closure = crate::message_dependency_closure(&schema, "en", &locale, "root").unwrap();
    assert_eq!(closure.locale_module().forms().len(), 1);
    assert!(closure.locale_module().variables().is_empty());
}
