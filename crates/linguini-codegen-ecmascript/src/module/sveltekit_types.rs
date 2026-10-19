//! SvelteKit declaration companions and ambient app types.
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement,
};
pub(super) fn generate_control_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "sveltekit-control.d.ts",
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "@sveltejs/kit".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![
            EcmaNamedImport::new("Handle", "Handle"),
            EcmaNamedImport::new("Reroute", "Reroute"),
            EcmaNamedImport::new("ServerLoad", "ServerLoad"),
        ]),
    });
    module.push_import(EcmaImport {
        specifier: "./locale.js".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![
            EcmaNamedImport::new("Locale", "Locale"),
            EcmaNamedImport::new("TextDirection", "TextDirection"),
        ]),
    });
    module.push_import(EcmaImport {
        specifier: "./web.js".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
            "LinguiniLocaleRequestContext",
            "LinguiniLocaleRequestContext",
        )]),
    });
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render("sveltekit/control_declarations", minijinja::context! {}),
        None,
    ));
    module.render_code()
}
pub(super) fn generate_ambient_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "linguini-app.d.ts",
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "@sveltejs/kit".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![
            EcmaNamedImport::new("Handle", "Handle"),
            EcmaNamedImport::new("Reroute", "Reroute"),
            EcmaNamedImport::new("ServerLoad", "ServerLoad"),
        ]),
    });
    module.push_import(EcmaImport {
        specifier: "./web.js".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![
            EcmaNamedImport::new(
                "LinguiniLocaleRequestContext",
                "LinguiniLocaleRequestContext",
            ),
            EcmaNamedImport::new("LinguiniRequestContext", "LinguiniRequestContext"),
        ]),
    });
    module.push_import(EcmaImport {
        specifier: "./index.js".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![
            EcmaNamedImport::new("Locale", "Locale"),
            EcmaNamedImport::new("Linguini", "Linguini"),
            EcmaNamedImport::new("TextDirection", "TextDirection"),
        ]),
    });
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render("sveltekit/ambient_declarations", minijinja::context! {}),
        None,
    ));
    module.render_code()
}
