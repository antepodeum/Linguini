//! Framework integration generation. The ECMAScript backend remains independent.

mod host;
mod sveltekit;
mod sveltekit_types;
mod templates;
#[cfg(test)]
mod test_templates;
use linguini_codegen_ecmascript_web::EcmaWebOptions;
#[cfg(test)]
use linguini_codegen_ecmascript_web::{EcmaLocaleSource, EcmaWebSwitchRoute};

pub use sveltekit::{compile_sveltekit_control_module, compile_sveltekit_module};

/// SvelteKit module configuration, separate from the language backend.
pub struct SvelteKitProjectOptions {
    pub web: Option<EcmaWebOptions>,
    pub declaration: bool,
}

/// Reject resolution policies that cannot select a locale for an initial server request.
pub fn validate_module_sources(web: Option<&EcmaWebOptions>) -> Result<(), &'static str> {
    if web.is_some_and(|web| {
        web.sources == [linguini_codegen_ecmascript_web::EcmaLocaleSource::LocalStorage]
    }) {
        return Err("local-storage-only locale resolution cannot determine the locale during SvelteKit SSR; add cookie or path, or use the svelte module");
    }
    Ok(())
}

/// Compose reusable Svelte generation with SvelteKit-owned server adapters.
pub fn generate_project_files(
    options: &SvelteKitProjectOptions,
    source_maps: bool,
) -> Vec<linguini_codegen_ecmascript::EcmaGeneratedFile> {
    use linguini_codegen_ecmascript::{generated_module_files, EcmaGeneratedFile};
    let mut files = Vec::new();
    if let Some(web) = &options.web {
        if web.features().has_cookie {
            files.extend(generated_module_files(
                linguini_codegen_ecmascript_web::compile_web_server_cookie_module(),
                source_maps,
            ));
            if options.declaration {
                files.push(EcmaGeneratedFile {
                    path: "web/server-cookie.d.ts".into(),
                    contents:
                        linguini_codegen_ecmascript_web::generate_web_server_cookie_declaration(),
                });
            }
        }
        if let Some(module) = linguini_codegen_ecmascript_web::compile_web_switch_route_module(web)
        {
            files.extend(generated_module_files(module, source_maps));
            if options.declaration {
                files.push(EcmaGeneratedFile {
                    path: "web/switch-route.d.ts".into(),
                    contents:
                        linguini_codegen_ecmascript_web::generate_web_switch_route_declaration(),
                });
            }
        }
    }
    let host = if let Some(web) = &options.web {
        files.extend(generated_module_files(
            host::compile_locale_module(),
            source_maps,
        ));
        if options.declaration {
            files.push(EcmaGeneratedFile {
                path: "sveltekit-locale.d.ts".into(),
                contents: host::declaration("locale"),
            });
        }
        if web.features().has_path {
            files.extend(generated_module_files(
                host::compile_navigation_module(),
                source_maps,
            ));
        }
        if options.declaration {
            files.push(EcmaGeneratedFile {
                path: "sveltekit-navigation.d.ts".into(),
                contents: host::declaration("navigation"),
            });
        }
        host::bindings(web.features().has_path)
    } else {
        Default::default()
    };
    files.extend(linguini_codegen_svelte::generate_project_files(
        &linguini_codegen_svelte::SvelteProjectOptions {
            web: options.web.clone(),
            declaration: options.declaration,
            host,
        },
        source_maps,
    ));
    if let Some(web) = &options.web {
        files.extend(generated_module_files(
            compile_sveltekit_control_module(web),
            source_maps,
        ));
        if options.declaration {
            files.push(EcmaGeneratedFile {
                path: "sveltekit-control.d.ts".into(),
                contents: sveltekit_types::generate_control_declaration(),
            });
        }
        files.extend(generated_module_files(
            compile_sveltekit_module(web),
            source_maps,
        ));
        if options.declaration {
            files.push(EcmaGeneratedFile {
                path: "linguini-app.d.ts".into(),
                contents: sveltekit_types::generate_ambient_declaration(),
            });
        }
    }
    files
}

#[cfg(test)]
mod project_test_support;
#[cfg(test)]
mod project_tests;

#[cfg(test)]
mod module_configuration_tests {
    use super::*;

    #[test]
    fn request_policy_rejects_client_only_locale_resolution() {
        assert!(validate_module_sources(None).is_ok());
        let web = linguini_codegen_ecmascript_web::parse_module_options(
            &serde_json::json!({"routing":{"localePrefix":"never"}, "locale":{"sources":["local-storage"]}}), "shop").unwrap();
        assert!(validate_module_sources(Some(&web))
            .unwrap_err()
            .contains("SvelteKit SSR"));
        let web = linguini_codegen_ecmascript_web::parse_module_options(
            &serde_json::json!({"routing":{"localePrefix":"never"}, "locale":{"sources":["cookie", "local-storage"]}}), "shop").unwrap();
        assert!(validate_module_sources(Some(&web)).is_ok());
        let empty = linguini_codegen_ecmascript_web::parse_module_options(
            &serde_json::json!({"locale":{"sources":[]}}),
            "shop",
        )
        .unwrap();
        assert!(validate_module_sources(Some(&empty)).is_ok());
    }
}
