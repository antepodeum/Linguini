use super::*;
use crate::{compose_locale_modules, message_dependency_closure};
use linguini_ir::{lower_locale_typed, lower_schema_typed, IrModuleBuilder};
use linguini_syntax::{parse_locale_in, parse_schema_in};

#[test]
fn effective_origins_exclude_replaced_declarations_and_unknown_symbols() {
    let source = concat!(
        "enum Choice { a }\nlet label = Text\n",
        "impl Fruit { apple { label = Apple } }\n",
        "fn Ready() { _ => ready }\n",
    );
    let base = lower_locale_typed(&parse_locale_in(source, SourceId(7)).unwrap());
    let selected = lower_locale_typed(&parse_locale_in(source, SourceId(22)).unwrap());
    let locale = compose_locale_modules([&base, &selected]);
    for (kind, name) in [
        (SemanticSymbolKind::LocaleEnum, "Choice"),
        (SemanticSymbolKind::LocaleVariable, "label"),
        (SemanticSymbolKind::LocaleForm, "Fruit"),
        (SemanticSymbolKind::LocaleFunction, "Ready"),
    ] {
        assert_eq!(
            semantic_symbol_span(&locale, kind, name).unwrap().source,
            SourceId(22)
        );
        assert_eq!(
            semantic_symbol_source_ids(&locale, kind, name),
            [SourceId(22)]
        );
        assert_eq!(semantic_symbol_span(&locale, kind, "missing"), None);
        assert!(semantic_symbol_source_ids(&locale, kind, "missing").is_empty());
    }
    assert!(
        semantic_symbol_source_ids(&locale, SemanticSymbolKind::LocaleFunction, "label").is_empty()
    );
}

#[test]
fn inline_inputs_arguments_and_branches_share_provenance_with_message_slices() {
    let locale = lower_locale_typed(
        &parse_locale_in(
            concat!(
                "let count = 1\nlet value = Ada\nfn Echo(value: String) { _ => {value} }\n",
                "let label = {fn(Plural(count), copy: Echo(value)) {\n one => {copy}\n _ => other\n}}\n",
                "root = {label}\n",
            ),
            SourceId(9),
        )
        .unwrap(),
    );
    let mut variable = locale.variables()[2].clone();
    variable.value.span.source = SourceId(3);
    let IrTextPart::Placeholder(expression) = &mut variable.value.parts[0] else {
        panic!("inline expression")
    };
    expression.span.source = SourceId(4);
    let IrExpressionKind::InlineFunction { inputs, branches } = &mut expression.kind else {
        panic!("inline function")
    };
    let IrInlineFunctionInput::Selector { value, span } = &mut inputs[0] else {
        panic!("selector")
    };
    span.source = SourceId(5);
    value.span.source = SourceId(6);
    value.arguments[0].span.source = SourceId(7);
    let IrInlineFunctionInput::Binding { value, span, .. } = &mut inputs[1] else {
        panic!("binding")
    };
    span.source = SourceId(8);
    value.arguments[0].span.source = SourceId(10);
    branches[0].span.source = SourceId(11);
    let IrFunctionBranchValue::Text(text) = &mut branches[0].value else {
        panic!("text")
    };
    text.span.source = SourceId(12);
    let IrTextPart::Placeholder(reference) = &mut text.parts[0] else {
        panic!("reference")
    };
    reference.span.source = SourceId(13);
    branches[1].span.source = SourceId(14);
    let IrFunctionBranchValue::Text(text) = &mut branches[1].value else {
        panic!("text")
    };
    text.span.source = SourceId(15);
    let locale = IrModuleBuilder::seeded(&locale)
        .clear_variables()
        .push_variable(locale.variables()[0].clone())
        .push_variable(locale.variables()[1].clone())
        .push_variable(variable)
        .build()
        .unwrap();
    assert_eq!(
        semantic_symbol_source_ids(&locale, SemanticSymbolKind::LocaleVariable, "label"),
        (3..=15).map(SourceId).collect::<Vec<_>>()
    );
    let schema = lower_schema_typed(&parse_schema_in("root\n", SourceId(2)).unwrap());
    let closure = message_dependency_closure(&schema, "en", &locale, "root").unwrap();
    assert_eq!(
        closure.source_ids(),
        (2..=15).map(SourceId).collect::<Vec<_>>()
    );
}

#[test]
fn nested_forms_maps_direct_branches_and_function_dispatch_retain_all_sources() {
    let locale = lower_locale_typed(
        &parse_locale_in(
            concat!(
        "impl Fruit { apple {\nlabel = Apple\nform nom(Plural) {\n one => apple\n _ => apples\n}\n",
        "display { short = app }\n} }\n",
        "fn Label(Plural, Choice) {\n one {\n a => A\n _ => B\n }\n _ => Many\n}\n",
    ),
            SourceId(20),
        )
        .unwrap(),
    );
    let mut form = locale.forms()[0].clone();
    let entries = &mut form.variants[0].entries;
    let IrFormEntry::Attribute {
        value: IrValue::Text(text),
        ..
    } = &mut entries[0]
    else {
        panic!("text attribute")
    };
    text.span.source = SourceId(22);
    let IrFormEntry::Attribute {
        value: IrValue::Map(branches),
        ..
    } = &mut entries[1]
    else {
        panic!("map")
    };
    branches[0].span.source = SourceId(23);
    branches[0].value.span.source = SourceId(24);
    branches[1].span.source = SourceId(25);
    branches[1].value.span.source = SourceId(26);
    let mut direct = branches[0].clone();
    direct.span.source = SourceId(28);
    direct.value.span.source = SourceId(29);
    let IrFormEntry::Attribute {
        value: IrValue::Object(children),
        ..
    } = &mut entries[2]
    else {
        panic!("object")
    };
    let IrFormEntry::Attribute {
        value: IrValue::Text(text),
        ..
    } = &mut children[0]
    else {
        panic!("nested text")
    };
    text.span.source = SourceId(27);
    entries.push(IrFormEntry::Branch(direct));
    let mut function = locale.functions()[0].clone();
    function.branches[0].span.source = SourceId(31);
    let IrFunctionBranchValue::Dispatch(children) = &mut function.branches[0].value else {
        panic!("nested dispatch")
    };
    children[0].span.source = SourceId(32);
    let IrFunctionBranchValue::Text(text) = &mut children[0].value else {
        panic!("nested text")
    };
    text.span.source = SourceId(33);
    let locale = IrModuleBuilder::seeded(&locale)
        .clear_forms()
        .push_form(form)
        .clear_functions()
        .push_function(function)
        .build()
        .unwrap();
    assert_eq!(
        semantic_symbol_source_ids(&locale, SemanticSymbolKind::LocaleForm, "Fruit"),
        [20, 22, 23, 24, 25, 26, 27, 28, 29].map(SourceId)
    );
    assert_eq!(
        semantic_symbol_source_ids(&locale, SemanticSymbolKind::LocaleFunction, "Label"),
        [20, 31, 32, 33].map(SourceId)
    );
}
