use crate::{
    validate_web_options, EcmaLinkMode, EcmaLocaleSource, EcmaWebOptions, WebCodegenError,
};
use linguini_codegen_ecmascript::{generated_module_files, EcmaGeneratedFile};

/// Web artifacts are composed explicitly alongside language and framework artifacts.
pub struct WebProjectOptions {
    pub web: EcmaWebOptions,
    pub declaration: bool,
}

/// Emit only selected web artifacts, validating their closed policy before rendering.
pub fn generate_project_files(
    options: &WebProjectOptions,
    source_maps: bool,
) -> Result<Vec<EcmaGeneratedFile>, WebCodegenError> {
    validate_web_options(&options.web)?;
    let web = &options.web;
    let declarations = options.declaration;
    let mut files = Vec::new();
    let rendered = |files: &mut Vec<EcmaGeneratedFile>, module| {
        files.extend(generated_module_files(module, source_maps))
    };
    rendered(&mut files, super::facade::compile_web_module(web));
    if declarations {
        data(
            &mut files,
            "web.d.ts",
            super::types::generate_web_declaration(),
        );
    }
    for source in &web.sources {
        let module = match source {
            EcmaLocaleSource::Path => super::path::compile_web_path_module(),
            EcmaLocaleSource::Cookie => super::cookie::compile_web_cookie_module(),
            EcmaLocaleSource::LocalStorage => {
                super::local_storage::compile_web_local_storage_module()
            }
            EcmaLocaleSource::AcceptLanguage => {
                super::accept_language::compile_web_accept_language_module()
            }
        };
        rendered(&mut files, module);
        if declarations {
            data(
                &mut files,
                format!("web/{}.d.ts", source.as_str()),
                source_declaration(*source),
            );
        }
    }
    if !web.exclude.is_empty() {
        rendered(&mut files, super::routes::compile_web_routes_module());
        if declarations {
            data(
                &mut files,
                "web/routes.d.ts",
                super::routes::generate_web_routes_declaration(),
            );
        }
    }
    match web.link_mode {
        EcmaLinkMode::Transform => {
            rendered(
                &mut files,
                super::link_transform::compile_web_link_transform_module(),
            );
            if declarations {
                data(
                    &mut files,
                    "web/link-transform.d.ts",
                    super::link_transform::generate_web_link_transform_declaration(),
                );
            }
        }
        EcmaLinkMode::Runtime => {
            rendered(
                &mut files,
                super::runtime_links::compile_web_runtime_links_module(),
            );
            if declarations {
                data(
                    &mut files,
                    "web/runtime-links.d.ts",
                    super::runtime_links::generate_web_runtime_links_declaration(),
                );
            }
        }
        EcmaLinkMode::Manual => {}
    }
    Ok(files)
}

fn source_declaration(source: EcmaLocaleSource) -> String {
    match source {
        EcmaLocaleSource::Path => super::path::generate_web_path_declaration(),
        EcmaLocaleSource::Cookie => super::cookie::generate_web_cookie_declaration(),
        EcmaLocaleSource::LocalStorage => {
            super::local_storage::generate_web_local_storage_declaration()
        }
        EcmaLocaleSource::AcceptLanguage => {
            super::accept_language::generate_web_accept_language_declaration()
        }
    }
}

fn data(files: &mut Vec<EcmaGeneratedFile>, path: impl Into<String>, contents: String) {
    files.push(EcmaGeneratedFile {
        path: path.into(),
        contents,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EcmaLocaleSwitchPlan;
    use std::collections::BTreeMap;

    #[test]
    fn project_artifacts_follow_only_selected_capabilities_and_output_modes() {
        let cases = [
            (
                EcmaWebOptions {
                    sources: vec![],
                    locale_switch: EcmaLocaleSwitchPlan {
                        writes_path: false,
                        writes_cookie: false,
                        writes_local_storage: false,
                    },
                    link_mode: EcmaLinkMode::Manual,
                    ..Default::default()
                },
                vec!["web"],
            ),
            (
                EcmaWebOptions {
                    sources: vec![EcmaLocaleSource::Path],
                    locale_switch: EcmaLocaleSwitchPlan {
                        writes_path: true,
                        writes_cookie: false,
                        writes_local_storage: false,
                    },
                    link_mode: EcmaLinkMode::Transform,
                    ..Default::default()
                },
                vec!["web", "web/path", "web/link-transform"],
            ),
            (
                EcmaWebOptions {
                    sources: vec![
                        EcmaLocaleSource::LocalStorage,
                        EcmaLocaleSource::AcceptLanguage,
                        EcmaLocaleSource::Cookie,
                        EcmaLocaleSource::Path,
                    ],
                    locale_switch: EcmaLocaleSwitchPlan {
                        writes_path: true,
                        writes_cookie: true,
                        writes_local_storage: true,
                    },
                    exclude: vec!["/assets".into()],
                    ..Default::default()
                },
                vec![
                    "web",
                    "web/local-storage",
                    "web/accept-language",
                    "web/cookie",
                    "web/path",
                    "web/routes",
                    "web/runtime-links",
                ],
            ),
        ];
        for (web, modules) in cases {
            for declaration in [false, true] {
                let options = WebProjectOptions {
                    web: web.clone(),
                    declaration,
                };
                let plain = generate_project_files(&options, false).unwrap();
                for source_maps in [false, true] {
                    let files = generate_project_files(&options, source_maps).unwrap();
                    let expected: Vec<_> = modules
                        .iter()
                        .flat_map(|path| {
                            let mut paths = vec![format!("{path}.js")];
                            if source_maps {
                                paths.push(format!("{path}.js.map"));
                            }
                            if declaration {
                                paths.push(format!("{path}.d.ts"));
                            }
                            paths
                        })
                        .collect();
                    assert_eq!(
                        files.iter().map(|file| &file.path).collect::<Vec<_>>(),
                        expected.iter().collect::<Vec<_>>()
                    );
                    let contents: BTreeMap<_, _> = files
                        .iter()
                        .map(|file| (file.path.as_str(), file.contents.as_str()))
                        .collect();
                    for file in &plain {
                        let mapped = contents[file.path.as_str()];
                        if source_maps && file.path.ends_with(".js") {
                            let name = file.path.rsplit('/').next().unwrap();
                            let trailer = format!("//# sourceMappingURL={name}.map\n");
                            assert_eq!(mapped.strip_suffix(&trailer), Some(file.contents.as_str()));
                            assert!(contents[format!("{}.map", file.path).as_str()]
                                .contains(&format!("\"file\":\"{}\"", file.path)));
                        } else {
                            assert_eq!(mapped, file.contents);
                            assert!(!mapped.contains("sourceMappingURL="));
                        }
                    }
                }
            }
        }
    }
}
