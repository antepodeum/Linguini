//! SvelteKit-owned bindings consumed by reusable Svelte generation.

use linguini_codegen_ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport, EcmaStatement,
    RenderedEcmaModule,
};
use linguini_codegen_svelte::{SvelteHost, SvelteHostImport};

pub(crate) fn bindings(navigation: bool) -> SvelteHost {
    SvelteHost {
        browser: Some(SvelteHostImport::new("$app/environment", "browser")),
        base_path: Some(SvelteHostImport::new("$app/paths", "base")),
        request_locale: Some(SvelteHostImport::new(
            "./sveltekit-locale.js",
            "getRequestLocale",
        )),
        navigate: navigation
            .then(|| SvelteHostImport::new("./sveltekit-navigation.js", "navigate")),
        navigation_state: Some(SvelteHostImport::new(
            "./sveltekit-navigation.js",
            "NavigationState",
        )),
    }
}

pub(crate) fn compile_locale_module() -> RenderedEcmaModule {
    compile_module("locale", "$app/state", "page")
}

pub(crate) fn compile_navigation_module() -> RenderedEcmaModule {
    compile_module("navigation", "$app/navigation", "goto")
}

fn compile_module(kind: &str, source: &str, binding: &str) -> RenderedEcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::JavaScript,
        format!("sveltekit-{kind}.js"),
        None,
    ));
    module.push_import(EcmaImport::named(
        source,
        vec![EcmaNamedImport::new(binding, binding)],
    ));
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        &format!("sveltekit/{kind}"),
        minijinja::context! {},
    )));
    module.render(&[])
}

pub(crate) fn declaration(kind: &str) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        format!("sveltekit-{kind}.d.ts"),
        None,
    ));
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render(
            &format!("sveltekit/{kind}_declarations"),
            minijinja::context! {},
        ),
        None,
    ));
    module.render_code()
}
