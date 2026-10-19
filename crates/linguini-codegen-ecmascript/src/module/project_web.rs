//! Optional web artifacts owned by the ECMAScript backend.
use super::project_output::{data, rendered, ProjectFile};
use super::{EcmaLinkMode, EcmaLocaleSource, ValidatedEcmaProject};
pub(super) fn append(project: &ValidatedEcmaProject<'_>, files: &mut Vec<ProjectFile>) {
    let Some(web) = &project.options.web else {
        return;
    };
    let declarations = project.options.declaration;
    rendered(files, super::web_facade::compile_web_module(web));
    if declarations {
        data(
            files,
            "web.d.ts",
            super::web_types::generate_web_declaration(),
        );
    }
    for source in &web.sources {
        let module = match source {
            EcmaLocaleSource::Path => super::web_path::compile_web_path_module(),
            EcmaLocaleSource::Cookie => super::web_cookie::compile_web_cookie_module(),
            EcmaLocaleSource::LocalStorage => {
                super::web_local_storage::compile_web_local_storage_module()
            }
            EcmaLocaleSource::AcceptLanguage => {
                super::web_accept_language::compile_web_accept_language_module()
            }
        };
        rendered(files, module);
        if declarations {
            data(
                files,
                format!("web/{}.d.ts", source.as_str()),
                source_declaration(*source),
            );
        }
    }
    if !web.exclude.is_empty() {
        rendered(files, super::web_routes::compile_web_routes_module());
        if declarations {
            data(
                files,
                "web/routes.d.ts",
                super::web_routes::generate_web_routes_declaration(),
            );
        }
    }
    match web.link_mode {
        EcmaLinkMode::Transform => {
            rendered(
                files,
                super::web_link_transform::compile_web_link_transform_module(),
            );
            if declarations {
                data(
                    files,
                    "web/link-transform.d.ts",
                    super::web_link_transform::generate_web_link_transform_declaration(),
                );
            }
        }
        EcmaLinkMode::Runtime => {
            rendered(
                files,
                super::web_runtime_links::compile_web_runtime_links_module(),
            );
            if declarations {
                data(
                    files,
                    "web/runtime-links.d.ts",
                    super::web_runtime_links::generate_web_runtime_links_declaration(),
                );
            }
        }
        EcmaLinkMode::Manual => {}
    }
}

fn source_declaration(source: EcmaLocaleSource) -> String {
    match source {
        EcmaLocaleSource::Path => super::web_path::generate_web_path_declaration(),
        EcmaLocaleSource::Cookie => super::web_cookie::generate_web_cookie_declaration(),
        EcmaLocaleSource::LocalStorage => {
            super::web_local_storage::generate_web_local_storage_declaration()
        }
        EcmaLocaleSource::AcceptLanguage => {
            super::web_accept_language::generate_web_accept_language_declaration()
        }
    }
}
