pub use linguini_core::FormatterKind as IrFormatterKind;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IrModule {
    pub enums: Vec<IrEnum>,
    pub type_aliases: Vec<IrTypeAlias>,
    pub messages: Vec<IrMessage>,
    pub forms: Vec<IrForm>,
    pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrTypeAlias {
    pub name: String,
    pub target: String,
    pub docs: Vec<String>,
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
    Attribute { name: String, value: IrValue },
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
    pub name: String,
    pub docs: Vec<String>,
    pub parameters: Vec<IrFunctionParameter>,
    pub branches: Vec<IrFunctionBranch>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrText {
    pub parts: Vec<IrTextPart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrTextPart {
    Text(String),
    Placeholder(IrExpression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrExpression {
    pub path: Vec<String>,
    pub arguments: Vec<IrExpression>,
    pub formatters: Vec<IrFormatter>,
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
