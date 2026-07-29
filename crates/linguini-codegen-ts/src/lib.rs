mod module;
mod plural;

pub use module::{
    generate_typescript_project_files, TypeScriptCodegenError, TypeScriptFramework,
    TypeScriptGeneratedFile, TypeScriptLocaleModule, TypeScriptLocaleSource, TypeScriptOptions,
    TypeScriptProjectOptions, TypeScriptWebOptions, ValidatedTypeScriptProject,
};
pub use plural::generate_plural_function;

pub const CRATE_PURPOSE: &str = "TypeScript code generation";

#[cfg(test)]
mod tests;
