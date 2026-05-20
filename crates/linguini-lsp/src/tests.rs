use super::{
    completion_items, diagnostics, document_symbols, format_document, hover_at, prepare_rename_at,
    references_at, rename_workspace_edits, semantic_tokens, LinguiniDocument, CRATE_PURPOSE,
};

#[test]
fn crate_has_unit_test_structure() {
    assert_eq!(CRATE_PURPOSE, "language server");
}

#[test]
fn diagnostics_report_schema_parse_errors() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "delivery(count: Number\n",
    );

    let diagnostics = diagnostics(&document);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("schema syntax error"));
}

#[test]
fn hover_uses_doc_comments_for_schema_symbols() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "/// Delivery label\ndelivery(count: Number)\n",
    );
    let offset = document.text.find("delivery").expect("delivery offset");

    let hover = hover_at(&document, offset).expect("hover");

    assert!(hover.contains("message `delivery`"));
    assert!(hover.contains("Delivery label"));
    assert!(hover.contains("Sample"));
    assert!(hover.contains("delivery(count: 3)"));
}

#[test]
fn hover_previews_locale_message_output_shape() {
    let document = LinguiniDocument::new(
        "file:///ru.lgl",
        "linguini-locale",
        "delivery = {count} items\n",
    );
    let offset = document.text.find("delivery").expect("delivery offset");

    let hover = hover_at(&document, offset).expect("hover");

    assert!(hover.contains("delivery -> {count} items"));
}

#[test]
fn hover_on_plural_branch_lists_locale_samples() {
    let document = LinguiniDocument::new(
        "file:///ru.lgl",
        "linguini-locale",
        "form Count(Plural) {\n  one => item\n  few => items\n  _ => items\n}\n",
    );
    let offset = document.text.find("one").expect("one offset");

    let hover = hover_at(&document, offset).expect("hover");

    assert!(hover.contains("plural branch `one`"));
    assert!(hover.contains("Locale `ru` category `one`"));
    assert!(hover.contains("Sample numbers: 1, 21"));
}

#[test]
fn diagnostics_include_branch_coverage() {
    let locale = LinguiniDocument::new(
        "file:///ru.lgl",
        "linguini-locale",
        "enum Gender { male, female, neuter, other }\nform SizeAdj(Plural, Gender) {\n  one {\n    male => большой\n    female => большая\n  }\n  _ => большие\n}\ndelivery = Delivered\n",
    );

    let locale_diagnostics = diagnostics(&locale);

    assert!(locale_diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("enum `Gender` is missing branch `neuter`")));
}

#[test]
fn completion_includes_keywords_and_document_symbols() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "enum Fruit { apple, pear }\ndelivery = {Fruit}\n",
    );

    let items = completion_items(&document, document.text.len());

    assert!(items.contains(&"impl".to_owned()));
    assert!(items.contains(&"Fruit".to_owned()));
}

#[test]
fn semantic_tokens_include_keywords_comments_and_text() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "// comment\nenum Fruit { apple }\ndelivery = Delivered\n",
    );

    let tokens = semantic_tokens(&document);

    assert!(tokens.iter().any(|token| token.token_type == 0));
    assert!(tokens.iter().any(|token| token.token_type == 5));
    assert!(tokens.iter().any(|token| token.token_type == 4));
}

#[test]
fn semantic_tokens_use_utf16_columns_for_non_ascii_text() {
    let document = LinguiniDocument::new("file:///shop.lgl", "linguini-locale", "hello = Привет\n");

    let tokens = semantic_tokens(&document);
    let text_token = tokens
        .iter()
        .find(|token| token.token_type == 4)
        .expect("raw text token");

    assert_eq!(text_token.start, 7);
    assert_eq!(text_token.length, 7);
}

#[test]
fn semantic_tokens_mark_form_names_as_functions() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "impl Fruit { apple { form nom(Plural) { one => яблоко } } }\n",
    );
    let (line, start) = document.position(document.text.find("nom").expect("nom offset"));

    let tokens = semantic_tokens(&document);

    assert!(tokens
        .iter()
        .any(|token| token.token_type == 7 && token.line == line && token.start == start));
}

#[test]
fn references_find_matching_identifiers() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "enum Fruit { apple }\nimpl Fruit { apple { label = Fruit } }\n",
    );
    let offset = document.text.find("Fruit").expect("Fruit offset");

    let references = references_at(&document, offset);

    assert_eq!(references.len(), 2);
}

#[test]
fn prepare_rename_uses_identifier_under_cursor() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "delivery(count: Number)\ncheckout(count: Number)\n",
    );
    let offset = document.text.find("checkout").expect("checkout offset");

    let span = prepare_rename_at(&document, offset).expect("rename span");

    assert_eq!(&document.text[span.start..span.end], "checkout");
}

#[test]
fn rename_workspace_edits_schema_symbol_and_locale_references() {
    let schema = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "delivery(count: Number)\n",
    );
    let locale = LinguiniDocument::new(
        "file:///ru.lgl",
        "linguini-locale",
        "delivery = Доставка\nsummary = {delivery}\n",
    );
    let offset = schema.text.find("delivery").expect("delivery offset");

    let edits = rename_workspace_edits(
        [schema.clone(), locale.clone()],
        &schema,
        offset,
        "shipping",
    );

    assert_eq!(edits.len(), 3);
    assert!(edits.iter().any(|edit| edit.uri == schema.uri));
    assert_eq!(
        edits
            .iter()
            .filter(|edit| edit.uri == locale.uri)
            .map(|edit| &locale.text[edit.edit.span.start..edit.edit.span.end])
            .collect::<Vec<_>>(),
        ["delivery", "delivery"]
    );
}

#[test]
fn document_symbols_expose_top_level_items() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "enum Fruit { apple, pear }\ndelivery(count: Number)\n",
    );

    let symbols = document_symbols(&document);

    assert_eq!(
        symbols
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>(),
        ["Fruit", "delivery"]
    );
}

#[test]
fn formatting_returns_whole_document_edit() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "delivery(count:Number)\n",
    );

    let edit = format_document(&document).expect("format document");

    assert_eq!(edit.new_text, "delivery(count: Number)\n");
    assert_eq!(edit.span.end, document.text.len());
}
