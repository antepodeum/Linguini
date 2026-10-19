//! Declaration interfaces for the context-only and web-enabled Svelte facades.

use linguini_codegen_ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaReExport, EcmaStatement,
};

pub(super) fn generate_svelte_declaration(web: bool) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "svelte.d.ts",
        None,
    ));
    let mut names = vec!["Locale", "Linguini"];
    if !web {
        names.push("TextDirection");
    }
    module.push_import(EcmaImport {
        specifier: "./index.js".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(
            names
                .into_iter()
                .map(|name| EcmaNamedImport::new(name, name))
                .collect(),
        ),
    });
    if web {
        module.push_import(EcmaImport {
            specifier: "./svelte-control.js".to_owned(),
            bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
                "LinguiniSvelteControl",
                "LinguiniSvelteControl",
            )]),
        });
        module.push_re_export(EcmaReExport::type_named(
            "./svelte-control.js",
            vec![EcmaNamedImport::new(
                "LinguiniSetLocaleOptions",
                "LinguiniSetLocaleOptions",
            )],
        ));
    }
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render("svelte/facade_declarations", minijinja::context! { web }),
        None,
    ));
    module.render_code()
}
