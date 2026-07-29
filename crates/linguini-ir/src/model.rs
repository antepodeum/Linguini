pub use linguini_core::FormatterKind as IrFormatterKind;
use linguini_syntax::Span;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IrModule {
    pub enums: Vec<IrEnum>,
    pub type_aliases: Vec<IrTypeAlias>,
    pub variables: Vec<IrVariable>,
    pub messages: Vec<IrMessage>,
    pub forms: Vec<IrForm>,
    pub functions: Vec<IrFunction>,
    /// Lossless declaration provenance, including entries superseded by `override`.
    pub origins: Vec<IrOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaIr(pub(crate) IrModule);

impl SchemaIr {
    pub fn as_module(&self) -> &IrModule {
        &self.0
    }

    pub fn into_module(self) -> IrModule {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocaleIr(pub(crate) IrModule);

impl LocaleIr {
    pub fn as_module(&self) -> &IrModule {
        &self.0
    }

    pub fn into_module(self) -> IrModule {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IrSymbolKind {
    Enum,
    TypeAlias,
    Variable,
    Message,
    Form,
    Function,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrOrigin {
    pub kind: IrSymbolKind,
    pub name: String,
    pub span: Span,
    pub is_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrTypeAlias {
    pub name: String,
    pub target: String,
    pub docs: Vec<String>,
    pub formatters: Vec<IrFormatter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrEnum {
    pub name: String,
    pub docs: Vec<String>,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrMessage {
    pub name: String,
    pub docs: Vec<String>,
    pub parameters: Vec<IrParameter>,
    pub body: Option<IrText>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrParameter {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrVariable {
    pub name: String,
    pub docs: Vec<String>,
    pub value: IrText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrForm {
    pub name: String,
    pub docs: Vec<String>,
    pub variants: Vec<IrFormVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFormVariant {
    pub name: String,
    pub entries: Vec<IrFormEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrFormEntry {
    Attribute {
        name: String,
        parameters: Vec<IrFunctionParameter>,
        value: IrValue,
    },
    Branch(IrBranch),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrValue {
    Text(IrText),
    Map(Vec<IrBranch>),
    Object(Vec<IrFormEntry>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunction {
    pub kind: IrFunctionKind,
    pub name: String,
    pub docs: Vec<String>,
    pub parameters: Vec<IrFunctionParameter>,
    pub branches: Vec<IrFunctionBranch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrFunctionKind {
    Form,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunctionParameter {
    pub name: Option<String>,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunctionBranch {
    pub key: String,
    pub value: IrFunctionBranchValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrFunctionBranchValue {
    Text(IrText),
    Dispatch(Vec<IrFunctionBranch>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBranch {
    pub keys: Vec<String>,
    pub value: IrText,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrText {
    pub parts: Vec<IrTextPart>,
    pub mode: IrTextBlockMode,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrTextBlockMode {
    Inline,
    Dedented,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrTextPart {
    Text(String),
    Placeholder(IrExpression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrExpression {
    pub kind: IrExpressionKind,
    pub path: Vec<String>,
    pub arguments: Vec<IrExpression>,
    pub formatters: Vec<IrFormatter>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrExpressionKind {
    Reference,
    Call,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFormatter {
    pub kind: IrFormatterKind,
    pub arguments: Vec<IrFormatterArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFormatterArgument {
    pub name: String,
    pub value: String,
}
