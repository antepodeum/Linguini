//! Closed web facade: statically selected locale resolvers and shared URL/request policy.

use super::{EcmaLocaleSource, EcmaWebOptions};
use linguini_codegen_ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport, EcmaStatement,
    RenderedEcmaModule,
};

/// Compile the closed web facade as checked JavaScript.
pub fn compile_web_module(options: &EcmaWebOptions) -> RenderedEcmaModule {
    web_module(options).render(&[])
}

fn web_module(options: &EcmaWebOptions) -> EcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::JavaScript,
        "web.js",
        Some("web.d.ts".to_owned()),
    ));
    if !options.exclude.is_empty() {
        module.push_import(EcmaImport::named(
            "./web/routes.js",
            vec![EcmaNamedImport::new("matchesRoute", "matchesRoute")],
        ));
    }
    for source in &options.features().sources {
        let (name, path) = match source {
            EcmaLocaleSource::Path => ("resolvePathLocale", "path"),
            EcmaLocaleSource::Cookie => ("resolveCookieLocale", "cookie"),
            EcmaLocaleSource::LocalStorage => ("resolveLocalStorageLocale", "local-storage"),
            EcmaLocaleSource::AcceptLanguage => ("resolveAcceptLanguageLocale", "accept-language"),
        };
        module.push_import(EcmaImport::named(
            format!("./web/{path}.js"),
            vec![EcmaNamedImport::new(name, name)],
        ));
    }
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "web/constants",
        minijinja::context! {},
    )));
    super::types::append(&mut module);
    let sources: Vec<_> = options
        .features()
        .sources
        .iter()
        .map(|source| match source {
            EcmaLocaleSource::Path => "path",
            EcmaLocaleSource::Cookie => "cookie",
            EcmaLocaleSource::LocalStorage => "local-storage",
            EcmaLocaleSource::AcceptLanguage => "accept-language",
        })
        .collect();
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "web/facade",
        minijinja::context! { sources, route_exclusions => !options.exclude.is_empty() },
    )));
    module
}

#[cfg(test)]
mod tests {
    use super::super::{accept_language, cookie, local_storage, path, routes, types};
    use super::*;
    use std::path::Path;

    #[test]
    fn web_facades_share_closed_source_policy_and_public_api() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web");
        assert_eq!(
            types::generate_web_declaration(),
            std::fs::read_to_string(root.join("legacy/web.runtime.d.ts")).unwrap()
        );
        let cases = [
            ("default", EcmaWebOptions::default()),
            (
                "all",
                EcmaWebOptions {
                    sources: vec![
                        EcmaLocaleSource::Path,
                        EcmaLocaleSource::Cookie,
                        EcmaLocaleSource::LocalStorage,
                        EcmaLocaleSource::AcceptLanguage,
                    ],
                    exclude: vec!["/api/**".to_owned()],
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
                "path",
                EcmaWebOptions {
                    sources: vec![EcmaLocaleSource::Path],
                    ..Default::default()
                },
            ),
            (
                "cookie",
                EcmaWebOptions {
                    sources: vec![EcmaLocaleSource::Cookie],
                    ..Default::default()
                },
            ),
            (
                "storage",
                EcmaWebOptions {
                    sources: vec![EcmaLocaleSource::LocalStorage],
                    ..Default::default()
                },
            ),
            (
                "language",
                EcmaWebOptions {
                    sources: vec![EcmaLocaleSource::AcceptLanguage],
                    ..Default::default()
                },
            ),
        ];
        let mut cases = cases.to_vec();
        cases.push((
            "language-cookie",
            EcmaWebOptions {
                sources: vec![EcmaLocaleSource::AcceptLanguage, EcmaLocaleSource::Cookie],
                ..Default::default()
            },
        ));
        let mut snapshots = vec![];
        for (name, options) in cases {
            let js = compile_web_module(&options);
            assert!(!js.code.contains("resolveLocaleSource"));
            assert!(!js.code.contains("createWebI18n<"));
            for source in [
                EcmaLocaleSource::Path,
                EcmaLocaleSource::Cookie,
                EcmaLocaleSource::LocalStorage,
                EcmaLocaleSource::AcceptLanguage,
            ] {
                let (path, js_leaf) = match source {
                    EcmaLocaleSource::Path => ("path", path::compile_web_path_module()),
                    EcmaLocaleSource::Cookie => ("cookie", cookie::compile_web_cookie_module()),
                    EcmaLocaleSource::LocalStorage => (
                        "local-storage",
                        local_storage::compile_web_local_storage_module(),
                    ),
                    EcmaLocaleSource::AcceptLanguage => (
                        "accept-language",
                        accept_language::compile_web_accept_language_module(),
                    ),
                };
                assert_eq!(
                    js.code.contains(&format!("from \"./web/{path}.js\"")),
                    options.sources.contains(&source)
                );
                if options.sources.contains(&source) {
                    snapshots.push((root.join(format!("{name}/web/{path}.js")), js_leaf.code));
                }
            }
            if !options.exclude.is_empty() {
                snapshots.push((
                    root.join(format!("{name}/web/routes.js")),
                    routes::compile_web_routes_module().code,
                ));
            }
            snapshots.extend([
                (
                    root.join(format!("{name}/typescript/web.d.ts")),
                    types::generate_web_declaration(),
                ),
                (root.join(format!("{name}/web.js")), js.code),
                (root.join(format!("{name}/web.js.map")), js.source_map),
            ]);
        }
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, code) in &snapshots {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, code).unwrap();
            }
        }
        for (path, code) in snapshots {
            assert_eq!(
                code,
                std::fs::read_to_string(&path).unwrap(),
                "{}",
                path.display()
            );
        }
    }
}
