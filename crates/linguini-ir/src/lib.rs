mod lower;
mod model;
mod namespace;
mod reference;

pub use lower::{lower_locale, lower_locale_typed, lower_schema, lower_schema_typed};
pub use model::{
    IrBranch, IrEnum, IrExpression, IrExpressionKind, IrForm, IrFormEntry, IrFormVariant,
    IrFormatter, IrFormatterArgument, IrFormatterKind, IrFunction, IrFunctionBranch,
    IrFunctionBranchValue, IrFunctionKind, IrFunctionParameter, IrMessage, IrModule, IrOrigin,
    IrParameter, IrSymbolKind, IrText, IrTextBlockMode, IrTextPart, IrTypeAlias, IrValue,
    IrVariable, LocaleIr, SchemaIr,
};
pub use namespace::qualify_module;
pub use reference::{
    ensure_no_unresolved_references, validate_ir, validate_typed_ir, IrReferenceError,
    IrRelatedError, ValidatedIr, BUILTIN_PLURAL,
};

pub const CRATE_PURPOSE: &str = "target-independent IR";

#[cfg(test)]
mod tests;
