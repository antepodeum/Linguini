mod ecmascript;
mod module;
mod plural;
mod source_paths;
mod template_engine;
mod templates;
pub use template_engine::EcmaTemplateEngine;

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
    EcmaCodegenError, EcmaGeneratedFile, EcmaLinkMode, EcmaLocaleArtifact, EcmaLocaleArtifactKind,
    EcmaLocaleGlobalsArtifact, EcmaLocaleModule, EcmaLocalePrefixMode, EcmaLocaleRuntimeArtifact,
    EcmaLocaleSource, EcmaLocaleSwitchPlan, EcmaMessageArtifact, EcmaOptions, EcmaProjectArtifact,
    EcmaProjectArtifactKind, EcmaProjectOptions, EcmaSemanticArtifact, EcmaSemanticImport,
    EcmaSemanticSymbolKind, EcmaWebFeatures, EcmaWebOptions, EcmaWebSwitchRoute, TypeModel,
    ValidatedEcmaProject,
};
pub use module::{
    compile_web_runtime_links_module, compile_web_server_cookie_module,
    compile_web_switch_route_module, generate_web_declaration,
    generate_web_server_cookie_declaration, generate_web_switch_route_declaration,
    validate_web_options,
};
pub use plural::generate_plural_function;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_runtime;

pub use module::render_web_options;

/// Collect one compiled module and its optional source map without re-rendering its body.
pub fn generated_module_files(
    module: RenderedEcmaModule,
    source_maps: bool,
) -> Vec<EcmaGeneratedFile> {
    let path = module.output.path().to_owned();
    if source_maps {
        return vec![
            EcmaGeneratedFile {
                path,
                contents: module.code,
            },
            EcmaGeneratedFile {
                path: module.output.source_map_path(),
                contents: module.source_map,
            },
        ];
    }
    let trailer = format!(
        "//# sourceMappingURL={}\n",
        module
            .output
            .source_map_path()
            .rsplit('/')
            .next()
            .expect("map file name")
    );
    vec![EcmaGeneratedFile {
        path,
        contents: module
            .code
            .strip_suffix(&trailer)
            .expect("owned map trailer")
            .to_owned(),
    }]
}

pub use source_paths::rebase_map_sources;
