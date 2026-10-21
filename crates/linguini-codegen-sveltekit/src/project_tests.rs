//! Composed framework project behavior remains verified by its owning integration crate.
use super::project_test_support::{generate_project_files, project_options};
use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
#[test]
fn project_codegen_emits_generated_sveltekit_adapter_when_enabled() {
    use super::project_test_support::{
        EcmaFramework, EcmaLinkMode, EcmaLocaleModule, EcmaLocalePrefixMode, EcmaLocaleSource,
        EcmaLocaleSwitchPlan, EcmaProjectOptions, EcmaWebOptions, EcmaWebSwitchRoute,
    };

    let files = generate_project_files(
        &Default::default(),
        &[EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }],
        &EcmaProjectOptions {
            framework: Some(EcmaFramework::SvelteKit),
            web: Some(EcmaWebOptions {
                sources: vec![
                    EcmaLocaleSource::Path,
                    EcmaLocaleSource::Cookie,
                    EcmaLocaleSource::AcceptLanguage,
                ],
                locale_switch: EcmaLocaleSwitchPlan {
                    writes_path: true,
                    writes_cookie: true,
                    writes_local_storage: false,
                },
                cookie_name: "SHOP_LOCALE".to_owned(),
                cookie_path: Some("/shop".to_owned()),
                cookie_domain: Some("example.com".to_owned()),
                cookie_max_age: 86400,
                cookie_same_site: "strict".to_owned(),
                cookie_secure: Some(true),
                cookie_http_only: true,
                local_storage_key: "SHOP_LOCALE".to_owned(),
                locale_prefix: EcmaLocalePrefixMode::Always,
                canonical_redirect: false,
                exclude: vec!["/api/**".to_owned()],
                link_mode: EcmaLinkMode::Manual,
                switch_route: Some(EcmaWebSwitchRoute {
                    path: "/_linguini/locale/{locale}".to_owned(),
                    return_query: "return".to_owned(),
                    status: 303,
                }),
            }),
            ..project_options("en")
        },
    )
    .expect("project codegen");

    let paths = files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"svelte.js"));
    assert!(paths.contains(&"svelte-control.js"));
    assert!(paths.contains(&"svelte-control.d.ts"));
    assert!(paths.contains(&"svelte-locale.svelte.js"));
    assert!(paths.contains(&"svelte-locale.svelte.d.ts"));
    assert!(paths.contains(&"svelte-effects.svelte.js"));
    assert!(paths.contains(&"svelte-effects.svelte.d.ts"));
    assert!(paths.contains(&"web.js"));
    assert!(paths.contains(&"web/path.js"));
    assert!(paths.contains(&"web/cookie.js"));
    assert!(paths.contains(&"web/accept-language.js"));
    assert!(paths.contains(&"web/routes.js"));
    assert!(paths.contains(&"web/server-cookie.js"));
    assert!(paths.contains(&"web/switch-route.js"));
    assert!(!paths.contains(&"web/local-storage.js"));
    let web = files
        .iter()
        .find(|file| file.path == "web.js")
        .expect("web runtime");
    assert!(!web
        .contents
        .contains("for (const source of normalized.sources)"));
    assert!(!web.contents.contains("const resolved = resolve"));
    assert!(web.contents.contains("resolvePathLocale"));
    assert!(web.contents.contains("resolveCookieLocale"));
    assert!(web.contents.contains("resolveAcceptLanguageLocale"));
    assert!(!web.contents.contains("resolveLocalStorageLocale"));
    assert!(web
        .contents
        .contains("import { matchesRoute } from \"./web/routes.js\""));
    assert!(!web.contents.contains("function matchesRoute("));
    let web_declaration = files
        .iter()
        .find(|file| file.path == "web.d.ts")
        .expect("web declaration");
    assert!(web_declaration
        .contents
        .contains("LocalePrefixMode = \"always\" | \"except-default\" | \"never\""));
    assert!(paths.contains(&"sveltekit-control.js"));
    assert!(paths.contains(&"sveltekit-control.d.ts"));
    assert!(paths.contains(&"sveltekit.js"));
    let sveltekit = files
        .iter()
        .find(|file| file.path == "sveltekit.js")
        .expect("sveltekit runtime");
    assert!(sveltekit.contents.contains("localePrefix: \"always\""));
    assert!(sveltekit.contents.contains("handleLocaleSwitchRoute"));
    let switch_route = files
        .iter()
        .find(|file| file.path == "web/switch-route.js")
        .expect("switch route capability");
    assert!(switch_route
        .contents
        .contains("web.options.environment.base + \"/_linguini/locale/\""));
    assert!(!switch_route.contents.contains("{locale}"));
    assert!(switch_route.contents.contains("const redirectStatus = 303"));
    assert!(switch_route.contents.contains("persistLocaleCookie"));
    assert!(switch_route.contents.contains("web.localizeHref"));

    let svelte = files
        .iter()
        .find(|file| file.path == "svelte.js")
        .expect("svelte module");
    let svelte_control = files
        .iter()
        .find(|file| file.path == "svelte-control.js")
        .expect("Svelte control module");
    assert!(svelte_control
        .contents
        .contains("./svelte-effects.svelte.js"));
    assert!(svelte_control
        .contents
        .contains("from \"$app/environment\""));
    assert!(svelte_control
        .contents
        .contains("from \"./sveltekit-navigation.js\""));
    assert!(svelte_control
        .contents
        .contains("await navigate(href, options)"));
    let navigation = files
        .iter()
        .find(|file| file.path == "sveltekit-navigation.js")
        .unwrap();
    assert!(navigation.contents.contains("from \"$app/navigation\""));
    assert!(navigation.contents.contains("return goto(href, {"));
    assert!(!svelte.contents.contains("createWebI18n"));
    assert!(!svelte.contents.contains("@antepod/"));
    assert!(svelte.contents.contains("export const l = linguini.l;"));
    assert!(svelte.contents.contains("./svelte-control.js"));
    assert!(svelte.contents.contains("getLocale: () => controls.locale"));
    assert!(svelte.contents.contains("get locale()"));
    assert!(!svelte.contents.contains("...controls"));
    let svelte_declaration = files
        .iter()
        .find(|file| file.path == "svelte.d.ts")
        .expect("Svelte declaration");
    assert!(svelte_declaration
        .contents
        .contains("export type { LinguiniSetLocaleOptions }"));
    assert!(svelte_control.contents.contains("cookie: true"));
    assert!(svelte_control.contents.contains("navigate: true"));
    assert!(svelte_control.contents.contains("keepFocus: true"));
    assert!(svelte_control.contents.contains("noScroll: true"));
    assert!(svelte_control
        .contents
        .contains("setCurrentLocale(resolved)"));
    assert!(svelte_control
        .contents
        .contains("writeLocaleCookie(web, resolved)"));
    assert!(svelte_control
        .contents
        .contains("clearCurrentLocaleOverride()"));
    assert!(svelte_control
        .contents
        .contains("./svelte-locale.svelte.js"));
    assert!(svelte_control.contents.contains("refreshLinguiniEffects()"));
    assert!(svelte_control
        .contents
        .contains("destroy: destroyLinguiniEffects"));
    for forbidden in ["./index", "./locales/", "./messages"] {
        assert!(!svelte_control.contents.contains(forbidden));
    }
    for forbidden in [
        "MutationObserver",
        "AUTO_LINK_MAX_PENDING_ROOTS",
        "initializeCurrentLocale",
        "hot?.dispose",
    ] {
        assert!(!svelte_control.contents.contains(forbidden));
    }

    let svelte_effects = files
        .iter()
        .find(|file| file.path == "svelte-effects.svelte.js")
        .expect("Svelte effects helper");
    assert!(svelte_effects.contents.contains("createWebLocaleI18n"));
    assert!(svelte_effects
        .contents
        .contains("from \"$app/environment\""));
    assert!(svelte_effects
        .contents
        .contains("initializeCurrentLocale(readInitialLocale())"));
    assert!(!svelte_effects
        .contents
        .contains("startAutoLinkLocalization(getCurrentLocale)"));
    assert!(!svelte_effects
        .contents
        .contains("hot?.dispose(destroyLinguiniEffects)"));
    assert!(svelte_effects.contents.contains("name: \"SHOP_LOCALE\""));
    assert!(svelte_effects
        .contents
        .contains("sources: [\"path\", \"cookie\", \"accept-language\"]"));
    assert!(svelte_effects
        .contents
        .contains("switch: { writesPath: true, writesCookie: true, writesLocalStorage: false }"));
    assert!(svelte_effects
        .contents
        .contains("links: { mode: \"manual\" }"));
    for capability in [
        "readBrowserCapability(() => window.location.href)",
        "readBrowserCapability(() => document.cookie)",
        "readBrowserCapability(() => window.navigator)",
    ] {
        assert!(svelte_effects.contents.contains(capability));
    }
    for forbidden in ["./index", "./locales/", "./messages", "createLinguini"] {
        assert!(!svelte_effects.contents.contains(forbidden));
    }
    assert!(!svelte_effects.contents.contains("MutationObserver"));
    assert!(!svelte_effects.contents.contains("linkEffects"));
    assert!(!svelte_effects.contents.contains("window.localStorage"));
    let svelte_effects_declaration = files
        .iter()
        .find(|file| file.path == "svelte-effects.svelte.d.ts")
        .expect("Svelte effects helper declaration");
    for export in ["web", "refreshLinguiniEffects", "destroyLinguiniEffects"] {
        assert!(svelte_effects_declaration.contents.contains(export));
    }

    let svelte_locale = files
        .iter()
        .find(|file| file.path == "svelte-locale.svelte.js")
        .expect("Svelte locale helper");
    assert!(svelte_locale
        .contents
        .contains("from \"./sveltekit-locale.js\""));
    let request_locale = files
        .iter()
        .find(|file| file.path == "sveltekit-locale.js")
        .unwrap();
    assert!(request_locale.contents.contains("from \"$app/state\""));
    assert!(request_locale
        .contents
        .contains("return page.data?.linguini?.locale"));
    assert!(svelte_locale
        .contents
        .contains("const dataLocale = getRequestLocale()"));
    assert!(svelte_locale
        .contents
        .contains("let clientLocale = $state(baseLocale)"));
    assert!(svelte_locale
        .contents
        .contains("let hasClientOverride = $state(false)"));
    assert!(svelte_locale.contents.contains("hasClientOverride\n    ? resolvedClientLocale\n    : normalizeLocale(dataLocale) ?? resolvedClientLocale"));
    assert!(svelte_locale.contents.contains("hasClientOverride = true"));
    assert!(svelte_locale.contents.contains("hasClientOverride = false"));
    assert!(svelte_locale.contents.contains("from \"./locale.js\""));
    assert!(!svelte_locale.contents.contains("export let clientLocale"));
    assert!(!svelte_locale
        .contents
        .contains("export let hasClientOverride"));
    for forbidden in ["./index", "./locales/", "./messages", "createWebI18n"] {
        assert!(!svelte_locale.contents.contains(forbidden));
    }
    let svelte_locale_declaration = files
        .iter()
        .find(|file| file.path == "svelte-locale.svelte.d.ts")
        .expect("Svelte locale helper declaration");
    assert!(svelte_locale_declaration
        .contents
        .contains("initializeCurrentLocale"));
    assert!(svelte_locale_declaration
        .contents
        .contains("clearCurrentLocaleOverride"));

    let goto_position = svelte_control
        .contents
        .find("await navigate(")
        .expect("host navigation call");
    let clear_position = svelte_control
        .contents
        .find("clearCurrentLocaleOverride();")
        .expect("override clear");
    assert!(clear_position > goto_position);

    let sveltekit = files
        .iter()
        .find(|file| file.path == "sveltekit.js")
        .expect("sveltekit module");
    assert!(sveltekit
        .contents
        .contains("import { createWebI18n } from \"./web.js\";"));
    assert!(sveltekit
        .contents
        .contains("import * as runtime from \"./index.js\";"));
    assert!(!sveltekit.contents.contains("@antepod/"));
    assert!(sveltekit.contents.contains("export const handle"));
    assert!(sveltekit.contents.contains("export const reroute"));
    assert!(sveltekit.contents.contains("export const load"));
    assert!(sveltekit
        .contents
        .contains("cookie: event.request.headers.get(\"cookie\") ?? undefined"));
    assert!(sveltekit
        .contents
        .contains("web.options.locale.switch.writesCookie"));

    let sveltekit_control = files
        .iter()
        .find(|file| file.path == "sveltekit-control.js")
        .expect("sveltekit control module");
    assert!(sveltekit_control
        .contents
        .contains("import(\"./locale.js\").Locale"));
    assert!(sveltekit_control
        .contents
        .contains("LinguiniWebLocale<import(\"./locale.js\").Locale>"));
    assert!(sveltekit_control.contents.contains(".Locale} resolved"));
    assert!(sveltekit_control
        .contents
        .contains("import { createWebLocaleI18n } from \"./web.js\";"));
    assert!(sveltekit_control
        .contents
        .contains("import * as locale from \"./locale.js\";"));
    assert!(sveltekit_control
        .contents
        .contains("web.options.locale.switch.writesCookie"));
    for forbidden in ["./index", "./locales/", "./messages", "createWebI18n"] {
        assert!(!sveltekit_control.contents.contains(forbidden));
    }

    let sveltekit_control_declaration = files
        .iter()
        .find(|file| file.path == "sveltekit-control.d.ts")
        .expect("sveltekit control declaration");
    assert!(sveltekit_control_declaration
        .contents
        .contains("type LinguiniServerLocaleContext"));
    assert!(sveltekit_control_declaration.contents.contains(
        "export type LinguiniSvelteKitLocaleContext = LinguiniServerLocaleContext<Locale>"
    ));
    for forbidden in ["LinguiniRequestContext", "./index", "declare global"] {
        assert!(!sveltekit_control_declaration.contents.contains(forbidden));
    }

    let sveltekit_declaration = files
        .iter()
        .find(|file| file.path == "linguini-app.d.ts")
        .expect("sveltekit declaration");
    assert!(sveltekit_declaration
        .contents
        .contains("interface SerializedLinguiniContext"));
    assert!(sveltekit_declaration.contents.contains("interface Locals"));
    assert!(sveltekit_declaration.contents.contains(
        "LinguiniLocaleRequestContext<Locale> | LinguiniRequestContext<Locale, Linguini>"
    ));
    assert!(sveltekit_declaration.contents.contains("Linguini>"));
    assert!(sveltekit_declaration.contents.contains("./index"));
    assert!(!sveltekit_declaration.contents.contains("@antepod/"));
}

#[test]
fn project_codegen_preserves_each_locale_prefix_mode_in_source_and_declarations() {
    use super::project_test_support::{
        EcmaFramework, EcmaLocaleModule, EcmaLocalePrefixMode, EcmaProjectOptions, EcmaWebOptions,
    };

    for mode in [
        EcmaLocalePrefixMode::Always,
        EcmaLocalePrefixMode::ExceptDefault,
        EcmaLocalePrefixMode::Never,
    ] {
        let files = generate_project_files(
            &Default::default(),
            &[EcmaLocaleModule {
                locale: "en".to_owned(),
                module: Default::default(),
            }],
            &EcmaProjectOptions {
                declaration: true,
                framework: Some(EcmaFramework::SvelteKit),
                web: Some(EcmaWebOptions {
                    locale_prefix: mode,
                    ..EcmaWebOptions::default()
                }),
                ..project_options("en")
            },
        )
        .expect("project codegen");

        let source = files
            .iter()
            .find(|file| file.path == "sveltekit.js")
            .expect("SvelteKit source");
        assert!(source
            .contents
            .contains(&format!("localePrefix: \"{}\"", mode.as_str())));
        for removed in ["prefixDefaultLocale", "basePath", "trailingSlash"] {
            assert!(!source.contents.contains(removed));
        }

        let declaration = files
            .iter()
            .find(|file| file.path == "web.d.ts")
            .expect("web declaration");
        assert!(declaration.contents.contains(
            "export type LocalePrefixMode = \"always\" | \"except-default\" | \"never\";"
        ));
        assert!(declaration
            .contents
            .contains("localePrefix?: LocalePrefixMode;"));
        assert!(!declaration.contents.contains("prefixDefaultLocale"));
    }
}

#[test]
fn project_codegen_emits_only_selected_web_source_and_browser_capabilities() {
    use super::project_test_support::{
        EcmaFramework, EcmaLinkMode, EcmaLocaleModule, EcmaLocaleSource, EcmaLocaleSwitchPlan,
        EcmaProjectOptions, EcmaWebOptions,
    };

    let web_options = EcmaWebOptions {
        sources: vec![EcmaLocaleSource::LocalStorage],
        locale_switch: EcmaLocaleSwitchPlan {
            writes_path: false,
            writes_cookie: false,
            writes_local_storage: true,
        },
        link_mode: EcmaLinkMode::Transform,
        ..EcmaWebOptions::default()
    };
    let features = web_options.features();
    assert!(features.has_local_storage);
    assert!(!features.has_path);
    assert!(!features.has_cookie);
    assert!(!features.has_accept_language);
    assert_eq!(features.link_mode, EcmaLinkMode::Transform);
    let files = generate_project_files(
        &Default::default(),
        &[EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }],
        &EcmaProjectOptions {
            framework: Some(EcmaFramework::Svelte),
            web: Some(web_options),
            ..project_options("en")
        },
    )
    .expect("project codegen");

    let paths = files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"web/local-storage.js"));
    assert!(!paths.contains(&"web/path.js"));
    assert!(!paths.contains(&"web/cookie.js"));
    assert!(!paths.contains(&"web/accept-language.js"));

    let web = files
        .iter()
        .find(|file| file.path == "web.js")
        .expect("web runtime");
    assert!(web.contents.contains("resolveLocalStorageLocale"));
    assert!(!web.contents.contains("resolvePathLocale"));
    assert!(!web.contents.contains("resolveCookieLocale"));
    assert!(!web.contents.contains("resolveAcceptLanguageLocale"));
    assert!(!web.contents.contains("const resolved = resolve"));

    let effects = files
        .iter()
        .find(|file| file.path == "svelte-effects.svelte.js")
        .expect("effects runtime");
    assert!(effects
        .contents
        .contains("readBrowserCapability(() => window.localStorage)"));
    assert!(!effects
        .contents
        .contains("url: readBrowserCapability(() => window.location.href)"));
    assert!(!effects
        .contents
        .contains("cookie: readBrowserCapability(() => document.cookie)"));
    assert!(!effects
        .contents
        .contains("navigator: readBrowserCapability(() => window.navigator)"));
    assert!(effects.contents.contains("links: { mode: \"transform\" }"));
    assert!(!effects.contents.contains("MutationObserver"));
    assert!(!effects.contents.contains("startRuntimeLinkLocalization"));
    let link_transform = files
        .iter()
        .find(|file| file.path == "web/link-transform.js")
        .expect("selected link-transform capability");
    assert!(link_transform.contents.contains("localizeTransformedHref"));

    let control = files
        .iter()
        .find(|file| file.path == "svelte-control.js")
        .expect("control runtime");
    assert!(!control
        .contents
        .contains("web.options.locale.switch.writesPath"));
    assert!(!control.contents.contains("window.location.href"));
}

#[test]
fn project_codegen_allows_closed_web_features_without_sources() {
    use super::project_test_support::{
        EcmaFramework, EcmaLocaleModule, EcmaLocaleSwitchPlan, EcmaProjectOptions, EcmaWebOptions,
    };

    let files = generate_project_files(
        &Default::default(),
        &[EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }],
        &EcmaProjectOptions {
            framework: Some(EcmaFramework::Svelte),
            web: Some(EcmaWebOptions {
                sources: Vec::new(),
                locale_switch: EcmaLocaleSwitchPlan {
                    writes_path: false,
                    writes_cookie: false,
                    writes_local_storage: false,
                },
                ..EcmaWebOptions::default()
            }),
            ..project_options("en")
        },
    )
    .expect("empty source policy is valid");

    let paths = files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"web.js"));
    assert!(paths.contains(&"svelte-effects.svelte.js"));
    assert!(paths.contains(&"web/runtime-links.js"));
    assert!(!paths.contains(&"web/routes.js"));
    assert!(!paths.contains(&"web/link-transform.js"));
    assert!(!paths.contains(&"web/server-cookie.js"));

    let web = files
        .iter()
        .find(|file| file.path == "web.js")
        .expect("web runtime");
    assert!(web.contents.contains("return runtime.baseLocale;"));
    assert!(!web.contents.contains("resolveLocaleSource("));
    assert!(!web
        .contents
        .contains("for (const source of normalized.sources)"));
    assert!(!web.contents.contains("matchesRoute("));

    let effects = files
        .iter()
        .find(|file| file.path == "svelte-effects.svelte.js")
        .expect("effects runtime");
    assert!(effects
        .contents
        .contains("initializeCurrentLocale(web.baseLocale)"));
    assert!(!effects.contents.contains("readInitialLocale"));
    assert!(!effects.contents.contains("readBrowserCapability"));
    assert!(!effects
        .contents
        .contains("readBrowserCapability(() => window."));
    assert!(!effects
        .contents
        .contains("readBrowserCapability(() => document."));

    let control = files
        .iter()
        .find(|file| file.path == "svelte-control.js")
        .expect("control runtime");
    assert!(!control
        .contents
        .contains("web.options.locale.switch.writesPath"));
    assert!(!control.contents.contains("window.location.href"));
}

#[test]
fn project_codegen_manual_link_mode_disables_runtime_observer() {
    use super::project_test_support::{
        EcmaFramework, EcmaLinkMode, EcmaLocaleModule, EcmaProjectOptions, EcmaWebOptions,
    };

    let files = generate_project_files(
        &Default::default(),
        &[EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }],
        &EcmaProjectOptions {
            framework: Some(EcmaFramework::Svelte),
            web: Some(EcmaWebOptions {
                link_mode: EcmaLinkMode::Manual,
                ..EcmaWebOptions::default()
            }),
            ..project_options("en")
        },
    )
    .expect("manual link policy");

    let effects = files
        .iter()
        .find(|file| file.path == "svelte-effects.svelte.js")
        .expect("effects runtime");
    assert!(effects.contents.contains("links: { mode: \"manual\" }"));
    assert!(!effects.contents.contains("MutationObserver"));
    assert!(!effects.contents.contains("linkEffects"));
    assert!(!effects.contents.contains("hot?.dispose"));
    assert!(!files
        .iter()
        .any(|file| file.path == "web/runtime-links.js" || file.path == "web/link-transform.js"));
}

#[test]
fn project_codegen_pathless_sveltekit_controls_drop_navigation_dependencies() {
    use super::project_test_support::{
        EcmaFramework, EcmaLocaleModule, EcmaLocaleSource, EcmaLocaleSwitchPlan,
        EcmaProjectOptions, EcmaWebOptions,
    };

    for sources in [Vec::new(), vec![EcmaLocaleSource::LocalStorage]] {
        let writes_local_storage = sources.contains(&EcmaLocaleSource::LocalStorage);
        let files = generate_project_files(
            &Default::default(),
            &[EcmaLocaleModule {
                locale: "en".to_owned(),
                module: Default::default(),
            }],
            &EcmaProjectOptions {
                framework: Some(EcmaFramework::SvelteKit),
                web: Some(EcmaWebOptions {
                    sources,
                    locale_switch: EcmaLocaleSwitchPlan {
                        writes_path: false,
                        writes_cookie: false,
                        writes_local_storage,
                    },
                    ..EcmaWebOptions::default()
                }),
                ..project_options("en")
            },
        )
        .expect("pathless SvelteKit policy");

        assert!(!files
            .iter()
            .any(|file| file.path == "sveltekit-navigation.js"));
        assert!(files
            .iter()
            .filter(|file| file.path.ends_with(".js"))
            .all(|file| !file.contents.contains("$app/navigation")));
        let control = files
            .iter()
            .find(|file| file.path == "svelte-control.js")
            .expect("SvelteKit control runtime");
        for forbidden in [
            "await goto(",
            "$app/navigation",
            "clearCurrentLocaleOverride",
            "window.location",
        ] {
            assert!(
                !control.contents.contains(forbidden),
                "pathless control retained {forbidden}"
            );
        }
    }
}

#[test]
fn project_codegen_emits_plain_svelte_web_runtime_without_sveltekit_imports() {
    use super::project_test_support::{
        EcmaFramework, EcmaLocaleModule, EcmaProjectOptions, EcmaWebOptions,
    };

    let files = generate_project_files(
        &Default::default(),
        &[EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }],
        &EcmaProjectOptions {
            framework: Some(EcmaFramework::Svelte),
            web: Some(EcmaWebOptions::default()),
            ..project_options("en")
        },
    )
    .expect("plain Svelte web codegen");

    let paths = files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    for path in [
        "web.js",
        "svelte.js",
        "svelte.d.ts",
        "svelte-control.js",
        "svelte-control.d.ts",
        "svelte-locale.svelte.js",
        "svelte-effects.svelte.js",
    ] {
        assert!(paths.contains(&path));
    }
    assert!(!paths.contains(&"sveltekit.js"));
    assert!(!paths.contains(&"sveltekit-control.js"));
    assert!(!paths.contains(&"sveltekit-control.d.ts"));

    let generated = |path: &str| {
        &files
            .iter()
            .find(|file| file.path == path)
            .unwrap_or_else(|| panic!("missing {path}"))
            .contents
    };
    let svelte = generated("svelte.js");
    let control = generated("svelte-control.js");
    let locale = generated("svelte-locale.svelte.js");
    let effects = generated("svelte-effects.svelte.js");
    let declaration = generated("svelte-control.d.ts");

    for output in [svelte, control, locale, effects, declaration] {
        assert!(!output.contains("$app/"));
    }
    assert!(control.contains("typeof window !== \"undefined\""));
    assert!(control.contains("window.location.assign(href)"));
    assert!(control.contains("window.location.replace(href)"));
    assert!(!control.contains("goto("));
    assert!(!control.contains("clearCurrentLocaleOverride"));
    assert!(effects.contains("typeof document !== \"undefined\""));
    assert!(effects.contains("initializeCurrentLocale(readInitialLocale())"));
    assert!(effects.contains("readBrowserCapability(() => document.cookie)"));
    assert!(effects.contains("readBrowserCapability(() => window.navigator)"));
    assert!(locale.contains("let activeLocale = $state(baseLocale)"));
    assert!(locale.contains("export function initializeCurrentLocale"));
    assert!(locale.contains("export function setCurrentLocale"));
    assert!(declaration.contains("state?: unknown"));
    let set_position = control
        .find("setCurrentLocale(resolved)")
        .expect("synchronous locale update");
    let navigation_position = control
        .find("if (options.navigate && web.options.locale.switch.writesPath)")
        .expect("navigation branch");
    assert!(set_position < navigation_position);
    assert!(control.contains("writeLocaleCookie(web, resolved)"));
    assert!(svelte.contains("get locale()"));
    assert!(!svelte.contains("...controls"));
    for forbidden in ["./index", "./locales/", "./messages"] {
        assert!(!control.contains(forbidden));
    }
    for forbidden in ["./index", "./locales/", "./messages", "createLinguini"] {
        assert!(!effects.contains(forbidden));
    }
}

#[test]
fn project_codegen_emits_context_only_svelte_without_web_config() {
    use super::project_test_support::{EcmaFramework, EcmaLocaleModule, EcmaProjectOptions};

    let files = generate_project_files(
        &Default::default(),
        &[EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }],
        &EcmaProjectOptions {
            framework: Some(EcmaFramework::SvelteKit),
            web: None,
            ..project_options("en")
        },
    )
    .expect("project codegen");

    let paths = files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"svelte.js"));
    assert!(paths.contains(&"svelte-locale.svelte.js"));
    assert!(!paths.contains(&"svelte-control.js"));
    assert!(!paths.contains(&"svelte-control.d.ts"));
    assert!(!paths.contains(&"sveltekit-control.js"));
    assert!(!paths.contains(&"sveltekit-control.d.ts"));
    assert!(!paths.contains(&"svelte-effects.svelte.js"));
    assert!(!paths.contains(&"svelte-effects.svelte.d.ts"));
    assert!(!paths.contains(&"web.js"));
    assert!(!paths.contains(&"sveltekit.js"));

    let svelte = files
        .iter()
        .find(|file| file.path == "svelte.js")
        .expect("svelte module");
    assert!(svelte.contents.contains("getLocale: getCurrentLocale"));
    assert!(!svelte.contents.contains("$app/navigation"));
    assert!(!svelte.contents.contains("createWebI18n"));

    let svelte_locale = files
        .iter()
        .find(|file| file.path == "svelte-locale.svelte.js")
        .expect("context locale helper");
    assert!(svelte_locale
        .contents
        .contains("let activeLocale = $state(baseLocale)"));
    assert!(svelte_locale
        .contents
        .contains("normalizeLocale(locale) ?? baseLocale"));
    assert!(!svelte_locale.contents.contains("$app/"));
    assert!(!svelte_locale.contents.contains("./index"));
    assert!(!svelte_locale.contents.contains("./locales/"));
    assert!(!svelte_locale.contents.contains("export let activeLocale"));
    assert!(svelte.contents.contains("./svelte-locale.svelte.js"));
    let svelte_locale_declaration = files
        .iter()
        .find(|file| file.path == "svelte-locale.svelte.d.ts")
        .expect("context locale helper declaration");
    assert!(svelte_locale_declaration
        .contents
        .contains("setCurrentLocale"));
    assert!(!svelte_locale_declaration
        .contents
        .contains("initializeCurrentLocale"));
    assert!(!svelte_locale_declaration
        .contents
        .contains("clearCurrentLocaleOverride"));
}

#[test]
fn project_codegen_preserves_closed_web_source_order_and_file_order() {
    use super::project_test_support::{
        EcmaFramework, EcmaLocaleModule, EcmaLocaleSource, EcmaLocaleSwitchPlan,
        EcmaProjectOptions, EcmaWebOptions,
    };

    let options = EcmaProjectOptions {
        declaration: true,
        framework: Some(EcmaFramework::Svelte),
        web: Some(EcmaWebOptions {
            sources: vec![
                EcmaLocaleSource::AcceptLanguage,
                EcmaLocaleSource::Path,
                EcmaLocaleSource::LocalStorage,
                EcmaLocaleSource::Cookie,
            ],
            locale_switch: EcmaLocaleSwitchPlan {
                writes_path: true,
                writes_cookie: true,
                writes_local_storage: true,
            },
            ..EcmaWebOptions::default()
        }),
        ..project_options("en")
    };
    let files = generate_project_files(
        &Default::default(),
        &[EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }],
        &options,
    )
    .expect("project codegen");

    let paths = files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    let web_start = paths.iter().position(|path| *path == "web.js").unwrap();
    assert_eq!(
        &paths[web_start..web_start + 10],
        [
            "web.js",
            "web.d.ts",
            "web/accept-language.js",
            "web/accept-language.d.ts",
            "web/path.js",
            "web/path.d.ts",
            "web/local-storage.js",
            "web/local-storage.d.ts",
            "web/cookie.js",
            "web/cookie.d.ts",
        ]
    );

    let web = files
        .iter()
        .find(|file| file.path == "web.js")
        .expect("web runtime");
    let ordered = [
        "import { resolveAcceptLanguageLocale } from \"./web/accept-language.js\";",
        "import { resolvePathLocale } from \"./web/path.js\";",
        "import { resolveLocalStorageLocale } from \"./web/local-storage.js\";",
        "import { resolveCookieLocale } from \"./web/cookie.js\";",
        "const resolved_0 = resolveAcceptLanguageLocale",
        "const resolved_1 = resolvePathLocale",
        "const resolved_2 = resolveLocalStorageLocale",
        "const resolved_3 = resolveCookieLocale",
    ];
    let mut previous = 0;
    for needle in ordered {
        let position = web.contents[previous..]
            .find(needle)
            .map(|offset| previous + offset)
            .unwrap_or_else(|| panic!("missing ordered generated web fragment: {needle}"));
        assert!(
            position >= previous,
            "generated web fragment order changed: {needle}"
        );
        previous = position + needle.len();
    }
    assert!(!web.contents.contains("resolveLocaleSource("));
    assert!(!web
        .contents
        .contains("for (const source of normalized.sources)"));
}

#[test]
fn composed_mapped_project_matches_the_strict_typescript_consumer_fixture() {
    use linguini_codegen_ecmascript::{EcmaLocaleModule, EcmaSource};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};
    let schema_source = "/// Greeting\nhello(name: String)\nsection { title }\n";
    let en_source = "hello = Hello {name}\nsection { title = English }\n";
    let fr_source = "hello = Bonjour {name}\n";
    let schema = lower_schema(&parse_schema_in(schema_source, SourceId(7)).unwrap());
    let locales = [
        EcmaLocaleModule {
            locale: "en".into(),
            module: lower_locale(&parse_locale_in(en_source, SourceId(11)).unwrap()),
        },
        EcmaLocaleModule {
            locale: "fr".into(),
            module: lower_locale(&parse_locale_in(fr_source, SourceId(13)).unwrap()),
        },
    ];
    let web = linguini_codegen_ecmascript::EcmaWebOptions::default();
    let options = linguini_codegen_ecmascript::EcmaProjectOptions {
        base_locale: Some("en".into()),
        gitignore: false,
        web: Some(web.clone()),
        ..Default::default()
    };
    let project =
        linguini_codegen_ecmascript::ValidatedEcmaProject::try_new(&schema, &locales, &options)
            .unwrap();
    let sources = [
        EcmaSource::new(SourceId(7), "schema/main.lgs", schema_source),
        EcmaSource::new(SourceId(11), "locale/en/main.lgl", en_source),
        EcmaSource::new(SourceId(13), "locale/fr/main.lgl", fr_source),
    ];
    let mut files =
        linguini_codegen_ecmascript::compile_ecmascript_project_files(&project, &sources).unwrap();
    files.extend(super::generate_project_files(
        &super::SvelteKitProjectOptions {
            web: Some(web),
            declaration: true,
        },
        true,
    ));
    let mut paths = std::collections::BTreeSet::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-complete-project/kit/javascript");
    for file in &files {
        assert!(paths.insert(&file.path), "duplicate artifact {}", file.path);
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            let path = root.join(&file.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &file.contents).unwrap();
        }
        assert_eq!(
            file.contents,
            std::fs::read_to_string(root.join(&file.path)).unwrap(),
            "{}",
            file.path
        );
    }
    for path in [
        "svelte.js",
        "svelte-control.js",
        "svelte-effects.svelte.js",
        "svelte-locale.svelte.js",
        "sveltekit.js",
        "sveltekit-control.js",
        "web/server-cookie.js",
        "sveltekit-locale.js",
        "sveltekit-navigation.js",
    ] {
        assert!(paths.contains(&path.to_owned()));
        assert!(paths.contains(&format!("{path}.map")));
    }
}
