mod common;
mod lexer;
mod parser;

use super::{parse_locale, LOCALE_EXTENSION, SCHEMA_EXTENSION};

#[test]
fn dsl_extensions_match_spec() {
    assert_eq!(SCHEMA_EXTENSION, "lqs");
    assert_eq!(LOCALE_EXTENSION, "lgl");
}

#[test]
fn parse_errors_hide_internal_token_debug_representation() {
    let error = parse_locale("fn delivered(gender) {\n")
        .expect_err("source should not parse")
        .remove(0);

    assert!(!error.message.contains("Ident("), "{}", error.message);
    assert!(!error.message.contains("Span"), "{}", error.message);
    assert!(error.message.contains("delivered") || error.message.contains("identifier"));
}
