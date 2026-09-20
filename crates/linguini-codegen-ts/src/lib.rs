mod ecmascript;
mod module;
mod plural;

pub use ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaReExport,
    EcmaReExportBindings, EcmaScriptTarget, EcmaSource, EcmaStatement, EcmaStatementKind,
    RenderedEcmaModule,
};
pub use module::{
    compile_javascript_bundler_message_artifact_module, compile_javascript_bundler_message_module,
    compile_javascript_bundler_semantic_module, compile_typescript_bundler_message_artifact_module,
    compile_typescript_bundler_message_module, compile_typescript_bundler_semantic_module,
    compile_typescript_message_module, generate_javascript_schema_files,
    generate_typescript_project_files, render_jsdoc_type, render_typescript_type,
    CompiledJavaScriptMessageModule, CompiledJavaScriptSemanticModule,
    CompiledTypeScriptMessageModule, CompiledTypeScriptSemanticModule, EcmaGeneratedFile,
    TypeModel, TypeScriptCodegenError, TypeScriptFramework, TypeScriptGeneratedFile,
    TypeScriptLinkMode, TypeScriptLocaleModule, TypeScriptLocalePrefixMode,
    TypeScriptLocaleRuntimeArtifact, TypeScriptLocaleSource, TypeScriptLocaleSwitchPlan,
    TypeScriptMessageArtifact, TypeScriptOptions, TypeScriptProjectOptions,
    TypeScriptSemanticArtifact, TypeScriptSemanticImport, TypeScriptSemanticSymbolKind,
    TypeScriptWebFeatures, TypeScriptWebOptions, TypeScriptWebSwitchRoute,
    ValidatedTypeScriptProject,
};
pub use plural::generate_plural_function;

#[cfg(test)]
mod tests;
