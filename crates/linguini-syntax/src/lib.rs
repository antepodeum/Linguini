mod ast;
mod lexer;
mod parser;
mod token;

pub const SCHEMA_EXTENSION: &str = "lgs";
pub const LOCALE_EXTENSION: &str = "lgl";

pub use ast::{
    Annotation, AnnotationArgument, DocComment, EnumDeclaration, Expression, ExpressionKind,
    FormAttribute, FormDeclaration, FormEntry, FormVariant, FormatterKind, FunctionBranch,
    FunctionBranchValue, FunctionDeclaration, FunctionKind, FunctionParameter, LocaleDeclaration,
    LocaleFile, LocaleValue, MapBranch, MessageGroup, MessageImplementation,
    MessageImplementationGroup, MessageSignature, Name, Parameter, Placeholder, RawText,
    SchemaDeclaration, SchemaFile, StringLiteral, TextBlockMode, TextPart, TextPattern,
    TypeAliasDeclaration, VariableDeclaration,
};
pub use lexer::{
    lex, lex_in, lex_schema, lex_schema_in, lex_schema_with_recovery, lex_schema_with_recovery_in,
    lex_with_recovery, lex_with_recovery_in, LexError, LexOutput,
};
pub use parser::{
    parse_locale, parse_locale_in, parse_locale_with_recovery, parse_locale_with_recovery_in,
    parse_schema, parse_schema_in, parse_schema_with_recovery, parse_schema_with_recovery_in,
    validate_locale_ast, validate_schema_ast, ParseError, ParseOutput,
};
pub use token::{SourceId, Span, Token, TokenKind};

#[cfg(test)]
mod tests;
