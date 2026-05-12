mod lexer;
mod token;

pub const SCHEMA_EXTENSION: &str = "lqs";
pub const LOCALE_EXTENSION: &str = "lgl";

pub use lexer::{lex, LexError};
pub use token::{Span, Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::{lex, Span, TokenKind, LOCALE_EXTENSION, SCHEMA_EXTENSION};

    #[test]
    fn dsl_extensions_match_spec() {
        assert_eq!(SCHEMA_EXTENSION, "lqs");
        assert_eq!(LOCALE_EXTENSION, "lgl");
    }

    #[test]
    fn lexes_schema_fixture_tokens() {
        let source = include_str!("../../../tests/fixtures/golden/schema/shop.lqs");
        let tokens = lex(source).expect("schema fixture lexes");
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
        let source = include_str!("../../../tests/fixtures/golden/locale/ru.lgl");
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
                TokenKind::RawText(" ".into()),
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
    fn reports_byte_spans() {
        let tokens = lex("x = й\n").expect("source lexes");
        assert_eq!(tokens[0].span, Span::new(0, 1));
        assert_eq!(tokens[2].span, Span::new(2, 3));
        assert_eq!(tokens[3].span, Span::new(3, 6));
    }
}
