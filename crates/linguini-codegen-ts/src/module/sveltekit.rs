//! Shared SvelteKit server adapters with closed cookie and switch-route capabilities.

use super::runtime_code::RuntimeCode;
use super::TypeScriptWebOptions;
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

pub(super) fn generate_typescript_sveltekit_module(
    options: &TypeScriptWebOptions,
    messages: bool,
) -> String {
    adapter_module(options, messages, EcmaScriptTarget::TypeScript).render_code()
}
/// Compile the message-enabled SvelteKit server adapter as TypeScript.
pub fn compile_typescript_sveltekit_module(options: &TypeScriptWebOptions) -> RenderedEcmaModule {
    adapter_module(options, true, EcmaScriptTarget::TypeScript).render(&[])
}
/// Compile the message-enabled SvelteKit server adapter as checked JavaScript.
pub fn compile_javascript_sveltekit_module(options: &TypeScriptWebOptions) -> RenderedEcmaModule {
    adapter_module(options, true, EcmaScriptTarget::JavaScript).render(&[])
}
/// Compile the lightweight SvelteKit server adapter as TypeScript.
pub fn compile_typescript_sveltekit_control_module(
    options: &TypeScriptWebOptions,
) -> RenderedEcmaModule {
    adapter_module(options, false, EcmaScriptTarget::TypeScript).render(&[])
}
/// Compile the lightweight SvelteKit server adapter as checked JavaScript.
pub fn compile_javascript_sveltekit_control_module(
    options: &TypeScriptWebOptions,
) -> RenderedEcmaModule {
    adapter_module(options, false, EcmaScriptTarget::JavaScript).render(&[])
}

fn adapter_module(
    options: &TypeScriptWebOptions,
    messages: bool,
    target: EcmaScriptTarget,
) -> EcmaModule {
    let name = if messages {
        "sveltekit"
    } else {
        "sveltekit-control"
    };
    let extension = if target.is_typescript() { "ts" } else { "js" };
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
        specifier: if target.is_typescript() {
            runtime_path.to_owned()
        } else {
            format!("{runtime_path}.js")
        },
        bindings: EcmaImportBindings::Namespace(
            if messages { "runtime" } else { "locale" }.to_owned(),
        ),
    };
    let web_import = EcmaImport::named(
        if target.is_typescript() {
            "./web"
        } else {
            "./web.js"
        },
        vec![EcmaNamedImport::new(factory, factory)],
    );
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
    let mut code = RuntimeCode::default();
    code.generated(format!(
        "const options = {};\n\n",
        super::web_options::web_options_literal(options, target)
    ));
    adapter_body(
        &mut code,
        messages,
        has_cookie,
        options.switch_route.is_some(),
    );
    module.push_statement(EcmaStatement::generated(code.render(target)));
    module
}

fn adapter_body(code: &mut RuntimeCode, messages: bool, has_cookie: bool, switch_route: bool) {
    code.typed(
        "export const linguiniHandle: Handle =",
        "/** @type {import(\"@sveltejs/kit\").Handle} */\nexport const linguiniHandle =",
    );
    code.generated(format!(
        " createHandle({}, options);\n",
        if messages { "runtime" } else { "locale" }
    ));
    code.typed(
        "export const linguiniReroute: Reroute =",
        "/** @type {import(\"@sveltejs/kit\").Reroute} */\nexport const linguiniReroute =",
    );
    code.generated(format!(
        " createReroute({}, options);\n",
        if messages { "runtime" } else { "locale" }
    ));
    code.typed(
        "export const linguiniLoad: ServerLoad =",
        "/** @type {import(\"@sveltejs/kit\").ServerLoad} */\nexport const linguiniLoad =",
    );
    code.shared(" createLoad();\n");
    code.typed(
        "export const handle: Handle =",
        "/** @type {import(\"@sveltejs/kit\").Handle} */\nexport const handle =",
    );
    code.shared(" linguiniHandle;\n");
    code.typed(
        "export const reroute: Reroute =",
        "/** @type {import(\"@sveltejs/kit\").Reroute} */\nexport const reroute =",
    );
    code.shared(" linguiniReroute;\n");
    code.typed(
        "export const load: ServerLoad =",
        "/** @type {import(\"@sveltejs/kit\").ServerLoad} */\nexport const load =",
    );
    code.shared(" linguiniLoad;\n\n");
    if messages {
        code.typed("function createHandle(runtime: typeof import(\"./index\"), options: Record<string, unknown> = {}) {", "/** @param {typeof import(\"./index.js\")} runtime @param {Record<string, unknown>} [options] @returns {import(\"@sveltejs/kit\").Handle} */\nfunction createHandle(runtime, options = {}) {");
    } else {
        code.typed("function createHandle(runtime: typeof import(\"./locale\"), options: Record<string, unknown> = {}) {", "/** @param {typeof import(\"./locale.js\")} runtime @param {Record<string, unknown>} [options] @returns {import(\"@sveltejs/kit\").Handle} */\nfunction createHandle(runtime, options = {}) {");
    }
    code.shared("\n  const web = ");
    web_factory(code, messages);
    code.shared(";\n");
    if has_cookie {
        code.shared("  const persistCookie = options.persistCookie !== false && web.options.locale.switch.writesCookie;");
    }
    code.shared("\n\n");
    code.typed("  return async function linguiniHandle({ event, resolve }: Parameters<Handle>[0]) {", "  /** @param {Parameters<import(\"@sveltejs/kit\").Handle>[0]} input */\n  return async function linguiniHandle({ event, resolve }) {");
    code.shared("\n    if (web.shouldExclude(event.url)) {\n      return resolve(event);\n    }\n");
    if switch_route {
        code.shared("    const switchResponse = handleLocaleSwitchRoute(web, event);\n    if (switchResponse) return switchResponse;");
    }
    code.shared("\n\n");
    request_context(code, messages, has_cookie);
    code.shared("\n\n    event.locals.linguini = context;\n\n    const redirectLocation = web.getCanonicalRedirect(event.url, context.locale);\n    if (redirectLocation) {\n      const response = new Response(null, {\n        status: 307,\n        headers: { location: redirectLocation },\n      });\n");
    if has_cookie {
        code.shared("      if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });");
    }
    code.shared(
        "\n      return response;\n    }\n\n    const response = await resolve(event, {\n      ",
    );
    code.typed(
        "transformPageChunk: ({ html }: { html: string }) =>",
        "transformPageChunk: ({ html }) =>",
    );
    code.shared("\n        html\n          .replaceAll(\"%linguini.lang%\", context.lang)\n          .replaceAll(\"%linguini.dir%\", context.direction)\n          .replaceAll(\"%linguini.locale%\", context.locale),\n    });\n\n");
    if has_cookie {
        code.shared("    if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });");
    }
    code.shared("\n    return response;\n  };\n}\n\n");
    if messages {
        code.typed("function createReroute(runtime: typeof import(\"./index\"), options: Record<string, unknown> = {}) {", "/** @param {typeof import(\"./index.js\")} runtime @param {Record<string, unknown>} [options] @returns {import(\"@sveltejs/kit\").Reroute} */\nfunction createReroute(runtime, options = {}) {");
    } else {
        code.typed("function createReroute(runtime: typeof import(\"./locale\"), options: Record<string, unknown> = {}) {", "/** @param {typeof import(\"./locale.js\")} runtime @param {Record<string, unknown>} [options] @returns {import(\"@sveltejs/kit\").Reroute} */\nfunction createReroute(runtime, options = {}) {");
    }
    code.shared("\n  const web = ");
    web_factory(code, messages);
    code.shared(";\n");
    code.typed(
        "  return function linguiniReroute({ url }: { url: URL }) {",
        "  /** @param {{url: URL}} input */\n  return function linguiniReroute({ url }) {",
    );
    code.shared("\n    if (web.shouldExclude(url)) return undefined;\n    const delocalized = web.delocalizePathname(url.pathname);\n    return delocalized === url.pathname ? undefined : delocalized;\n  };\n}\n");
    if !messages {
        super::sveltekit_context::append(code);
    }
    code.shared("\nfunction createLoad() {\n");
    code.typed("  return function linguiniLayoutLoad({ locals }: { locals: Record<string, any> }) {", "  /** @param {{locals: App.Locals}} input */\n  return function linguiniLayoutLoad({ locals }) {");
    code.shared(
        "\n    return {\n      linguini: serializeContext(locals.linguini),\n    };\n  };\n}\n\n",
    );
    code.typed("function serializeContext(context: any) {", "/** @param {import(\"./sveltekit-control.js\").LinguiniSvelteKitLocaleContext | undefined} context @returns {import(\"./sveltekit-control.js\").LinguiniSerializedContext | undefined} */\nfunction serializeContext(context) {");
    code.shared("\n  if (!context) return undefined;\n  return {\n    locale: context.locale,\n    baseLocale: context.baseLocale,\n    locales: context.locales,\n    direction: context.direction,\n    lang: context.lang,\n    htmlAttrs: context.htmlAttrs,\n  };\n}\n");
}

fn web_factory(code: &mut RuntimeCode, messages: bool) {
    let factory = if messages {
        "createWebI18n"
    } else {
        "createWebLocaleI18n"
    };
    code.generated(factory);
    code.typed("(runtime, options.web as Record<string, unknown> | undefined ?? options, { base })", "(runtime, /** @type {Record<string, unknown> | undefined} */ (options.web) ?? options, { base })");
}

fn request_context(code: &mut RuntimeCode, messages: bool, has_cookie: bool) {
    if messages {
        code.shared("    const context = await web.resolveRequest(event.request, {\n      url: event.url,\n      currentUrl: event.url,\n      origin: event.url.origin,\n      headers: event.request.headers,\n");
        if has_cookie {
            code.shared("      cookie: event.request.headers.get(\"cookie\") ?? undefined,");
        }
        code.shared("\n    });\n");
    } else {
        code.shared("    const contextInput = {\n      url: event.url,\n      currentUrl: event.url,\n      origin: event.url.origin,\n      headers: event.request.headers,\n");
        if has_cookie {
            code.shared("      cookie: event.request.headers.get(\"cookie\") ?? undefined,");
        }
        code.shared("\n    };\n    const resolvedLocale = await web.resolveLocale(contextInput);\n    const context = createRequestContext(web, resolvedLocale, contextInput);\n");
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        templates::render_template, TypeScriptLocaleSource, TypeScriptWebSwitchRoute,
    };
    use super::*;
    use std::path::Path;

    #[test]
    fn server_adapters_share_policy_request_contexts_and_streaming() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-sveltekit");
        for (name, options) in [
            ("default", TypeScriptWebOptions::default()),
            (
                "path",
                TypeScriptWebOptions {
                    sources: vec![TypeScriptLocaleSource::Path],
                    ..Default::default()
                },
            ),
            (
                "empty",
                TypeScriptWebOptions {
                    sources: vec![],
                    ..Default::default()
                },
            ),
            (
                "all",
                TypeScriptWebOptions {
                    sources: vec![
                        TypeScriptLocaleSource::Path,
                        TypeScriptLocaleSource::Cookie,
                        TypeScriptLocaleSource::AcceptLanguage,
                    ],
                    exclude: vec!["/api/**".to_owned()],
                    switch_route: Some(TypeScriptWebSwitchRoute {
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
                let ts =
                    adapter_module(&options, messages, EcmaScriptTarget::TypeScript).render(&[]);
                let js =
                    adapter_module(&options, messages, EcmaScriptTarget::JavaScript).render(&[]);
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
                let legacy = render_template(&std::fs::read_to_string(root.join(format!("legacy/{module_name}.runtime.ts"))).unwrap(), &[
                    ("OPTIONS", super::super::web_options::web_options_literal(&options, EcmaScriptTarget::TypeScript)),
                    ("SERVER_COOKIE_IMPORT", if cookie { "import { persistLocaleCookie } from \"./web/server-cookie.js\";" } else { "" }.to_owned()),
                    ("SWITCH_ROUTE_IMPORT", if options.switch_route.is_some() { "import { handleLocaleSwitchRoute } from \"./web/switch-route.js\";" } else { "" }.to_owned()),
                    ("PERSIST_COOKIE_DECLARATION", if cookie { "  const persistCookie = options.persistCookie !== false && web.options.locale.switch.writesCookie;" } else { "" }.to_owned()),
                    ("COOKIE_INPUT", if cookie { "      cookie: event.request.headers.get(\"cookie\") ?? undefined," } else { "" }.to_owned()),
                    ("PERSIST_REDIRECT_COOKIE", if cookie { "      if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });" } else { "" }.to_owned()),
                    ("PERSIST_RESPONSE_COOKIE", if cookie { "    if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });" } else { "" }.to_owned()),
                    ("SWITCH_ROUTE_BRANCH", if options.switch_route.is_some() { "    const switchResponse = handleLocaleSwitchRoute(web, event);\n    if (switchResponse) return switchResponse;" } else { "" }.to_owned()),
                ]);
                let lines = |code: &str| {
                    code.lines()
                        .filter(|line| {
                            !line.trim().is_empty() && !line.starts_with("//# sourceMappingURL=")
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                assert_eq!(lines(&ts.code), lines(&legacy));
                let snapshots = [
                    (root.join(format!("{name}/{module_name}.js")), js.code),
                    (
                        root.join(format!("{name}/{module_name}.js.map")),
                        js.source_map,
                    ),
                    (
                        root.join(format!("{name}/typescript/{module_name}.ts")),
                        ts.code,
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
                std::fs::read_to_string(root.join("legacy/sveltekit-control.runtime.d.ts"))
                    .unwrap()
            );
            assert_eq!(
                super::super::sveltekit_types::generate_ambient_declaration(),
                std::fs::read_to_string(root.join("legacy/sveltekit.runtime.d.ts")).unwrap()
            );
            let mut dependencies = vec![
                (
                    "web.js".to_owned(),
                    super::super::web_facade::compile_javascript_web_module(&options).code,
                ),
                (
                    "typescript/web.ts".to_owned(),
                    super::super::web_facade::compile_typescript_web_module(&options).code,
                ),
                (
                    "typescript/web.d.ts".to_owned(),
                    super::super::web_types::generate_web_declaration(),
                ),
                (
                    "sveltekit-control.d.ts".to_owned(),
                    super::super::sveltekit_types::generate_control_declaration(),
                ),
            ];
            for source in options.sources.iter() {
                let (name, ts, js) = match source {
                    TypeScriptLocaleSource::Path => ("path", super::super::web_path::compile_typescript_web_path_module(), super::super::web_path::compile_javascript_web_path_module()),
                    TypeScriptLocaleSource::Cookie => ("cookie", super::super::web_cookie::compile_typescript_web_cookie_module(), super::super::web_cookie::compile_javascript_web_cookie_module()),
                    TypeScriptLocaleSource::LocalStorage => unreachable!(),
                    TypeScriptLocaleSource::AcceptLanguage => ("accept-language", super::super::web_accept_language::compile_typescript_web_accept_language_module(), super::super::web_accept_language::compile_javascript_web_accept_language_module()),
                };
                dependencies.push((format!("web/{name}.js"), js.code));
                dependencies.push((format!("typescript/web/{name}.ts"), ts.code));
            }
            if !options.exclude.is_empty() {
                dependencies.extend([
                    (
                        "web/routes.js".to_owned(),
                        super::super::web_routes::compile_javascript_web_routes_module().code,
                    ),
                    (
                        "typescript/web/routes.ts".to_owned(),
                        super::super::web_routes::compile_typescript_web_routes_module().code,
                    ),
                ]);
            }
            if cookie {
                dependencies.extend([("web/server-cookie.js".to_owned(), super::super::web_server_cookie::compile_javascript_web_server_cookie_module().code), ("typescript/web/server-cookie.ts".to_owned(), super::super::web_server_cookie::compile_typescript_web_server_cookie_module().code)]);
            }
            if let Some(js) =
                super::super::web_switch_route::compile_javascript_web_switch_route_module(&options)
            {
                dependencies.push(("web/switch-route.js".to_owned(), js.code));
                dependencies.push((
                    "typescript/web/switch-route.ts".to_owned(),
                    super::super::web_switch_route::compile_typescript_web_switch_route_module(
                        &options,
                    )
                    .unwrap()
                    .code,
                ));
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
