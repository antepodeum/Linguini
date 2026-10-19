//! Same-origin locale switching with selected server cookie persistence.

use super::names::escape_string;
use super::EcmaWebOptions;
use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_switch_route_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/switch-route.d.ts", None),
        "web/switch-route_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the configured locale switch route, or omit the absent capability.
pub fn compile_web_switch_route_module(options: &EcmaWebOptions) -> Option<RenderedEcmaModule> {
    let route = options.switch_route.as_ref()?;
    Some(
        crate::templates::module(
            EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/switch-route.js", None),
            "web/switch-route",
            minijinja::context! {
                route_path => escape_string(&route.path),
                return_query => escape_string(&route.return_query),
                status => route.status,
                has_cookie => options.features().has_cookie,
            },
        )
        .render(&[]),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{web_server_cookie, EcmaLocaleSource, EcmaWebSwitchRoute};
    use super::*;
    use std::path::Path;

    #[test]
    fn server_leaves_keep_cookie_delegation_and_safe_switch_route_parity() {
        assert!(compile_web_switch_route_module(&Default::default()).is_none());
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-server");
        let cookie_js = web_server_cookie::compile_web_server_cookie_module();
        assert_eq!(
            web_server_cookie::generate_web_server_cookie_declaration(),
            super::super::test_templates::declaration_with_runtime_imports(
                std::fs::read_to_string(root.join("legacy/web.server-cookie.runtime.d.ts"))
                    .unwrap()
            )
        );
        assert_eq!(
            generate_web_switch_route_declaration(),
            super::super::test_templates::declaration_with_runtime_imports(
                std::fs::read_to_string(root.join("legacy/web.switch-route.runtime.d.ts")).unwrap()
            )
        );
        let mut snapshots = vec![
            (root.join("web/server-cookie.js"), cookie_js.code),
            (root.join("web/server-cookie.js.map"), cookie_js.source_map),
            (
                root.join("typescript/web/server-cookie.d.ts"),
                web_server_cookie::generate_web_server_cookie_declaration(),
            ),
        ];
        for (name, has_cookie, path, return_query, status) in [
            ("cookie", true, "/locale/{locale}/set", "next", 303),
            ("path-only", false, "/locale/{locale}", "return", 307),
        ] {
            let options = EcmaWebOptions {
                sources: if has_cookie {
                    vec![EcmaLocaleSource::Path, EcmaLocaleSource::Cookie]
                } else {
                    vec![EcmaLocaleSource::Path]
                },
                switch_route: Some(EcmaWebSwitchRoute {
                    path: path.to_owned(),
                    return_query: return_query.to_owned(),
                    status,
                }),
                ..Default::default()
            };
            let js = compile_web_switch_route_module(&options).unwrap();
            assert_eq!(js.code.contains("persistLocaleCookie"), has_cookie);
            assert!(!js.code.contains("handleLocaleSwitchRoute<"));
            snapshots.extend([
                (root.join(format!("{name}/web/switch-route.js")), js.code),
                (
                    root.join(format!("{name}/web/switch-route.js.map")),
                    js.source_map,
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
