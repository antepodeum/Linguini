use crate::Span;
pub use linguini_core::FormatterKind;

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
pub struct LocaleFile {
    pub declarations: Vec<LocaleDeclaration>,
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
pub enum LocaleDeclaration {
    Enum(EnumDeclaration),
    Form(FormDeclaration),
    Function(FunctionDeclaration),
    Message(MessageImplementation),
    Group(MessageImplementationGroup),
    Override(Box<LocaleDeclaration>),
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
    pub kind: FormatterKind,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDeclaration {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub variants: Vec<FormVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormVariant {
    pub name: Name,
    pub entries: Vec<FormEntry>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormEntry {
    Attribute(FormAttribute),
    Branch(MapBranch),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormAttribute {
    pub name: Name,
    pub value: LocaleValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocaleValue {
    Text(TextPattern),
    Map(Vec<MapBranch>),
    Object(Vec<FormEntry>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDeclaration {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub parameters: Vec<FunctionParameter>,
    pub branches: Vec<FunctionBranch>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: Option<Name>,
    pub ty: Name,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionBranch {
    pub key: Name,
    pub value: FunctionBranchValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionBranchValue {
    Text(TextPattern),
    Dispatch(Vec<FunctionBranch>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapBranch {
    pub keys: Vec<Name>,
    pub value: TextPattern,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageImplementation {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub value: TextPattern,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageImplementationGroup {
    pub docs: Vec<DocComment>,
    pub name: Name,
    pub messages: Vec<MessageImplementation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPattern {
    pub parts: Vec<TextPart>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextPart {
    Text(RawText),
    Placeholder(Placeholder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawText {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder {
    pub expression: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub path: Vec<Name>,
    pub arguments: Vec<Expression>,
    pub annotations: Vec<Annotation>,
    pub span: Span,
}
