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
    generate_typescript_project_files, render_jsdoc_type, render_typescript_type,
    CompiledTypeScriptMessageModule, CompiledTypeScriptSemanticModule, TypeModel,
    TypeScriptCodegenError, TypeScriptFramework, TypeScriptGeneratedFile, TypeScriptLinkMode,
    TypeScriptLocaleModule, TypeScriptLocalePrefixMode, TypeScriptLocaleRuntimeArtifact,
    TypeScriptLocaleSource, TypeScriptLocaleSwitchPlan, TypeScriptMessageArtifact,
    TypeScriptOptions, TypeScriptProjectOptions, TypeScriptSemanticArtifact,
    TypeScriptSemanticImport, TypeScriptSemanticSymbolKind, TypeScriptWebFeatures,
    TypeScriptWebOptions, TypeScriptWebSwitchRoute, ValidatedTypeScriptProject,
};
pub use plural::generate_plural_function;

#[cfg(test)]
mod tests;
