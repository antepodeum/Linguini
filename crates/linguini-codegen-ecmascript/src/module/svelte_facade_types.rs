//! Declaration interfaces for the context-only and web-enabled Svelte facades.

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaReExport,
    EcmaScriptTarget, EcmaStatement,
};

const WEB_INTERFACE: &str = r#"export interface LinguiniRune<Locale extends string, Linguini> extends LinguiniSvelteControl<Locale> {
  readonly messages: Linguini;
  readonly l: Linguini;
}

export declare const linguini: LinguiniRune<Locale, Linguini>;
export declare const l: Linguini;
export declare const messages: Linguini;
export declare const setLocale: LinguiniRune<Locale, Linguini>["setLocale"];
export declare const localizeHref: LinguiniRune<Locale, Linguini>["localizeHref"];
export declare const localizeUrl: LinguiniRune<Locale, Linguini>["localizeUrl"];
export declare const shouldLocalizeHref: LinguiniRune<Locale, Linguini>["shouldLocalizeHref"];
export declare const shouldLocalizeLink: LinguiniRune<Locale, Linguini>["shouldLocalizeLink"];
export declare const localizeHrefAttribute: LinguiniRune<Locale, Linguini>["localizeHrefAttribute"];
export declare const delocalizeUrl: LinguiniRune<Locale, Linguini>["delocalizeUrl"];
export declare const alternateLinks: LinguiniRune<Locale, Linguini>["alternateLinks"];
"#;
const CONTEXT_INTERFACE: &str = r#"export interface LinguiniRune<Locale extends string, Linguini> {
  readonly messages: Linguini;
  readonly l: Linguini;
  readonly locale: Locale;
  readonly lang: Locale;
  readonly direction: TextDirection;
  readonly textDirection: TextDirection;
  readonly htmlAttrs: { lang: Locale; dir: TextDirection };
  setLocale(locale: Locale | string): Promise<Locale>;
}

export declare const linguini: LinguiniRune<Locale, Linguini>;
export declare const l: Linguini;
export declare const messages: Linguini;
export declare const setLocale: LinguiniRune<Locale, Linguini>["setLocale"];
"#;

pub(super) fn generate_svelte_declaration(web: bool) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "svelte.d.ts",
        None,
    ));
    let mut names = vec!["Locale", "Linguini"];
    if !web {
        names.push("TextDirection");
    }
    module.push_import(EcmaImport {
        specifier: "./index".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(
            names
                .into_iter()
                .map(|name| EcmaNamedImport::new(name, name))
                .collect(),
        ),
    });
    if web {
        module.push_import(EcmaImport {
            specifier: "./svelte-control.js".to_owned(),
            bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
                "LinguiniSvelteControl",
                "LinguiniSvelteControl",
            )]),
        });
        module.push_re_export(EcmaReExport::type_named(
            "./svelte-control.js",
            vec![EcmaNamedImport::new(
                "LinguiniSetLocaleOptions",
                "LinguiniSetLocaleOptions",
            )],
        ));
    }
    module.push_statement(EcmaStatement::type_declaration(
        if web {
            WEB_INTERFACE
        } else {
            CONTEXT_INTERFACE
        },
        None,
    ));
    module.render_code()
}
