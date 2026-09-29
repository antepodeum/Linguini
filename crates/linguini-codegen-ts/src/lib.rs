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
    compile_javascript_bundler_semantic_module, compile_javascript_locale_artifact_module,
    compile_javascript_locale_globals_artifact_module,
    compile_javascript_locale_runtime_artifact_module, compile_javascript_project_artifact_module,
    compile_javascript_web_accept_language_module, compile_javascript_web_cookie_module,
    compile_javascript_web_link_transform_module, compile_javascript_web_local_storage_module,
    compile_javascript_web_path_module, compile_javascript_web_routes_module,
    compile_typescript_bundler_message_artifact_module, compile_typescript_bundler_message_module,
    compile_typescript_bundler_semantic_module, compile_typescript_locale_artifact_module,
    compile_typescript_locale_globals_artifact_module,
    compile_typescript_locale_runtime_artifact_module, compile_typescript_message_module,
    compile_typescript_project_artifact_module, compile_typescript_web_accept_language_module,
    compile_typescript_web_cookie_module, compile_typescript_web_link_transform_module,
    compile_typescript_web_local_storage_module, compile_typescript_web_path_module,
    compile_typescript_web_routes_module, generate_javascript_schema_files,
    generate_typescript_project_files, render_jsdoc_type, render_typescript_type,
    CompiledJavaScriptLocaleGlobalsModule, CompiledJavaScriptLocaleModule,
    CompiledJavaScriptLocaleRuntimeModule, CompiledJavaScriptMessageModule,
    CompiledJavaScriptProjectModule, CompiledJavaScriptSemanticModule,
    CompiledTypeScriptLocaleGlobalsModule, CompiledTypeScriptLocaleModule,
    CompiledTypeScriptLocaleRuntimeModule, CompiledTypeScriptMessageModule,
    CompiledTypeScriptProjectModule, CompiledTypeScriptSemanticModule, EcmaGeneratedFile,
    TypeModel, TypeScriptCodegenError, TypeScriptFramework, TypeScriptGeneratedFile,
    TypeScriptLinkMode, TypeScriptLocaleArtifact, TypeScriptLocaleArtifactKind,
    TypeScriptLocaleGlobalsArtifact, TypeScriptLocaleModule, TypeScriptLocalePrefixMode,
    TypeScriptLocaleRuntimeArtifact, TypeScriptLocaleSource, TypeScriptLocaleSwitchPlan,
    TypeScriptMessageArtifact, TypeScriptOptions, TypeScriptProjectArtifact,
    TypeScriptProjectArtifactKind, TypeScriptProjectOptions, TypeScriptSemanticArtifact,
    TypeScriptSemanticImport, TypeScriptSemanticSymbolKind, TypeScriptWebFeatures,
    TypeScriptWebOptions, TypeScriptWebSwitchRoute, ValidatedTypeScriptProject,
};
pub use module::{
    compile_javascript_svelte_control_module, compile_javascript_svelte_effects_module,
    compile_javascript_svelte_locale_module, compile_javascript_svelte_module,
    compile_javascript_web_runtime_links_module, compile_javascript_web_server_cookie_module,
    compile_javascript_web_switch_route_module, compile_typescript_svelte_control_module,
    compile_typescript_svelte_effects_module, compile_typescript_svelte_locale_module,
    compile_typescript_svelte_module, compile_typescript_web_runtime_links_module,
    compile_typescript_web_server_cookie_module, compile_typescript_web_switch_route_module,
    SvelteLocaleMode,
};
pub use plural::generate_plural_function;

#[cfg(test)]
mod tests;
