mod ast;
mod lexer;
mod parser;
mod token;

pub const SCHEMA_EXTENSION: &str = "lqs";
pub const LOCALE_EXTENSION: &str = "lgl";

pub use ast::{
    Annotation, AnnotationArgument, BranchPattern, DocComment, EnumDeclaration, Expression,
    FormAttribute, FormDeclaration, FormEntry, FormVariant, FunctionBranch, FunctionDeclaration,
    LocaleDeclaration, LocaleFile, LocaleValue, MapBranch, MessageGroup, MessageImplementation,
    MessageImplementationGroup, MessageSignature, Name, Parameter, Placeholder, RawText,
    SchemaDeclaration, SchemaFile, StringLiteral, TextPart, TextPattern, TypeAliasDeclaration,
};
pub use lexer::{
    lex, lex_schema, lex_schema_with_recovery, lex_with_recovery, LexError, LexOutput,
};
pub use parser::{
    parse_locale, parse_locale_with_recovery, parse_schema, parse_schema_with_recovery, ParseError,
    ParseOutput,
};
pub use token::{Span, Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::{
        lex, lex_schema, lex_with_recovery, parse_locale, parse_locale_with_recovery, parse_schema,
        parse_schema_with_recovery, BranchPattern, FormEntry, LocaleDeclaration, LocaleValue,
        SchemaDeclaration, Span, TextPart, Token, TokenKind, LOCALE_EXTENSION, SCHEMA_EXTENSION,
    };

    #[test]
    fn dsl_extensions_match_spec() {
        assert_eq!(SCHEMA_EXTENSION, "lqs");
        assert_eq!(LOCALE_EXTENSION, "lgl");
    }

    #[test]
    fn lexes_schema_fixture_tokens() {
        let source = include_str!("../../../tests/fixtures/golden/schema/shop.lqs");
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
        let source = include_str!("../../../tests/fixtures/golden/schema/shop.lqs");
        let tokens = lex_schema(source).expect("schema fixture lexes");

        assert_eq!(
            render_tokens(&tokens),
            include_str!("../../../tests/fixtures/golden/snapshots/lexer-schema.tokens")
        );
    }

    #[test]
    fn lexer_locale_snapshot_matches_committed_fixture() {
        let source = include_str!("../../../tests/fixtures/golden/locale/ru.lgl");
        let tokens = lex(source).expect("locale fixture lexes");

        assert_eq!(
            render_tokens(&tokens),
            include_str!("../../../tests/fixtures/golden/snapshots/lexer-locale.tokens")
        );
    }

    #[test]
    fn parses_schema_fixture() {
        let source = include_str!("../../../tests/fixtures/golden/schema/shop.lqs");
        let schema = parse_schema(source).expect("schema fixture parses");

        assert_eq!(schema.declarations.len(), 8);
        match &schema.declarations[0] {
            SchemaDeclaration::Enum(declaration) => {
                assert_eq!(declaration.name.value, "Fruit");
                assert_eq!(declaration.variants.len(), 3);
            }
            other => panic!("expected enum, got {other:?}"),
        }
        match &schema.declarations[4] {
            SchemaDeclaration::Message(declaration) => {
                assert_eq!(declaration.name.value, "delivery");
                assert_eq!(declaration.parameters[0].name.value, "fruit");
                assert_eq!(declaration.parameters[0].ty.value, "Fruit");
            }
            other => panic!("expected message, got {other:?}"),
        }
        match &schema.declarations[7] {
            SchemaDeclaration::Group(declaration) => {
                assert_eq!(declaration.name.value, "email_input");
                assert_eq!(declaration.messages.len(), 3);
            }
            other => panic!("expected group, got {other:?}"),
        }
    }

    #[test]
    fn schema_parser_recovery_reports_invalid_fixture_diagnostics() {
        let source =
            include_str!("../../../tests/fixtures/invalid/schema/missing-message-paren.lqs");
        let output = parse_schema_with_recovery(source);

        assert!(!output.errors.is_empty());
        assert!(output.errors.iter().any(|error| error.span.start >= 25));
        assert!(parse_schema(source).is_err());
    }

    #[test]
    fn parses_schema_docs_type_alias_annotations_and_groups() {
        let source = r#"/// money amount
type Money = Decimal @currency

type ShortDate = Date @date(style = "short")

email_input {
  label()
  placeholder()
}
"#;
        let schema = parse_schema(source).expect("schema parses");

        assert_eq!(schema.declarations.len(), 3);
        match &schema.declarations[0] {
            SchemaDeclaration::TypeAlias(declaration) => {
                assert_eq!(declaration.docs[0].text, " money amount");
                assert_eq!(declaration.name.value, "Money");
                assert_eq!(declaration.target.value, "Decimal");
                assert_eq!(declaration.annotations[0].name.value, "currency");
            }
            other => panic!("expected type alias, got {other:?}"),
        }
        match &schema.declarations[1] {
            SchemaDeclaration::TypeAlias(declaration) => {
                assert_eq!(declaration.annotations[0].arguments[0].name.value, "style");
                assert_eq!(declaration.annotations[0].arguments[0].value.value, "short");
            }
            other => panic!("expected type alias, got {other:?}"),
        }
        match &schema.declarations[2] {
            SchemaDeclaration::Group(declaration) => {
                assert_eq!(declaration.name.value, "email_input");
                assert_eq!(declaration.messages.len(), 2);
                assert_eq!(declaration.messages[0].name.value, "label");
            }
            other => panic!("expected group, got {other:?}"),
        }
    }

    #[test]
    fn parses_locale_fixture() {
        let source = include_str!("../../../tests/fixtures/golden/locale/ru.lgl");
        let locale = parse_locale(source).expect("locale fixture parses");

        assert_eq!(locale.declarations.len(), 9);
        match &locale.declarations[1] {
            LocaleDeclaration::Form(form) => {
                assert_eq!(form.name.value, "Fruit");
                assert_eq!(form.variants.len(), 3);
                let apple_display = &form.variants[0].entries[4];
                match apple_display {
                    FormEntry::Attribute(attribute) => match &attribute.value {
                        LocaleValue::Object(entries) => assert_eq!(entries.len(), 2),
                        other => panic!("expected nested object, got {other:?}"),
                    },
                    other => panic!("expected nested display attribute, got {other:?}"),
                }
            }
            other => panic!("expected form, got {other:?}"),
        }
        match &locale.declarations[5] {
            LocaleDeclaration::Message(message) => {
                assert_eq!(message.name.value, "delivery");
                let placeholders = message
                    .value
                    .parts
                    .iter()
                    .filter(|part| matches!(part, TextPart::Placeholder(_)))
                    .count();
                assert_eq!(placeholders, 3);
            }
            other => panic!("expected message, got {other:?}"),
        }
    }

    #[test]
    fn locale_parser_recovery_reports_invalid_fixture_diagnostics() {
        let source = include_str!("../../../tests/fixtures/invalid/locale/broken-placeholder.lgl");
        let output = parse_locale_with_recovery(source);

        assert!(!output.errors.is_empty());
        assert_eq!(output.errors[0].message, "unterminated placeholder");
        assert!(parse_locale(source).is_err());
    }

    #[test]
    fn parses_locale_forms_functions_messages_and_placeholders() {
        let source = r#"enum gender {
  male
  female
  neuter
  other
}

form Fruit {
  apple {
    gender = neuter
    nom {
      one => яблоко
      few => яблока
      many => яблок
      other => яблока
    }
  }
}

form Size {
  small:gender {
    male => маленький
    female => маленькая
    neuter => маленькое
    other => маленький
  }
}

fn adjective(size, gender) {
  small, male => маленький
  else => обычное
}

delivery = {adjective(size, fruit.gender)} {fruit.nom(count)} {amount @currency(code = "USD")}

email_input {
  label = Email
}
"#;
        let locale = parse_locale(source).expect("locale parses");

        assert_eq!(locale.declarations.len(), 6);
        match &locale.declarations[1] {
            LocaleDeclaration::Form(form) => {
                assert_eq!(form.name.value, "Fruit");
                assert_eq!(form.variants[0].name.value, "apple");
                assert_eq!(form.variants[0].entries.len(), 2);
                match &form.variants[0].entries[1] {
                    FormEntry::Attribute(attribute) => match &attribute.value {
                        LocaleValue::Map(branches) => assert_eq!(branches.len(), 4),
                        other => panic!("expected map value, got {other:?}"),
                    },
                    other => panic!("expected form attribute, got {other:?}"),
                }
            }
            other => panic!("expected form, got {other:?}"),
        }
        match &locale.declarations[2] {
            LocaleDeclaration::Form(form) => {
                assert_eq!(
                    form.variants[0]
                        .selector
                        .as_ref()
                        .expect("selector exists")
                        .value,
                    "gender"
                );
                assert!(matches!(form.variants[0].entries[0], FormEntry::Branch(_)));
            }
            other => panic!("expected selector form, got {other:?}"),
        }
        match &locale.declarations[3] {
            LocaleDeclaration::Function(function) => {
                assert_eq!(function.name.value, "adjective");
                assert_eq!(function.parameters.len(), 2);
                assert!(matches!(
                    function.branches[1].pattern,
                    BranchPattern::Else(_)
                ));
            }
            other => panic!("expected function, got {other:?}"),
        }
        match &locale.declarations[4] {
            LocaleDeclaration::Message(message) => {
                let placeholders = message
                    .value
                    .parts
                    .iter()
                    .filter(|part| matches!(part, TextPart::Placeholder(_)))
                    .count();
                assert_eq!(placeholders, 3);
                let annotated_amount = message.value.parts.iter().any(|part| match part {
                    TextPart::Placeholder(placeholder) => {
                        placeholder.expression.path[0].value == "amount"
                            && placeholder.expression.annotations[0].name.value == "currency"
                    }
                    TextPart::Text(_) => false,
                });
                assert!(annotated_amount);
            }
            other => panic!("expected message, got {other:?}"),
        }
        match &locale.declarations[5] {
            LocaleDeclaration::Group(group) => {
                assert_eq!(group.name.value, "email_input");
                assert_eq!(group.messages[0].name.value, "label");
            }
            other => panic!("expected group, got {other:?}"),
        }
    }

    #[test]
    fn parses_locale_override_declaration() {
        let locale = parse_locale("override enum gender { other }\n").expect("locale parses");

        match &locale.declarations[0] {
            LocaleDeclaration::Override(declaration) => match declaration.as_ref() {
                LocaleDeclaration::Enum(declaration) => {
                    assert_eq!(declaration.name.value, "gender")
                }
                other => panic!("expected enum override, got {other:?}"),
            },
            other => panic!("expected override, got {other:?}"),
        }
    }

    fn render_tokens(tokens: &[Token]) -> String {
        tokens
            .iter()
            .map(|token| {
                format!(
                    "{:?} @ {}..{}\n",
                    token.kind, token.span.start, token.span.end
                )
            })
            .collect()
    }
}
