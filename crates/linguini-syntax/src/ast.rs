use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocComment {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaFile {
    pub declarations: Vec<SchemaDeclaration>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaDeclaration {
    Enum(EnumDeclaration),
    TypeAlias(TypeAliasDeclaration),
    Message(MessageSignature),
    Group(MessageGroup),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDeclaration {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub variants: Vec<Name>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAliasDeclaration {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub target: Name,
    pub annotations: Vec<Annotation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub parameters: Vec<Parameter>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageGroup {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub messages: Vec<MessageSignature>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Name,
    pub ty: Name,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub name: Name,
    pub arguments: Vec<AnnotationArgument>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationArgument {
    pub name: Name,
    pub value: StringLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLiteral {
    pub value: String,
    pub span: Span,
}
