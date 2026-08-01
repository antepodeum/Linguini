use crate::{lex, lex_schema, lex_with_recovery, Span, TokenKind};

use super::common::{assert_snapshot, render_tokens};

#[test]
fn lexes_schema_fixture_tokens() {
    let source = include_str!("../../../../tests/fixtures/golden/schema/shop.lgs");
    let tokens = lex_schema(source).expect("schema fixture lexes");
    let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();

    assert!(kinds.contains(&&TokenKind::Ident("enum".into())));
    assert!(kinds.contains(&&TokenKind::Ident("Fruit".into())));
    assert!(kinds.contains(&&TokenKind::Ident("delivery".into())));
    assert!(kinds.contains(&&TokenKind::LBrace));
    assert!(kinds.contains(&&TokenKind::RBrace));
    assert!(kinds.contains(&&TokenKind::LParen));
    assert!(kinds.contains(&&TokenKind::RParen));
    assert!(kinds.contains(&&TokenKind::Colon));
}

#[test]
fn lexes_locale_fixture_with_cyrillic_raw_text() {
    let source = include_str!("../../../../tests/fixtures/golden/locale/ru.lgl");
    let tokens = lex(source).expect("locale fixture lexes");

    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::RawText(" Доставлено".into())));
}

#[test]
fn lexes_raw_text_placeholders() {
    let tokens = lex("delivery = {count} {fruit.nom}\n").expect("source lexes");
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("delivery".into()),
            TokenKind::Whitespace,
            TokenKind::Equals,
            TokenKind::RawText(" ".into()),
            TokenKind::LBrace,
            TokenKind::Ident("count".into()),
            TokenKind::RBrace,
            TokenKind::RawText(" ".into()),
            TokenKind::LBrace,
            TokenKind::Ident("fruit".into()),
            TokenKind::Dot,
            TokenKind::Ident("nom".into()),
            TokenKind::RBrace,
            TokenKind::Newline,
        ]
    );
}

#[test]
fn lexes_single_line_raw_text_before_closing_brace() {
    let tokens = lex("impl Plan { starter { label = Starter } }\n").expect("source lexes");
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();

    assert!(kinds.windows(3).any(|window| {
        window
            == [
                TokenKind::Equals,
                TokenKind::RawText(" Starter ".into()),
                TokenKind::RBrace,
            ]
    }));
}

#[test]
fn lexes_arrow_raw_text_and_comments() {
    let tokens = lex("/// doc\n// note\nmale => Доставлен\n").expect("source lexes");
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::DocComment(" doc".into()),
            TokenKind::Newline,
            TokenKind::Comment(" note".into()),
            TokenKind::Newline,
            TokenKind::Ident("male".into()),
            TokenKind::Whitespace,
            TokenKind::Arrow,
            TokenKind::RawText(" Доставлен".into()),
            TokenKind::Newline,
        ]
    );
}

#[test]
fn lexes_inline_function_branches_and_resumes_message_text() {
    let source = "greeting = Hello {fn(gender, name: GreetingName(name)) {\n  masculine => dear {name}\n  _ => friend\n}}!\n";
    let tokens = lex(source).expect("inline fn source lexes");
    let kinds = tokens
        .into_iter()
        .map(|token| token.kind)
        .collect::<Vec<_>>();

    assert!(kinds.windows(5).any(|window| {
        window
            == [
                TokenKind::Ident("masculine".into()),
                TokenKind::Whitespace,
                TokenKind::Arrow,
                TokenKind::RawText(" dear ".into()),
                TokenKind::LBrace,
            ]
    }));
    assert!(kinds.windows(3).any(|window| {
        window
            == [
                TokenKind::RBrace,
                TokenKind::RawText("!".into()),
                TokenKind::Newline,
            ]
    }));
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| matches!(kind, TokenKind::Arrow))
            .count(),
        2
    );
}

#[test]
fn lexes_inline_function_selector_and_binding_expressions() {
    let source = "summary = {fn(tone, Plural(count), name: GreetingName(tone, count)) {\n  formal {\n    one => {name}\n    _ => many\n  }\n  _ => other\n}}\n";
    let significant = lex(source)
        .expect("inline function inputs lex")
        .into_iter()
        .map(|token| token.kind)
        .filter(|kind| !matches!(kind, TokenKind::Whitespace | TokenKind::Newline))
        .collect::<Vec<_>>();

    assert!(significant.windows(5).any(|window| {
        window
            == [
                TokenKind::Ident("Plural".into()),
                TokenKind::LParen,
                TokenKind::Ident("count".into()),
                TokenKind::RParen,
                TokenKind::Comma,
            ]
    }));
    assert!(significant.windows(4).any(|window| {
        window
            == [
                TokenKind::Ident("name".into()),
                TokenKind::Colon,
                TokenKind::Ident("GreetingName".into()),
                TokenKind::LParen,
            ]
    }));
    assert_eq!(
        significant
            .iter()
            .filter(|kind| matches!(kind, TokenKind::Equals))
            .count(),
        1
    );
    assert_eq!(
        significant
            .iter()
            .filter(|kind| matches!(kind, TokenKind::Arrow))
            .count(),
        3
    );
}

#[test]
fn lexes_multiline_leaf_inside_inline_function() {
    let source = "summary = {fn(tone, name: name) {\n  formal => \"\"\"\nDear {name}\n\"\"\"\n  _ => Friend\n}}\n";
    let kinds = lex(source)
        .expect("inline multiline leaf lexes")
        .into_iter()
        .map(|token| token.kind)
        .collect::<Vec<_>>();

    assert_eq!(
        kinds
            .iter()
            .filter(|kind| matches!(kind, TokenKind::TripleQuote))
            .count(),
        2
    );
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| matches!(kind, TokenKind::Arrow))
            .count(),
        2
    );
    assert!(matches!(kinds.last(), Some(TokenKind::Newline)));
}

#[test]
fn lexes_multiline_text_with_placeholder() {
    let source = "body = \"\"\"\nHello, {name}\n\"\"\"\n";
    let tokens = lex(source).expect("source lexes");
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("body".into()),
            TokenKind::Whitespace,
            TokenKind::Equals,
            TokenKind::TripleQuote,
            TokenKind::RawText("\n".into()),
            TokenKind::RawText("Hello, ".into()),
            TokenKind::LBrace,
            TokenKind::Ident("name".into()),
            TokenKind::RBrace,
            TokenKind::RawText("\n".into()),
            TokenKind::TripleQuote,
            TokenKind::Newline,
        ]
    );
}

#[test]
fn schema_lexer_keeps_type_alias_rhs_in_code_mode() {
    let tokens =
        lex_schema("type ShortDate = Date @date(style = \"short\")\n").expect("source lexes");
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();

    assert!(kinds.contains(&TokenKind::Ident("Date".into())));
    assert!(kinds.contains(&TokenKind::At));
    assert!(kinds.contains(&TokenKind::String("short".into())));
    assert!(!kinds
        .iter()
        .any(|kind| matches!(kind, TokenKind::RawText(_))));
}

#[test]
fn reports_byte_spans() {
    let tokens = lex("x = й\n").expect("source lexes");
    assert_eq!(tokens[0].span, Span::new(0, 1));
    assert_eq!(tokens[2].span, Span::new(2, 3));
    assert_eq!(tokens[3].span, Span::new(3, 6));
}

#[test]
fn recovers_after_invalid_code_token() {
    let output = lex_with_recovery("first # bad\nsecond()\n");

    assert_eq!(output.errors.len(), 1);
    assert_eq!(output.errors[0].span, Span::new(6, 7));
    assert!(output
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::Error("#".into())));
    assert!(output
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::Ident("second".into())));
}

#[test]
fn strict_lex_reports_first_recovery_error() {
    let error = lex("first # bad\n").expect_err("invalid token is reported");

    assert_eq!(error.span, Span::new(6, 7));
}

#[test]
fn reports_unterminated_placeholder_with_recovered_prefix() {
    let output = lex_with_recovery("value = {name\nnext = ok\n");

    assert!(output
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::Ident("next".into())));
    assert_eq!(output.errors.len(), 1);
    assert_eq!(output.errors[0].message, "unterminated placeholder");
}

#[test]
fn decodes_string_escapes() {
    let tokens =
        lex_schema(r#"type Label = String @number(note = "a\n\u{41}\\\"")"#).expect("source lexes");

    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::String("a\nA\\\"".into())));
}

#[test]
fn escaped_braces_are_text_not_placeholders() {
    let tokens = lex("value = {{left}} and {{right}}\n").expect("source lexes");
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();

    assert_eq!(
        kinds
            .iter()
            .filter(|kind| matches!(kind, TokenKind::LBrace | TokenKind::RBrace))
            .count(),
        0
    );
    assert!(kinds.contains(&TokenKind::RawText("{".into())));
    assert!(kinds.contains(&TokenKind::RawText("}".into())));
}

#[test]
fn malformed_equals_does_not_poison_following_line() {
    let output = lex_with_recovery("=\nnext = valid\n");

    assert!(output
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::Ident("next".into())));
    assert!(output
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::RawText(" valid".into())));
}

#[test]
fn lexer_schema_snapshot_matches_committed_fixture() {
    let source = include_str!("../../../../tests/fixtures/golden/schema/shop.lgs");
    let tokens = lex_schema(source).expect("schema fixture lexes");

    assert_snapshot(
        "tests/fixtures/golden/snapshots/lexer-schema.tokens",
        &render_tokens(&tokens),
    );
}

#[test]
fn lexer_locale_snapshot_matches_committed_fixture() {
    let source = include_str!("../../../../tests/fixtures/golden/locale/ru.lgl");
    let tokens = lex(source).expect("locale fixture lexes");

    assert_snapshot(
        "tests/fixtures/golden/snapshots/lexer-locale.tokens",
        &render_tokens(&tokens),
    );
}

#[test]
fn vscode_schema_grammar_snapshot_matches_the_lexer() {
    let source = include_str!("../../../../tests/fixtures/golden/syntax/all.lgs");
    let tokens = lex_schema(source).expect("VS Code schema fixture lexes");

    assert_snapshot(
        "tests/fixtures/golden/snapshots/vscode-schema.tokens",
        &render_tokens(&tokens),
    );
}

#[test]
fn vscode_locale_grammar_snapshot_matches_the_lexer() {
    let source = include_str!("../../../../tests/fixtures/golden/syntax/all.lgl");
    let tokens = lex(source).expect("VS Code locale fixture lexes");

    assert_snapshot(
        "tests/fixtures/golden/snapshots/vscode-locale.tokens",
        &render_tokens(&tokens),
    );
}
