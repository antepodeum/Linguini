use super::{
    completion_items, definition_at_with_workspace, diagnostics, diagnostics_with_workspace,
    document_symbols, format_document, hover_at, hover_at_with_workspace, prepare_rename_at,
    references_at, references_at_with_workspace, rename_workspace_edits, semantic_tokens,
    LinguiniDocument,
};

#[test]
fn diagnostics_report_schema_parse_errors() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "delivery(count: Number\n",
    );

    let diagnostics = diagnostics(&document);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("schema syntax error")));
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
fn hover_uses_enum_samples_for_schema_messages() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "enum Fruit { apple, pear }\ndelivery(fruit: Fruit)\n",
    );
    let offset = document.text.find("delivery").expect("delivery offset");

    let hover = hover_at(&document, offset).expect("hover");

    assert!(hover.contains("delivery(fruit: apple)"));
}

#[test]
fn locale_hover_inherits_schema_docs_from_workspace() {
    let schema = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "/// Delivery label\ndelivery(count: Number)\n",
    );
    let locale =
        LinguiniDocument::new("file:///ru.lgl", "linguini-locale", "delivery = Доставка\n");
    let offset = locale.text.find("delivery").expect("delivery offset");

    let hover = hover_at_with_workspace(&locale, offset, [schema]).expect("hover");

    assert!(hover.contains("Delivery label"));
    assert!(hover.contains("delivery(count: Number)"));
    assert!(hover.contains("delivery -> Доставка"));
}

#[test]
fn locale_hover_shows_schema_signature_without_runtime_emulation() {
    let schema = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "enum Fruit { apple, pear }\nenum Size { small, big }\ndelivery(fruit: Fruit, size: Size, count: Number)\n",
    );
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "impl Fruit {\n  apple { form nom(Plural) { one => apple\n_ => apples } }\n}\nform SizeWord(Size, Plural) {\n  small { _ => small }\n  big { _ => big }\n}\ndelivery = Delivered {count} {SizeWord(size, count)} {fruit.nom(count)}.\n",
    );
    let offset = locale.text.find("delivery").expect("delivery offset");

    let hover = hover_at_with_workspace(&locale, offset, [schema]).expect("hover");

    assert!(hover.contains("delivery(fruit: Fruit, size: Size, count: Number)"));
    assert!(
        hover.contains("delivery -> Delivered {count} {SizeWord(size, count)} {fruit.nom(count)}.")
    );
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
fn hover_on_inline_plural_branch_lists_locale_samples() {
    let document = LinguiniDocument::new(
        "file:///ru.lgl",
        "linguini-locale",
        "summary = {fn(Plural(count)) {\n  one => item\n  other => items\n}}\n",
    );
    let offset = document.text.find("one").expect("one offset");

    let hover = hover_at(&document, offset).expect("hover");

    assert!(hover.contains("plural branch `one` in `inline fn`"));
    assert!(hover.contains("Locale `ru` category `one`"));
    assert!(hover.contains("Sample numbers: 1, 21"));
}

#[test]
fn hover_finds_inline_plural_inside_nested_function_dispatch() {
    let document = LinguiniDocument::new(
        "file:///ru.lgl",
        "linguini-locale",
        "fn summary(Gender, count: Number) {\n  masculine => {fn(Plural(count)) {\n    one => item\n    other => items\n  }}\n  _ => items\n}\n",
    );
    let offset = document
        .text
        .find("one => item")
        .expect("inline branch offset");

    let hover = hover_at(&document, offset).expect("hover");

    assert!(hover.contains("plural branch `one` in `inline fn`"));
    assert!(hover.contains("Locale `ru` category `one`"));
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
        "let cart_label = Cart\nenum Fruit { apple, pear }\ndelivery = {Fruit}\n",
    );

    let items = completion_items(&document, document.text.len());

    assert!(items.contains(&"impl".to_owned()));
    assert!(items.contains(&"let".to_owned()));
    assert!(items.contains(&"cart_label".to_owned()));
    assert!(items.contains(&"Fruit".to_owned()));
}

#[test]
fn semantic_tokens_include_keywords_comments_and_text() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "// comment\nlet cart_label = Cart\nenum Fruit { apple }\ndelivery = Delivered\n",
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
fn semantic_tokens_mark_inline_plural_intrinsic_as_the_builtin_selector_type() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "summary = {fn(Plural(count)) {\n  one => item\n  other => items\n}}\n",
    );
    let (line, start) = document.position(document.text.find("Plural").expect("Plural offset"));

    let tokens = semantic_tokens(&document);

    assert!(tokens
        .iter()
        .any(|token| token.token_type == 2 && token.line == line && token.start == start));
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
fn inline_selector_and_body_resolve_to_the_schema_parameter() {
    let schema = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "enum Tone { formal, casual }\ngreeting(tone: Tone)\n",
    );
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "greeting = {fn(tone) {\n  formal => {tone}\n  casual => {tone}\n}}\n",
    );
    let offset = schema.text.find("tone").expect("schema parameter offset");

    let references = references_at_with_workspace(&schema, offset, [locale.clone()]);

    assert_eq!(references.len(), 4);
    assert_eq!(
        references
            .iter()
            .filter(|reference| reference.uri == locale.uri)
            .map(|reference| &locale.text[reference.span.start..reference.span.end])
            .collect::<Vec<_>>(),
        ["tone", "tone", "tone"]
    );
}

#[test]
fn inline_binding_declaration_and_branch_references_are_document_local() {
    let document = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "greeting = {fn(tone, greet: name) {\n  formal => {greet}\n  casual => {greet}\n}}\n",
    );
    let offset = document.text.find("greet:").expect("binding offset");

    let references = references_at(&document, offset);

    assert_eq!(references.len(), 3);
    assert_eq!(
        references
            .iter()
            .map(|span| &document.text[span.start..span.end])
            .collect::<Vec<_>>(),
        ["greet", "greet", "greet"]
    );
}

#[test]
fn definition_from_locale_variable_reference_jumps_to_let_declaration() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "let cart_label = Cart\nsummary = {cart_label}: {count}\n",
    );
    let offset = document.text.rfind("cart_label").expect("reference offset");

    let (uri, span) = definition_at_with_workspace(&document, offset, []).expect("definition");

    assert_eq!(uri, document.uri);
    assert_eq!(&document.text[span.start..span.end], "cart_label");
    assert_eq!(
        span.start,
        document
            .text
            .find("cart_label")
            .expect("declaration offset")
    );
}

#[test]
fn rename_updates_locale_variable_declaration_and_references() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "let cart_label = Cart\nsummary = {cart_label}: {count}\n",
    );
    let offset = document
        .text
        .find("cart_label")
        .expect("declaration offset");

    let edits = rename_workspace_edits([document.clone()], &document, offset, "cart_title");

    assert_eq!(edits.len(), 2);
    assert!(edits.iter().all(|edit| edit.uri == document.uri));
    assert_eq!(
        edits
            .iter()
            .map(|edit| &document.text[edit.edit.span.start..edit.edit.span.end])
            .collect::<Vec<_>>(),
        ["cart_label", "cart_label"]
    );
}

#[test]
fn definition_from_locale_message_jumps_to_schema_message() {
    let schema = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "delivery(count: Number)\n",
    );
    let locale =
        LinguiniDocument::new("file:///ru.lgl", "linguini-locale", "delivery = Доставка\n");
    let offset = locale.text.find("delivery").expect("delivery offset");

    let (uri, span) =
        definition_at_with_workspace(&locale, offset, [schema.clone()]).expect("definition");

    assert_eq!(uri, schema.uri);
    assert_eq!(&schema.text[span.start..span.end], "delivery");
}

#[test]
fn document_symbols_expose_top_level_items() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "let cart_label = Cart\nenum Fruit { apple, pear }\ndelivery = Delivered\n",
    );

    let symbols = document_symbols(&document);

    assert_eq!(
        symbols
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>(),
        ["cart_label", "Fruit", "delivery"]
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

#[test]
fn locale_diagnostics_use_only_matching_schema_namespace() {
    let shop_schema = LinguiniDocument::new(
        "file:///schema/shop.lgs",
        "linguini-schema",
        "delivery(count: Number)\n",
    )
    .with_source_identity("shop", None);
    let account_schema = LinguiniDocument::new(
        "file:///schema/account.lgs",
        "linguini-schema",
        "sign_in(email: String)\n",
    )
    .with_source_identity("account", None);
    let locale = LinguiniDocument::new(
        "file:///locales/shop/en.lgl",
        "linguini-locale",
        "delivery = Delivered\n",
    )
    .with_source_identity("shop", Some("en".to_owned()));

    let diagnostics = diagnostics_with_workspace(&locale, [shop_schema, account_schema]);

    assert!(diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message.contains("sign_in")));
}

#[test]
fn schema_semantic_diagnostics_are_reported() {
    let schema = LinguiniDocument::new(
        "file:///schema/shop.lgs",
        "linguini-schema",
        "enum Fruit { apple, apple }\ndelivery(count: Missing)\n",
    );

    let diagnostics = diagnostics(&schema);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("duplicate") && diagnostic.message.contains("variant")
    }));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unknown schema type")));
}

#[test]
fn recoverable_syntax_errors_do_not_suppress_independent_semantic_diagnostics() {
    let schema = LinguiniDocument::new(
        "file:///schema/shop.lgs",
        "linguini-schema",
        "enum Fruit { apple, apple }\n#\ndelivery(count: Missing)\n",
    );

    let diagnostics = diagnostics(&schema);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("schema syntax error")));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("duplicate") && diagnostic.message.contains("variant")
    }));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unknown schema type")));
}

#[test]
fn token_end_is_not_inside_rename_target() {
    let document = LinguiniDocument::new("file:///shop.lgs", "linguini-schema", "delivery\n");
    let end = document.text.find("delivery").expect("offset") + "delivery".len();

    assert!(prepare_rename_at(&document, end).is_none());
}

#[test]
fn rename_does_not_touch_unrelated_same_text() {
    let schema = LinguiniDocument::new("file:///shop.lgs", "linguini-schema", "delivery\n");
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "let delivery = Local\nsummary = {delivery}\n",
    );
    let offset = schema.text.find("delivery").expect("offset");

    let edits = rename_workspace_edits([schema.clone(), locale], &schema, offset, "shipping");

    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].uri, schema.uri);
}

#[test]
fn rename_rejects_reserved_names_and_collisions() {
    let document = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "delivery\nshipping\n",
    );
    let offset = document.text.find("delivery").expect("offset");

    assert!(rename_workspace_edits([document.clone()], &document, offset, "enum").is_empty());
    assert!(rename_workspace_edits([document.clone()], &document, offset, "shipping").is_empty());
}

#[test]
fn semantic_resolution_handles_zero_argument_calls_and_grouped_paths() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "fn label() { _ => Label }\nshop { title = Title }\nsummary = {label()} {shop.title}\n",
    );

    let call = document.text.rfind("label").expect("call offset");
    let (call_uri, call_span) =
        definition_at_with_workspace(&document, call, []).expect("function definition");
    assert_eq!(call_uri, document.uri);
    assert_eq!(&document.text[call_span.start..call_span.end], "label");
    assert_eq!(
        call_span.start,
        document.text.find("label").expect("declaration")
    );

    let grouped = document.text.rfind("title").expect("grouped reference");
    let (group_uri, group_span) =
        definition_at_with_workspace(&document, grouped, []).expect("grouped definition");
    assert_eq!(group_uri, document.uri);
    assert_eq!(&document.text[group_span.start..group_span.end], "title");
}

#[test]
fn schema_parameter_references_bind_across_locale_documents() {
    let schema = LinguiniDocument::new(
        "file:///shop.lgs",
        "linguini-schema",
        "greeting(name: String)\n",
    );
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "greeting = Hello, {name}!\n",
    );
    let reference = locale.text.rfind("name").expect("parameter reference");

    let (uri, span) =
        definition_at_with_workspace(&locale, reference, [schema.clone()]).expect("definition");
    assert_eq!(uri, schema.uri);
    assert_eq!(&schema.text[span.start..span.end], "name");

    let declaration = schema.text.find("name").expect("parameter declaration");
    let edits = rename_workspace_edits(
        [schema.clone(), locale.clone()],
        &schema,
        declaration,
        "customer",
    );
    assert_eq!(edits.len(), 2);
    assert!(edits.iter().any(|edit| edit.uri == schema.uri));
    assert!(edits.iter().any(|edit| edit.uri == locale.uri));
}

#[test]
fn rename_rejects_cross_kind_declaration_collisions() {
    let document = LinguiniDocument::new(
        "file:///shop.lgl",
        "linguini-locale",
        "let shipping = Shipping\ndelivery = Delivered\n",
    );
    let offset = document.text.find("delivery").expect("message declaration");

    assert!(rename_workspace_edits([document.clone()], &document, offset, "shipping").is_empty());
}

#[test]
fn workspace_references_include_declaration_metadata() {
    let schema = LinguiniDocument::new("file:///shop.lgs", "linguini-schema", "delivery\n");
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "delivery = Delivered\nsummary = {delivery}\n",
    );
    let offset = schema.text.find("delivery").expect("offset");

    let references = references_at_with_workspace(&schema, offset, [schema.clone(), locale]);

    assert_eq!(references.len(), 3);
    assert_eq!(
        references
            .iter()
            .filter(|reference| reference.declaration)
            .count(),
        2
    );
}

#[test]
fn definition_does_not_fall_back_to_undefined_reference() {
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "summary = {ghost}\nother = {ghost}\n",
    );
    let offset = locale.text.find("ghost").expect("offset");

    assert!(definition_at_with_workspace(&locale, offset, []).is_none());
}

#[test]
fn ambiguous_unqualified_schema_definition_is_rejected() {
    let first = LinguiniDocument::new("file:///one.lgs", "linguini-schema", "delivery\n");
    let second = LinguiniDocument::new("file:///two.lgs", "linguini-schema", "delivery\n");
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "delivery = Delivered\n",
    );
    let offset = locale.text.find("delivery").expect("offset");

    assert!(definition_at_with_workspace(&locale, offset, [first, second]).is_none());
}

#[test]
fn semantic_tokens_split_multiline_raw_text() {
    let locale = LinguiniDocument::new(
        "file:///en.lgl",
        "linguini-locale",
        "message = \"\"\"\nfirst\nsecond\n\"\"\"\n",
    );

    let text_tokens = semantic_tokens(&locale)
        .into_iter()
        .filter(|token| token.token_type == 4)
        .collect::<Vec<_>>();

    assert!(text_tokens.iter().any(|token| token.line == 1));
    assert!(text_tokens.iter().any(|token| token.line == 2));
}

#[test]
fn unsupported_language_id_is_rejected() {
    assert!(LinguiniDocument::try_new("file:///notes.txt", "plaintext", "notes").is_none());
}

#[test]
fn oversized_documents_are_rejected_before_indexing_lines() {
    let source = "x".repeat(4 * 1024 * 1024 + 1);

    assert!(LinguiniDocument::try_new("file:///huge.lgl", "linguini-locale", source).is_none());
}

#[test]
fn configured_locale_drives_plural_hover_for_custom_layout() {
    let document = LinguiniDocument::new(
        "file:///translations/current.lgl",
        "linguini-locale",
        "form Count(Plural) {\n  one => item\n  _ => items\n}\n",
    )
    .with_source_identity("main", Some("ru".to_owned()));
    let offset = document.text.find("one").expect("offset");

    let hover = hover_at(&document, offset).expect("hover");

    assert!(hover.contains("Locale `ru` category `one`"));
}
