mod ast;
mod lexer;
mod parser;
mod token;

pub const SCHEMA_EXTENSION: &str = "lqs";
pub const LOCALE_EXTENSION: &str = "lgl";

pub use ast::{
    Annotation, AnnotationArgument, DocComment, EnumDeclaration, Expression, FormAttribute,
    FormDeclaration, FormEntry, FormVariant, FunctionBranch, FunctionBranchValue,
    FunctionDeclaration, FunctionParameter, LocaleDeclaration, LocaleFile, LocaleValue, MapBranch,
    MessageGroup, MessageImplementation, MessageImplementationGroup, MessageSignature, Name,
    Parameter, Placeholder, RawText, SchemaDeclaration, SchemaFile, StringLiteral, TextPart,
    TextPattern, TypeAliasDeclaration,
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
mod tests;
