//! Shared SvelteKit server adapters with closed cookie and switch-route capabilities.

use super::EcmaWebOptions;
use linguini_codegen_ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement, RenderedEcmaModule,
};

/// Compile the message-enabled SvelteKit server adapter as checked JavaScript.
pub fn compile_sveltekit_module(options: &EcmaWebOptions) -> RenderedEcmaModule {
    adapter_module(options, true, EcmaModuleKind::JavaScript).render(&[])
}

/// Compile the lightweight SvelteKit server adapter as checked JavaScript.
pub fn compile_sveltekit_control_module(options: &EcmaWebOptions) -> RenderedEcmaModule {
    adapter_module(options, false, EcmaModuleKind::JavaScript).render(&[])
}

fn adapter_module(options: &EcmaWebOptions, messages: bool, target: EcmaModuleKind) -> EcmaModule {
    let name = if messages {
        "sveltekit"
    } else {
        "sveltekit-control"
    };
    let extension = "js";
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        target,
        format!("{name}.{extension}"),
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "@sveltejs/kit".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(
            ["Handle", "Reroute", "ServerLoad"]
                .map(|name| EcmaNamedImport::new(name, name))
                .to_vec(),
        ),
    });
    module.push_import(EcmaImport::named(
        "$app/paths",
        vec![EcmaNamedImport::new("base", "base")],
    ));
    let factory = if messages {
        "createWebI18n"
    } else {
        "createWebLocaleI18n"
    };
    let runtime_path = if messages { "./index" } else { "./locale" };
    if !messages {
        module.push_import(EcmaImport {
            specifier: "./locale".to_owned(),
            bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new("Locale", "Locale")]),
        });
    }
    let runtime_import = EcmaImport {
        specifier: { format!("{runtime_path}.js") },
        bindings: EcmaImportBindings::Namespace(
            if messages { "runtime" } else { "locale" }.to_owned(),
        ),
    };
    let web_import = EcmaImport::named("./web.js", vec![EcmaNamedImport::new(factory, factory)]);
    if messages {
        module.push_import(web_import);
        module.push_import(runtime_import);
    } else {
        module.push_import(runtime_import);
        module.push_import(web_import);
    }
    let has_cookie = options.features().has_cookie;
    if has_cookie {
        module.push_import(EcmaImport::named(
            "./web/server-cookie.js",
            vec![EcmaNamedImport::new(
                "persistLocaleCookie",
                "persistLocaleCookie",
            )],
        ));
    }
    if options.switch_route.is_some() {
        module.push_import(EcmaImport::named(
            "./web/switch-route.js",
            vec![EcmaNamedImport::new(
                "handleLocaleSwitchRoute",
                "handleLocaleSwitchRoute",
            )],
        ));
    }
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "sveltekit/adapter",
        minijinja::context! { options => linguini_codegen_ecmascript_web::render_web_options(options), messages, has_cookie, switch_route => options.switch_route.is_some() },
    )));
    module
}

#[cfg(test)]
mod tests {
    use super::super::{EcmaLocaleSource, EcmaWebSwitchRoute};
    use super::*;
    use std::path::Path;

    #[test]
    fn server_adapters_share_policy_request_contexts_and_streaming() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-sveltekit");
        for (name, options) in [
            ("default", EcmaWebOptions::default()),
            (
                "path",
                EcmaWebOptions {
                    sources: vec![EcmaLocaleSource::Path],
                    ..Default::default()
                },
            ),
            (
                "empty",
                EcmaWebOptions {
                    sources: vec![],
                    ..Default::default()
                },
            ),
            (
                "all",
                EcmaWebOptions {
                    sources: vec![
                        EcmaLocaleSource::Path,
                        EcmaLocaleSource::Cookie,
                        EcmaLocaleSource::AcceptLanguage,
                    ],
                    exclude: vec!["/api/**".to_owned()],
                    switch_route: Some(EcmaWebSwitchRoute {
                        path: "/locale/{locale}".to_owned(),
                        return_query: "return".to_owned(),
                        status: 303,
                    }),
                    ..Default::default()
                },
            ),
        ] {
            let cookie = options.features().has_cookie;
            for messages in [false, true] {
                let module_name = if messages {
                    "sveltekit"
                } else {
                    "sveltekit-control"
                };
                let js = adapter_module(&options, messages, EcmaModuleKind::JavaScript).render(&[]);
                assert_eq!(js.code.contains("persistLocaleCookie"), cookie);
                assert_eq!(
                    js.code.contains("event.request.headers.get(\"cookie\")"),
                    cookie
                );
                assert_eq!(
                    js.code.contains("handleLocaleSwitchRoute"),
                    options.switch_route.is_some()
                );
                assert_eq!(js.code.contains("resolveRequest"), messages);
                assert!(!js.code.contains("prepareLinguini"));
                assert!(!js.code.contains("bufferedHtml"));
                for required in [
                    "createHandle",
                    "export const linguiniHandle",
                    "export const linguiniReroute",
                    "export const linguiniLoad",
                    "status: 307",
                    "locals.linguini = context",
                ] {
                    assert!(js.code.contains(required));
                }
                for removed in [
                    "redirectStatus",
                    "locals.locale",
                    "locals.direction",
                    "locals.l =",
                ] {
                    assert!(!js.code.contains(removed));
                }
                let declaration = super::super::sveltekit_types::generate_ambient_declaration();
                for required in [
                    "linguiniHandle: Handle",
                    "linguiniReroute: Reroute",
                    "linguiniLoad: ServerLoad",
                    "SerializedLinguiniContext",
                ] {
                    assert!(declaration.contains(required));
                }
                for removed in [
                    "\n      locale: Locale;",
                    "\n      direction: TextDirection;",
                    "\n      l: Linguini;",
                ] {
                    assert!(!declaration.contains(removed));
                }

                assert!(!js.code.contains("options.web as"));
                let snapshots = [
                    (root.join(format!("{name}/{module_name}.js")), js.code),
                    (
                        root.join(format!("{name}/{module_name}.js.map")),
                        js.source_map,
                    ),
                    (
                        root.join(format!("{name}/typescript/sveltekit-control.d.ts")),
                        super::super::sveltekit_types::generate_control_declaration(),
                    ),
                    (
                        root.join(format!("{name}/typescript/linguini-app.d.ts")),
                        super::super::sveltekit_types::generate_ambient_declaration(),
                    ),
                ];
                if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
                    for (path, code) in &snapshots {
                        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                        std::fs::write(path, code).unwrap();
                    }
                }
                for (path, code) in snapshots {
                    assert_eq!(code, std::fs::read_to_string(path).unwrap());
                }
            }
            assert_eq!(
                super::super::sveltekit_types::generate_control_declaration(),
                super::super::test_templates::declaration_with_runtime_imports(
                    std::fs::read_to_string(root.join("legacy/sveltekit-control.runtime.d.ts"))
                        .unwrap()
                )
            );
            assert_eq!(
                super::super::sveltekit_types::generate_ambient_declaration(),
                super::super::test_templates::declaration_with_runtime_imports(
                    std::fs::read_to_string(root.join("legacy/sveltekit.runtime.d.ts")).unwrap()
                )
            );
            let mut dependencies = vec![
                (
                    "web.js".to_owned(),
                    linguini_codegen_ecmascript_web::compile_web_module(&options).code,
                ),
                (
                    "typescript/web.d.ts".to_owned(),
                    linguini_codegen_ecmascript_web::generate_web_declaration(),
                ),
                (
                    "sveltekit-control.d.ts".to_owned(),
                    super::super::sveltekit_types::generate_control_declaration(),
                ),
            ];
            for source in options.sources.iter() {
                let (name, js) = match source {
                    EcmaLocaleSource::Path => (
                        "path",
                        linguini_codegen_ecmascript_web::compile_web_path_module(),
                    ),
                    EcmaLocaleSource::Cookie => (
                        "cookie",
                        linguini_codegen_ecmascript_web::compile_web_cookie_module(),
                    ),
                    EcmaLocaleSource::LocalStorage => unreachable!(),
                    EcmaLocaleSource::AcceptLanguage => (
                        "accept-language",
                        linguini_codegen_ecmascript_web::compile_web_accept_language_module(),
                    ),
                };
                dependencies.push((format!("web/{name}.js"), js.code));
            }
            if !options.exclude.is_empty() {
                dependencies.extend([(
                    "web/routes.js".to_owned(),
                    linguini_codegen_ecmascript_web::compile_web_routes_module().code,
                )]);
            }
            if cookie {
                dependencies.extend([(
                    "web/server-cookie.js".to_owned(),
                    linguini_codegen_ecmascript_web::compile_web_server_cookie_module().code,
                )]);
            }
            if let Some(js) =
                linguini_codegen_ecmascript_web::compile_web_switch_route_module(&options)
            {
                dependencies.push(("web/switch-route.js".to_owned(), js.code));
            }
            for (path, code) in dependencies {
                let path = root.join(format!("{name}/{path}"));
                if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    std::fs::write(&path, &code).unwrap();
                }
                assert_eq!(code, std::fs::read_to_string(path).unwrap());
            }
        }
    }
}
