mod module;
mod plural;

pub use module::{
    generate_typescript_files, generate_typescript_module, TypeScriptGeneratedFile,
    TypeScriptOptions,
};
pub use plural::generate_plural_function;

pub const CRATE_PURPOSE: &str = "TypeScript code generation";

#[cfg(test)]
mod tests;
