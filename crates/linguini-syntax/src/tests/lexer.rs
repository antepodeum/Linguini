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
            TokenKind::Newline,
            TokenKind::RawText("Hello, ".into()),
            TokenKind::LBrace,
            TokenKind::Ident("name".into()),
            TokenKind::RBrace,
            TokenKind::Newline,
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
