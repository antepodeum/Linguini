//! SvelteKit declaration companions and ambient app types.
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement,
};
pub(super) fn generate_control_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
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
    module.push_statement(EcmaStatement::type_declaration("export declare const linguiniHandle: Handle;\nexport declare const linguiniReroute: Reroute;\nexport declare const linguiniLoad: ServerLoad;\nexport declare const handle: Handle;\nexport declare const reroute: Reroute;\nexport declare const load: ServerLoad;\n\nexport type LinguiniServerLocaleContext<Locale extends string = string> = LinguiniLocaleRequestContext<Locale>;\n\nexport interface SerializedLinguiniContext<Locale extends string = string> {\n  locale: Locale;\n  baseLocale: Locale;\n  locales: readonly Locale[];\n  direction: TextDirection;\n  lang: Locale;\n  htmlAttrs: { lang: Locale; dir: TextDirection };\n}\n\nexport type LinguiniSvelteKitLocaleContext = LinguiniServerLocaleContext<Locale>;\nexport type LinguiniSerializedContext = SerializedLinguiniContext<Locale>;\n", None));
    module.render_code()
}
pub(super) fn generate_ambient_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
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
    module.push_statement(EcmaStatement::type_declaration("export declare const linguiniHandle: Handle;\nexport declare const linguiniReroute: Reroute;\nexport declare const linguiniLoad: ServerLoad;\nexport declare const handle: Handle;\nexport declare const reroute: Reroute;\nexport declare const load: ServerLoad;\n\nexport interface SerializedLinguiniContext<Locale extends string = string> {\n  locale: Locale;\n  baseLocale: Locale;\n  locales: readonly Locale[];\n  direction: TextDirection;\n  lang: Locale;\n  htmlAttrs: { lang: Locale; dir: TextDirection };\n}\n\ndeclare global {\n  namespace App {\n    interface Locals {\n      linguini: LinguiniLocaleRequestContext<Locale> | LinguiniRequestContext<Locale, Linguini>;\n    }\n    interface PageData {\n      linguini?: SerializedLinguiniContext<Locale>;\n    }\n  }\n}\n", None));
    module.render_code()
}
