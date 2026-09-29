//! Declaration surface shared by framework-selected Svelte browser controls.

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement,
};

const BEFORE_STATE: &str = r#"export interface LinguiniSetLocaleOptions {
  navigate?: boolean;
  replaceState?: boolean;
  invalidateAll?: boolean;
  keepFocus?: boolean;
  noScroll?: boolean;
  cookie?: boolean;
  state?: "#;
const AFTER_STATE: &str = r#";
}

export interface LinguiniSvelteControl<Locale extends string> {
  readonly locale: Locale;
  readonly lang: Locale;
  readonly direction: TextDirection;
  readonly textDirection: TextDirection;
  readonly htmlAttrs: { lang: Locale; dir: TextDirection };
  setLocale(locale: Locale | string, options?: LinguiniSetLocaleOptions): Promise<Locale>;
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  shouldLocalizeLink(href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
  destroy(): void;
}

export declare const linguini: LinguiniSvelteControl<Locale>;
export declare const setLocale: LinguiniSvelteControl<Locale>["setLocale"];
export declare const localizeHref: LinguiniSvelteControl<Locale>["localizeHref"];
export declare const localizeUrl: LinguiniSvelteControl<Locale>["localizeUrl"];
export declare const shouldLocalizeHref: LinguiniSvelteControl<Locale>["shouldLocalizeHref"];
export declare const shouldLocalizeLink: LinguiniSvelteControl<Locale>["shouldLocalizeLink"];
export declare const localizeHrefAttribute: LinguiniSvelteControl<Locale>["localizeHrefAttribute"];
export declare const delocalizeUrl: LinguiniSvelteControl<Locale>["delocalizeUrl"];
export declare const alternateLinks: LinguiniSvelteControl<Locale>["alternateLinks"];
export declare const destroy: LinguiniSvelteControl<Locale>["destroy"];
"#;

pub(super) fn generate_svelte_control_declaration(sveltekit: bool) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "svelte-control.d.ts",
        None,
    ));
    for (path, names) in [
        ("./locale", ["Locale", "TextDirection"]),
        ("./web", ["AlternateLink", "LinkLocalizationAttributes"]),
    ] {
        module.push_import(EcmaImport {
            specifier: path.to_owned(),
            bindings: EcmaImportBindings::TypeNamed(
                names
                    .into_iter()
                    .map(|name| EcmaNamedImport::new(name, name))
                    .collect(),
            ),
        });
    }
    let state = if sveltekit {
        "App.PageState"
    } else {
        "unknown"
    };
    module.push_statement(EcmaStatement::type_declaration(
        format!("{BEFORE_STATE}{state}{AFTER_STATE}"),
        None,
    ));
    module.render_code()
}
