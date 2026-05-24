use crate::{generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions};
use linguini_ir::{lower_locale, lower_schema};
use linguini_syntax::{parse_locale, parse_schema};
use std::fs;
use std::path::Path;

#[test]
fn generated_module_snapshot_is_stable() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lgs"
    ))
    .expect("schema");
    let locale =
        parse_locale(include_str!("../../../tests/fixtures/golden/locale/ru.lgl")).expect("locale");

    let files = generate_typescript_project_files(
        &lower_schema(&schema),
        &[TypeScriptLocaleModule {
            locale: "ru".to_owned(),
            module: lower_locale(&locale),
        }],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: Some("ru".to_owned()),
            ..TypeScriptProjectOptions::default()
        },
    )
    .expect("project files");

    for file in files {
        let snapshot_path = format!("tests/fixtures/golden/snapshots/ts/{}", file.path);
        assert_snapshot(&snapshot_path, &file.contents);
    }
}

fn assert_snapshot(path: &str, snapshot: &str) {
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        let path = repo_root().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create snapshot dir");
        }
        fs::write(path, snapshot).expect("write snapshot");
    }

    let expected = fs::read_to_string(repo_root().join(path)).expect("read snapshot");
    assert_eq!(snapshot, expected);
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn project_codegen_owns_multilocale_index_files() {
    use crate::{
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions,
    };
    use linguini_ir::IrModule;

    let files = generate_typescript_project_files(
        &IrModule::default(),
        &[
            TypeScriptLocaleModule {
                locale: "en".to_owned(),
                module: IrModule::default(),
            },
            TypeScriptLocaleModule {
                locale: "ru".to_owned(),
                module: IrModule::default(),
            },
        ],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        },
    )
    .expect("project codegen");

    let index = files
        .iter()
        .find(|file| file.path == "index.ts")
        .expect("index.ts");
    assert!(index
        .contents
        .contains("import locale_en from \"./locales/en\";"));
    assert!(index
        .contents
        .contains("import locale_ru from \"./locales/ru\";"));
    assert!(index.contents.contains("ru: locale_ru"));
    assert_eq!(
        files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        [
            "shared.ts",
            "shared.d.ts",
            "locales/en.ts",
            "locales/en.d.ts",
            "locales/ru.ts",
            "locales/ru.d.ts",
            "index.ts",
            "index.d.ts",
        ]
    );
    for forbidden in [
        ["coo", "kie"].concat(),
        ["localize", "Href"].concat(),
        ["Middle", "ware"].concat(),
    ] {
        assert!(!index.contents.contains(&forbidden));
    }
}

#[test]
fn project_runtime_index_snapshot_is_stable() {
    use linguini_ir::IrModule;

    let files = generate_typescript_project_files(
        &IrModule::default(),
        &[
            TypeScriptLocaleModule {
                locale: "en".to_owned(),
                module: IrModule::default(),
            },
            TypeScriptLocaleModule {
                locale: "ru".to_owned(),
                module: IrModule::default(),
            },
        ],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        },
    )
    .expect("project codegen");

    let index = files
        .iter()
        .find(|file| file.path == "index.ts")
        .expect("index.ts");
    assert_snapshot(
        "tests/fixtures/golden/snapshots/ts-runtime/index.ts",
        &index.contents,
    );
}

#[test]
fn project_codegen_filters_messages_in_tree_shaking_mode() {
    use crate::{
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions,
    };

    let schema = lower_schema(&parse_schema("keep\ndrop\ngroup { label help }\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale("keep = Keep\ndrop = Drop\ngroup {\n  label = Label\n  help = Help\n}\n")
            .expect("locale"),
    );

    let files = generate_typescript_project_files(
        &schema,
        &[TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }],
        &TypeScriptProjectOptions {
            declaration: true,
            tree_shaking: true,
            included_messages: vec!["keep".to_owned(), "group.label".to_owned()],
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        },
    )
    .expect("project codegen");

    let locale_module = files
        .iter()
        .find(|file| file.path == "locales/en.ts")
        .expect("locale module");
    assert!(locale_module.contents.contains("export function keep"));
    assert!(!locale_module.contents.contains("export function drop"));
    assert!(locale_module.contents.contains("label: \"Label\""));
    assert!(!locale_module.contents.contains("help: \"Help\""));
}

#[test]
fn project_codegen_applies_schema_formatter_aliases() {
    let schema = lower_schema(
        &parse_schema(
            r#"
type Price = Number @currency(code = "EUR")
type ShortDate = Date @date(style = "short")
total(price: Price, created: ShortDate)
raw(price: Price)
"#,
        )
        .expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale(
            r#"
total = Total {price} on {created}
raw = Raw {price @number}
"#,
        )
        .expect("locale"),
    );

    let files = generate_typescript_project_files(
        &schema,
        &[TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }],
        &TypeScriptProjectOptions::default(),
    )
    .expect("project codegen");

    let locale_module = files
        .iter()
        .find(|file| file.path == "locales/en.ts")
        .expect("locale module");
    assert!(!locale_module.contents.contains("FORMATTER_DATA"));
    assert!(!locale_module.contents.contains("GeneratedNumberPattern"));
    assert!(!locale_module.contents.contains("Intl.DateTimeFormat"));
    assert!(!locale_module.contents.contains("replace(/[#0"));
    assert!(locale_module.contents.contains("function formatCurrency("));
    assert!(locale_module
        .contents
        .contains("formatCurrency(price, { code: \"EUR\" })"));
    assert!(locale_module.contents.contains("function formatDate("));
    assert!(locale_module
        .contents
        .contains("formatDate(created, { style: \"short\" })"));
    assert!(locale_module
        .contents
        .contains("export function raw(price: Price): string"));
    assert!(locale_module.contents.contains("formatNumber(price)"));
}

#[test]
fn project_codegen_applies_primitive_schema_formatters() {
    let schema = lower_schema(
        &parse_schema("summary(count: Number, amount: Decimal, created: Date)\n").expect("schema"),
    );
    let locale =
        lower_locale(&parse_locale("summary = {count} / {amount} / {created}\n").expect("locale"));

    let files = generate_typescript_project_files(
        &schema,
        &[TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }],
        &TypeScriptProjectOptions::default(),
    )
    .expect("project codegen");

    let locale_module = files
        .iter()
        .find(|file| file.path == "locales/en.ts")
        .expect("locale module");
    assert!(locale_module.contents.contains("formatNumber(count)"));
    assert!(locale_module.contents.contains("formatNumber(amount)"));
    assert!(locale_module.contents.contains("formatDate(created"));
}

#[test]
fn project_codegen_emits_schema_namespace_objects() {
    use linguini_ir::{IrMessage, IrModule, IrText, IrTextPart};

    fn schema_message(name: &str) -> IrMessage {
        IrMessage {
            name: name.to_owned(),
            docs: Vec::new(),
            parameters: Vec::new(),
            body: None,
        }
    }

    fn locale_message(name: &str, value: &str) -> IrMessage {
        IrMessage {
            name: name.to_owned(),
            docs: Vec::new(),
            parameters: Vec::new(),
            body: Some(IrText {
                parts: vec![IrTextPart::Text(value.to_owned())],
            }),
        }
    }

    let schema = IrModule {
        messages: vec![
            schema_message("checkout.order_ready"),
            schema_message("checkout.cart_summary"),
        ],
        ..IrModule::default()
    };
    let locale = IrModule {
        messages: vec![
            locale_message("checkout.order_ready", "Ready"),
            locale_message("checkout.cart_summary", "Cart"),
        ],
        ..IrModule::default()
    };

    let files = generate_typescript_project_files(
        &schema,
        &[TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }],
        &TypeScriptProjectOptions::default(),
    )
    .expect("project codegen");

    let locale_module = files
        .iter()
        .find(|file| file.path == "locales/en.ts")
        .expect("locale module");
    assert!(locale_module.contents.contains("export const checkout = {"));
    assert!(locale_module.contents.contains("  order_ready: \"Ready\","));
    assert!(locale_module.contents.contains("  cart_summary: \"Cart\","));
    assert!(locale_module.contents.contains("  checkout,"));

    let declaration = files
        .iter()
        .find(|file| file.path == "locales/en.d.ts")
        .expect("locale declaration");
    assert!(declaration
        .contents
        .contains("export declare const checkout: {"));
    assert!(declaration
        .contents
        .contains("  readonly order_ready: string;"));
}

#[test]
fn project_codegen_emits_generated_sveltekit_adapter_when_enabled() {
    use crate::{
        generate_typescript_project_files, TypeScriptFramework, TypeScriptLocaleModule,
        TypeScriptProjectOptions, TypeScriptWebOptions,
    };
    use linguini_ir::IrModule;

    let files = generate_typescript_project_files(
        &IrModule::default(),
        &[TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: IrModule::default(),
        }],
        &TypeScriptProjectOptions {
            framework: Some(TypeScriptFramework::SvelteKit),
            web: TypeScriptWebOptions {
                strategy: vec![
                    "url".to_owned(),
                    "cookie".to_owned(),
                    "header".to_owned(),
                    "baseLocale".to_owned(),
                ],
                cookie_name: "SHOP_LOCALE".to_owned(),
                cookie_path: "/shop".to_owned(),
                cookie_domain: Some("example.com".to_owned()),
                cookie_max_age: 86400,
                cookie_same_site: "strict".to_owned(),
                cookie_secure: true,
                cookie_http_only: true,
                local_storage_key: "SHOP_LOCALE".to_owned(),
                global_variable_name: Some("__SHOP_LOCALE__".to_owned()),
                prefix_default_locale: true,
                base_path: "/shop".to_owned(),
                trailing_slash: "never".to_owned(),
                redirect: false,
                origin: Some("https://example.com".to_owned()),
                exclude: vec!["/api/**".to_owned()],
                localize_links: false,
            },
            ..TypeScriptProjectOptions::default()
        },
    )
    .expect("project codegen");

    let paths = files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"svelte.ts"));
    assert!(paths.contains(&"web.ts"));
    assert!(paths.contains(&"sveltekit.ts"));

    let svelte = files
        .iter()
        .find(|file| file.path == "svelte.ts")
        .expect("svelte module");
    assert!(svelte
        .contents
        .contains("import { createWebI18n } from \"./web\";"));
    assert!(!svelte.contents.contains("@antepod/"));
    assert!(svelte.contents.contains("export const l = linguini.l;"));
    assert!(svelte.contents.contains("cookieName: \"SHOP_LOCALE\""));
    assert!(svelte.contents.contains("localizeLinks: false"));

    let sveltekit = files
        .iter()
        .find(|file| file.path == "sveltekit.ts")
        .expect("sveltekit module");
    assert!(sveltekit
        .contents
        .contains("import { createWebI18n } from \"./web\";"));
    assert!(!sveltekit.contents.contains("@antepod/"));
    assert!(sveltekit.contents.contains("export const handle"));
    assert!(sveltekit.contents.contains("export const reroute"));
    assert!(sveltekit.contents.contains("export const load"));
    assert!(sveltekit
        .contents
        .contains("cookie: event.request.headers.get(\"cookie\") ?? undefined"));

    let sveltekit_declaration = files
        .iter()
        .find(|file| file.path == "sveltekit.d.ts")
        .expect("sveltekit declaration");
    assert!(sveltekit_declaration
        .contents
        .contains("interface SerializedLinguiniContext"));
    assert!(sveltekit_declaration.contents.contains("interface Locals"));
    assert!(sveltekit_declaration
        .contents
        .contains("linguini: LinguiniRequestContext<Locale, Linguini>"));
    assert!(!sveltekit_declaration.contents.contains("@antepod/"));
}

#[test]
fn project_codegen_uses_cldr_text_direction_metadata() {
    use crate::{
        generate_typescript_project_files, TypeScriptLocaleModule, TypeScriptProjectOptions,
    };
    use linguini_ir::IrModule;

    let files = generate_typescript_project_files(
        &IrModule::default(),
        &[
            TypeScriptLocaleModule {
                locale: "en".to_owned(),
                module: IrModule::default(),
            },
            TypeScriptLocaleModule {
                locale: "ar".to_owned(),
                module: IrModule::default(),
            },
        ],
        &TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        },
    )
    .expect("project codegen");

    let index = files
        .iter()
        .find(|file| file.path == "index.ts")
        .expect("index module");
    assert!(index.contents.contains(r#"en: "ltr""#));
    assert!(index.contents.contains(r#"ar: "rtl""#));
}
