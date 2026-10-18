//! Server cookie persistence delegates to the shared web policy.

use crate::ecmascript::RuntimeCode;
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

pub(super) fn generate_web_server_cookie_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/server-cookie.d.ts",
        None,
    ));
    module.push_import(type_import(EcmaScriptTarget::JavaScript));
    module.push_statement(EcmaStatement::type_declaration("export declare function persistLocaleCookie<Locale extends string>(\n  web: LinguiniWebLocale<Locale>,\n  target: unknown,\n  locale: Locale,\n  input?: Record<string, unknown>,\n): void;\n", None));
    module.render_code()
}

/// Compile server cookie policy delegation as TypeScript.
pub fn compile_typescript_web_server_cookie_module() -> RenderedEcmaModule {
    cookie_module(EcmaScriptTarget::TypeScript).render(&[])
}
/// Compile server cookie policy delegation as checked JavaScript.
pub fn compile_javascript_web_server_cookie_module() -> RenderedEcmaModule {
    cookie_module(EcmaScriptTarget::JavaScript).render(&[])
}

fn type_import(target: EcmaScriptTarget) -> EcmaImport {
    EcmaImport {
        specifier: super::names::import_path_for_target("../web", target),
        bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
            "LinguiniWebLocale",
            "LinguiniWebLocale",
        )]),
    }
}

fn cookie_module(target: EcmaScriptTarget) -> EcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        target,
        if target.is_typescript() {
            "web/server-cookie.ts"
        } else {
            "web/server-cookie.js"
        },
        None,
    ));
    module.push_import(type_import(EcmaScriptTarget::TypeScript));
    let mut code = RuntimeCode::default();
    code.typed("export function persistLocaleCookie<Locale extends string>(\n  web: LinguiniWebLocale<Locale>,\n  target: unknown,\n  locale: Locale,\n  input: Record<string, unknown> = {},\n) {\n", "/** @template {string} Locale @param {import(\"../web.js\").LinguiniWebLocale<Locale>} web @param {unknown} target @param {Locale} locale @param {Record<string, unknown>} [input] @returns {void} */\nexport function persistLocaleCookie(web, target, locale, input = {}) {\n");
    code.shared("  web.setLocaleCookie(target, locale, input);\n}\n");
    module.push_statement(EcmaStatement::generated(code.render(target)));
    module
}
