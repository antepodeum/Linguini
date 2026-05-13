mod lower;
mod model;

pub use lower::{lower_locale, lower_schema};
pub use model::{
    IrBranch, IrBranchPattern, IrExpression, IrForm, IrFormEntry, IrFormVariant, IrFormatter,
    IrFormatterArgument, IrFunction, IrFunctionBranch, IrMessage, IrModule, IrParameter, IrText,
    IrTextPart, IrValue,
};

pub const CRATE_PURPOSE: &str = "target-independent IR";

#[cfg(test)]
mod tests;
