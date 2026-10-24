use linguini_codegen_ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement, RenderedEcmaModule,
};
pub fn compile_svelte_module(host: Option<&str>) -> RenderedEcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::JavaScript,
        "svelte.js",
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "./index.js".into(),
        bindings: EcmaImportBindings::Namespace("runtime".into()),
    });
    module.push_import(EcmaImport::named(
        "./svelte-locale.svelte.js",
        vec![EcmaNamedImport::new("localeState", "localeState")],
    ));
    module.push_import(EcmaImport::named(
        crate::runtime_module(host),
        vec![EcmaNamedImport::new(
            "createLinguiniRune",
            "createLinguiniRune",
        )],
    ));
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "svelte/facade",
        minijinja::context! {},
    )));
    module.render(&[])
}
pub(crate) fn declaration(host: Option<&str>) -> String {
    crate::templates::render(
        "svelte/facade_declarations",
        minijinja::context! {runtime=>crate::runtime_module(host)},
    )
}
