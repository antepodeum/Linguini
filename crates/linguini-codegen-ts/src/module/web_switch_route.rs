//! Same-origin locale switching with selected server cookie persistence.

use super::names::escape_string;
use super::runtime_code::RuntimeCode;
use super::TypeScriptWebOptions;
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

pub(super) fn generate_typescript_web_switch_route_module(
    options: &TypeScriptWebOptions,
) -> Option<String> {
    Some(switch_module(options, EcmaScriptTarget::TypeScript)?.render_code())
}

pub(super) fn generate_web_switch_route_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/switch-route.d.ts",
        None,
    ));
    module.push_import(type_import());
    module.push_statement(EcmaStatement::type_declaration("export declare function handleLocaleSwitchRoute<Locale extends string>(\n  web: LinguiniWebLocale<Locale>,\n  event: { url: URL; request: Request },\n): Response | undefined;\n", None));
    module.render_code()
}

/// Compile the configured locale switch route as TypeScript, or omit the absent capability.
pub fn compile_typescript_web_switch_route_module(
    options: &TypeScriptWebOptions,
) -> Option<RenderedEcmaModule> {
    Some(switch_module(options, EcmaScriptTarget::TypeScript)?.render(&[]))
}
/// Compile the configured locale switch route as checked JavaScript, or omit the absent capability.
pub fn compile_javascript_web_switch_route_module(
    options: &TypeScriptWebOptions,
) -> Option<RenderedEcmaModule> {
    Some(switch_module(options, EcmaScriptTarget::JavaScript)?.render(&[]))
}

fn type_import() -> EcmaImport {
    EcmaImport {
        specifier: "../web".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
            "LinguiniWebLocale",
            "LinguiniWebLocale",
        )]),
    }
}

fn switch_module(options: &TypeScriptWebOptions, target: EcmaScriptTarget) -> Option<EcmaModule> {
    let route = options.switch_route.as_ref()?;
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        target,
        if target.is_typescript() {
            "web/switch-route.ts"
        } else {
            "web/switch-route.js"
        },
        None,
    ));
    module.push_import(type_import());
    if options.features().has_cookie {
        module.push_import(EcmaImport::named(
            "./server-cookie.js",
            vec![EcmaNamedImport::new(
                "persistLocaleCookie",
                "persistLocaleCookie",
            )],
        ));
    }
    let mut code = RuntimeCode::default();
    code.generated(format!(
        "const routePath = \"{}\";\nconst returnQuery = \"{}\";\nconst redirectStatus = {};\n\n",
        escape_string(&route.path),
        escape_string(&route.return_query),
        route.status
    ));
    code.typed("export function handleLocaleSwitchRoute<Locale extends string>(\n  web: LinguiniWebLocale<Locale>,\n  event: { url: URL; request: Request },\n): Response | undefined {\n", "/** @template {string} Locale @param {import(\"../web.js\").LinguiniWebLocale<Locale>} web @param {{url: URL; request: Request}} event @returns {Response | undefined} */\nexport function handleLocaleSwitchRoute(web, event) {\n");
    code.shared(
        r#"  const locale = matchSwitchLocale(web, event.url.pathname);
  if (!locale) return undefined;
  const input = {
    url: event.url,
    currentUrl: event.url,
    origin: event.url.origin,
    headers: event.request.headers,
  };
  const returnTarget = (safeReturnTarget(event) ?? web.options.environment.base) || "/";
  const location = web.options.locale.switch.writesPath
    ? web.localizeHref(returnTarget, locale, input)
    : returnTarget;
  const response = new Response(null, {
    status: redirectStatus,
    headers: { location },
  });
"#,
    );
    if options.features().has_cookie {
        code.shared("  if (web.options.locale.switch.writesCookie) {\n    persistLocaleCookie(web, response, locale, { origin: event.url.origin });\n  }\n");
    } else {
        code.shared("\n");
    }
    code.shared("  return response;\n}\n\n");
    code.typed("function matchSwitchLocale<Locale extends string>(\n  web: LinguiniWebLocale<Locale>,\n  pathname: string,\n) {\n", "/** @template {string} Locale @param {import(\"../web.js\").LinguiniWebLocale<Locale>} web @param {string} pathname @returns {Locale | undefined} */\nfunction matchSwitchLocale(web, pathname) {\n");
    code.shared(
        r#"  const effectiveRoutePath = `${web.options.environment.base}${routePath}` || routePath;
  const marker = "{locale}";
  const markerIndex = effectiveRoutePath.indexOf(marker);
  const prefix = effectiveRoutePath.slice(0, markerIndex);
  const suffix = effectiveRoutePath.slice(markerIndex + marker.length);
  if (!pathname.startsWith(prefix) || !pathname.endsWith(suffix)) return undefined;
  const encoded = pathname.slice(
    prefix.length,
    suffix.length > 0 ? pathname.length - suffix.length : pathname.length,
  );
  if (!encoded || encoded.includes("/")) return undefined;
  try {
    return web.matchLocale(decodeURIComponent(encoded));
  } catch {
    return undefined;
  }
}

"#,
    );
    code.typed("function safeReturnTarget(event: { url: URL; request: Request }) {\n", "/** @param {{url: URL; request: Request}} event @returns {string | undefined} */\nfunction safeReturnTarget(event) {\n");
    code.shared(
        r#"  const queryTarget = event.url.searchParams.get(returnQuery);
  const query = queryTarget ? sameOriginPath(queryTarget, event.url.origin) : undefined;
  if (query) return query;
  const referer = event.request.headers.get("referer");
  return referer ? sameOriginPath(referer, event.url.origin) : undefined;
}

"#,
    );
    code.typed("function sameOriginPath(value: string, origin: string) {\n", "/** @param {string} value @param {string} origin @returns {string | undefined} */\nfunction sameOriginPath(value, origin) {\n");
    code.shared(
        r#"  if (!value) return undefined;
  let decoded = value;
  for (let depth = 0; depth < 3; depth += 1) {
    if (decoded.includes("\\") || decoded.startsWith("//")) return undefined;
    try {
      const next = decodeURIComponent(decoded);
      if (next === decoded) break;
      decoded = next;
    } catch {
      return undefined;
    }
  }
  try {
    const parsed = new URL(value, origin);
    if (parsed.origin !== origin) return undefined;
    return `${parsed.pathname}${parsed.search}${parsed.hash}`;
  } catch {
    return undefined;
  }
}
"#,
    );
    module.push_statement(EcmaStatement::generated(code.render(target)));
    Some(module)
}

#[cfg(test)]
mod tests {
    use super::super::{web_server_cookie, TypeScriptLocaleSource, TypeScriptWebSwitchRoute};
    use super::*;
    use std::path::Path;

    #[test]
    fn server_leaves_keep_cookie_delegation_and_safe_switch_route_parity() {
        assert!(compile_javascript_web_switch_route_module(&Default::default()).is_none());
        assert!(compile_typescript_web_switch_route_module(&Default::default()).is_none());
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-server");
        let cookie_ts = web_server_cookie::compile_typescript_web_server_cookie_module();
        let cookie_js = web_server_cookie::compile_javascript_web_server_cookie_module();
        assert_eq!(
            cookie_ts
                .code
                .strip_suffix("//# sourceMappingURL=server-cookie.ts.map\n")
                .unwrap(),
            std::fs::read_to_string(root.join("legacy/web.server-cookie.runtime.ts")).unwrap()
        );
        assert_eq!(
            web_server_cookie::generate_web_server_cookie_declaration(),
            std::fs::read_to_string(root.join("legacy/web.server-cookie.runtime.d.ts")).unwrap()
        );
        assert_eq!(
            generate_web_switch_route_declaration(),
            std::fs::read_to_string(root.join("legacy/web.switch-route.runtime.d.ts")).unwrap()
        );
        let mut snapshots = vec![
            (root.join("web/server-cookie.js"), cookie_js.code),
            (root.join("web/server-cookie.js.map"), cookie_js.source_map),
            (root.join("typescript/web/server-cookie.ts"), cookie_ts.code),
            (
                root.join("typescript/web/server-cookie.d.ts"),
                web_server_cookie::generate_web_server_cookie_declaration(),
            ),
        ];
        for (name, has_cookie, path, return_query, status) in [
            ("cookie", true, "/locale/{locale}/set", "next", 303),
            ("path-only", false, "/locale/{locale}", "return", 307),
        ] {
            let options = TypeScriptWebOptions {
                sources: if has_cookie {
                    vec![TypeScriptLocaleSource::Path, TypeScriptLocaleSource::Cookie]
                } else {
                    vec![TypeScriptLocaleSource::Path]
                },
                switch_route: Some(TypeScriptWebSwitchRoute {
                    path: path.to_owned(),
                    return_query: return_query.to_owned(),
                    status,
                }),
                ..Default::default()
            };
            let ts = compile_typescript_web_switch_route_module(&options).unwrap();
            let js = compile_javascript_web_switch_route_module(&options).unwrap();
            assert_eq!(js.code.contains("persistLocaleCookie"), has_cookie);
            assert!(!js.code.contains("handleLocaleSwitchRoute<"));
            let legacy = super::super::templates::render_template(&std::fs::read_to_string(root.join("legacy/web.switch-route.runtime.ts")).unwrap(), &[
                ("SERVER_COOKIE_IMPORT", if has_cookie { "import { persistLocaleCookie } from \"./server-cookie.js\";" } else { "" }.to_owned()),
                ("SWITCH_ROUTE_PATH", path.to_owned()), ("SWITCH_ROUTE_RETURN_QUERY", return_query.to_owned()), ("SWITCH_ROUTE_STATUS", status.to_string()),
                ("PERSIST_SWITCH_COOKIE", if has_cookie { "  if (web.options.locale.switch.writesCookie) {\n    persistLocaleCookie(web, response, locale, { origin: event.url.origin });\n  }" } else { "" }.to_owned()),
            ]);
            let body = |code: &str| {
                code.lines()
                    .filter(|line| {
                        !line.trim().is_empty() && !line.starts_with("//# sourceMappingURL=")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            assert_eq!(body(&ts.code), body(&legacy));
            snapshots.extend([
                (root.join(format!("{name}/web/switch-route.js")), js.code),
                (
                    root.join(format!("{name}/web/switch-route.js.map")),
                    js.source_map,
                ),
                (
                    root.join(format!("{name}/typescript/web/switch-route.ts")),
                    ts.code,
                ),
                (
                    root.join(format!("{name}/typescript/web/switch-route.d.ts")),
                    generate_web_switch_route_declaration(),
                ),
            ]);
        }
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
}
