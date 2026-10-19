mod ecmascript;
mod module;
mod plural;
mod templates;

pub use ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaReExport, EcmaReExportBindings, EcmaSource, EcmaStatement, EcmaStatementKind,
    RenderedEcmaModule,
};
pub use module::{
    compile_bundler_message_artifact_module, compile_bundler_message_module,
    compile_bundler_semantic_module, compile_ecmascript_project_files,
    compile_locale_artifact_module, compile_locale_globals_artifact_module,
    compile_locale_runtime_artifact_module, compile_message_module,
    compile_project_artifact_module, compile_web_accept_language_module, compile_web_cookie_module,
    compile_web_link_transform_module, compile_web_local_storage_module, compile_web_module,
    compile_web_path_module, compile_web_routes_module, generate_project_files,
    generate_schema_files, render_jsdoc_type, render_typescript_type,
    CompiledEcmaLocaleGlobalsModule, CompiledEcmaLocaleModule, CompiledEcmaLocaleRuntimeModule,
    CompiledEcmaMessageModule, CompiledEcmaProjectModule, CompiledEcmaSemanticModule,
    EcmaCodegenError, EcmaFramework, EcmaGeneratedFile, EcmaLinkMode, EcmaLocaleArtifact,
    EcmaLocaleArtifactKind, EcmaLocaleGlobalsArtifact, EcmaLocaleModule, EcmaLocalePrefixMode,
    EcmaLocaleRuntimeArtifact, EcmaLocaleSource, EcmaLocaleSwitchPlan, EcmaMessageArtifact,
    EcmaOptions, EcmaProjectArtifact, EcmaProjectArtifactKind, EcmaProjectOptions,
    EcmaSemanticArtifact, EcmaSemanticImport, EcmaSemanticSymbolKind, EcmaWebFeatures,
    EcmaWebOptions, EcmaWebSwitchRoute, TypeModel, ValidatedEcmaProject,
};
pub use module::{
    compile_svelte_control_module, compile_svelte_effects_module, compile_svelte_locale_module,
    compile_svelte_module, compile_sveltekit_control_module, compile_sveltekit_module,
    compile_web_runtime_links_module, compile_web_server_cookie_module,
    compile_web_switch_route_module, SvelteLocaleMode,
};
pub use plural::generate_plural_function;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_runtime;
