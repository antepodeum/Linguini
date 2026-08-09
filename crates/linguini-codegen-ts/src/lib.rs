mod ecmascript;
mod module;
mod plural;

pub use ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaNamedImport, EcmaSource, EcmaStatement,
    RenderedEcmaModule,
};
pub use module::{
    compile_typescript_bundler_message_artifact_module, compile_typescript_bundler_message_module,
    compile_typescript_bundler_semantic_module, compile_typescript_message_module,
    generate_typescript_project_files, CompiledTypeScriptMessageModule,
    CompiledTypeScriptSemanticModule, TypeScriptCodegenError, TypeScriptFramework,
    TypeScriptGeneratedFile, TypeScriptLocaleModule, TypeScriptLocaleRuntimeArtifact,
    TypeScriptLocaleSource, TypeScriptMessageArtifact, TypeScriptOptions, TypeScriptProjectOptions,
    TypeScriptSemanticArtifact, TypeScriptSemanticImport, TypeScriptSemanticSymbolKind,
    TypeScriptWebOptions, ValidatedTypeScriptProject,
};
pub use plural::generate_plural_function;

pub const CRATE_PURPOSE: &str = "TypeScript code generation";

#[cfg(test)]
mod tests;
