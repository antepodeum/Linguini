//! Declaration surface shared by framework-selected Svelte browser controls.

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement,
};

pub(super) fn generate_svelte_control_declaration(sveltekit: bool) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "svelte-control.d.ts",
        None,
    ));
    for (path, names) in [
        ("./locale.js", ["Locale", "TextDirection"]),
        ("./web.js", ["AlternateLink", "LinkLocalizationAttributes"]),
    ] {
        module.push_import(EcmaImport {
            specifier: path.to_owned(),
            bindings: EcmaImportBindings::TypeNamed(
                names
                    .into_iter()
                    .map(|name| EcmaNamedImport::new(name, name))
                    .collect(),
            ),
        });
    }
    let state = if sveltekit {
        "App.PageState"
    } else {
        "unknown"
    };
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render("svelte/control_declarations", minijinja::context! { state }),
        None,
    ));
    module.render_code()
}
