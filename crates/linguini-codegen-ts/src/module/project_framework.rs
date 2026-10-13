//! Framework-selected project artifacts; shared target compilers own every implementation.

use super::project_output::{data, rendered, ProjectFile};
use super::{
    TypeScriptFramework, TypeScriptLinkMode, TypeScriptLocaleSource, ValidatedTypeScriptProject,
};
use crate::ecmascript::{EcmaScriptTarget, RenderedEcmaModule};

fn select(
    target: EcmaScriptTarget,
    typescript: impl FnOnce() -> RenderedEcmaModule,
    javascript: impl FnOnce() -> RenderedEcmaModule,
) -> RenderedEcmaModule {
    if target.is_typescript() {
        typescript()
    } else {
        javascript()
    }
}

pub(super) fn append(
    project: &ValidatedTypeScriptProject<'_>,
    target: EcmaScriptTarget,
    files: &mut Vec<ProjectFile>,
) {
    let options = &project.options;
    if !options
        .framework
        .is_some_and(TypeScriptFramework::needs_svelte_module)
    {
        return;
    }
    let kit = options.framework == Some(TypeScriptFramework::SvelteKit);
    let declarations = options.declaration;
    if let Some(web) = &options.web {
        rendered(
            files,
            select(
                target,
                || super::web_facade::compile_typescript_web_module(web),
                || super::web_facade::compile_javascript_web_module(web),
            ),
        );
        if declarations {
            data(
                files,
                "web.d.ts",
                super::web_types::generate_web_declaration(),
            );
        }
        for source in &web.sources {
            let module = match source {
                TypeScriptLocaleSource::Path => select(
                    target,
                    super::web_path::compile_typescript_web_path_module,
                    super::web_path::compile_javascript_web_path_module,
                ),
                TypeScriptLocaleSource::Cookie => select(
                    target,
                    super::web_cookie::compile_typescript_web_cookie_module,
                    super::web_cookie::compile_javascript_web_cookie_module,
                ),
                TypeScriptLocaleSource::LocalStorage => select(
                    target,
                    super::web_local_storage::compile_typescript_web_local_storage_module,
                    super::web_local_storage::compile_javascript_web_local_storage_module,
                ),
                TypeScriptLocaleSource::AcceptLanguage => select(
                    target,
                    super::web_accept_language::compile_typescript_web_accept_language_module,
                    super::web_accept_language::compile_javascript_web_accept_language_module,
                ),
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
            rendered(
                files,
                select(
                    target,
                    super::web_routes::compile_typescript_web_routes_module,
                    super::web_routes::compile_javascript_web_routes_module,
                ),
            );
            if declarations {
                data(
                    files,
                    "web/routes.d.ts",
                    super::web_routes::generate_web_routes_declaration(),
                );
            }
        }
        match web.link_mode {
            TypeScriptLinkMode::Transform => {
                rendered(
                    files,
                    select(
                        target,
                        super::web_link_transform::compile_typescript_web_link_transform_module,
                        super::web_link_transform::compile_javascript_web_link_transform_module,
                    ),
                );
                if declarations {
                    data(
                        files,
                        "web/link-transform.d.ts",
                        super::web_link_transform::generate_web_link_transform_declaration(),
                    );
                }
            }
            TypeScriptLinkMode::Runtime => {
                rendered(
                    files,
                    select(
                        target,
                        super::web_runtime_links::compile_typescript_web_runtime_links_module,
                        super::web_runtime_links::compile_javascript_web_runtime_links_module,
                    ),
                );
                if declarations {
                    data(
                        files,
                        "web/runtime-links.d.ts",
                        super::web_runtime_links::generate_web_runtime_links_declaration(),
                    );
                }
            }
            TypeScriptLinkMode::Manual => {}
        }
        if kit && web.features().has_cookie {
            rendered(
                files,
                select(
                    target,
                    super::web_server_cookie::compile_typescript_web_server_cookie_module,
                    super::web_server_cookie::compile_javascript_web_server_cookie_module,
                ),
            );
            if declarations {
                data(
                    files,
                    "web/server-cookie.d.ts",
                    super::web_server_cookie::generate_web_server_cookie_declaration(),
                );
            }
        }
        if kit && web.switch_route.is_some() {
            rendered(
                files,
                select(
                    target,
                    || {
                        super::web_switch_route::compile_typescript_web_switch_route_module(web)
                            .expect("selected switch route")
                    },
                    || {
                        super::web_switch_route::compile_javascript_web_switch_route_module(web)
                            .expect("selected switch route")
                    },
                ),
            );
            if declarations {
                data(
                    files,
                    "web/switch-route.d.ts",
                    super::web_switch_route::generate_web_switch_route_declaration(),
                );
            }
        }
        rendered(
            files,
            select(
                target,
                || super::svelte_effects::compile_typescript_svelte_effects_module(web, kit),
                || super::svelte_effects::compile_javascript_svelte_effects_module(web, kit),
            ),
        );
        if declarations {
            data(
                files,
                "svelte-effects.svelte.d.ts",
                super::svelte_effects::generate_svelte_effects_declaration(),
            );
        }
        rendered(
            files,
            select(
                target,
                || super::svelte_control::compile_typescript_svelte_control_module(web, kit),
                || super::svelte_control::compile_javascript_svelte_control_module(web, kit),
            ),
        );
        if declarations {
            data(
                files,
                "svelte-control.d.ts",
                super::svelte_control_types::generate_svelte_control_declaration(kit),
            );
        }
    }
    let mode = super::svelte_locale::SvelteLocaleMode::from_framework(options.web.is_some(), kit);
    rendered(
        files,
        select(
            target,
            || super::svelte_locale::compile_typescript_svelte_locale_module(mode),
            || super::svelte_locale::compile_javascript_svelte_locale_module(mode),
        ),
    );
    if declarations {
        data(
            files,
            "svelte-locale.svelte.d.ts",
            super::svelte_locale::generate_svelte_locale_declaration(mode),
        );
    }
    rendered(
        files,
        select(
            target,
            || super::svelte_facade::compile_typescript_svelte_module(options.web.is_some()),
            || super::svelte_facade::compile_javascript_svelte_module(options.web.is_some()),
        ),
    );
    if declarations {
        data(
            files,
            "svelte.d.ts",
            super::svelte_facade_types::generate_svelte_declaration(options.web.is_some()),
        );
    }
    if let Some(web) = options.web.as_ref().filter(|_| kit) {
        rendered(
            files,
            select(
                target,
                || super::sveltekit::compile_typescript_sveltekit_control_module(web),
                || super::sveltekit::compile_javascript_sveltekit_control_module(web),
            ),
        );
        if declarations {
            data(
                files,
                "sveltekit-control.d.ts",
                super::sveltekit_types::generate_control_declaration(),
            );
        }
        rendered(
            files,
            select(
                target,
                || super::sveltekit::compile_typescript_sveltekit_module(web),
                || super::sveltekit::compile_javascript_sveltekit_module(web),
            ),
        );
        if declarations {
            data(
                files,
                "linguini-app.d.ts",
                super::sveltekit_types::generate_ambient_declaration(),
            );
        }
    }
}

fn source_declaration(source: TypeScriptLocaleSource) -> String {
    match source {
        TypeScriptLocaleSource::Path => super::web_path::generate_web_path_declaration(),
        TypeScriptLocaleSource::Cookie => super::web_cookie::generate_web_cookie_declaration(),
        TypeScriptLocaleSource::LocalStorage => {
            super::web_local_storage::generate_web_local_storage_declaration()
        }
        TypeScriptLocaleSource::AcceptLanguage => {
            super::web_accept_language::generate_web_accept_language_declaration()
        }
    }
}
