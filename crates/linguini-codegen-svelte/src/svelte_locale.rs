use linguini_codegen_ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement, RenderedEcmaModule,
};
pub fn compile_svelte_locale_module(host: Option<&str>) -> RenderedEcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::JavaScript,
        "svelte-locale.svelte.js",
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "./locale.js".into(),
        bindings: EcmaImportBindings::Namespace("locale".into()),
    });
    let mut names = vec![EcmaNamedImport::new(
        "createLocaleState",
        "createLocaleState",
    )];
    if host.is_some() {
        names.push(EcmaNamedImport::new("getRequestLocale", "getRequestLocale"));
    }
    module.push_import(EcmaImport::named(crate::runtime_module(host), names));
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "svelte/locale",
        minijinja::context! {request_locale=>host.is_some()},
    )));
    module.render(&[])
}
pub(crate) fn declaration(host: Option<&str>) -> String {
    crate::templates::render(
        "svelte/locale_declarations",
        minijinja::context! {runtime=>crate::runtime_module(host)},
    )
}
