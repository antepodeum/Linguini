//! Target annotations and project imports around the shared provider runtime.

use super::{
    locale_identifier, project_locale_import, project_locale_loaders, project_locale_modules,
    EcmaScriptTarget, TypeScriptLocaleModule,
};
use crate::ecmascript::RuntimeCode;

pub(super) fn render(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
    target: EcmaScriptTarget,
) -> String {
    let base_locale = base_locale.expect("validated TypeScript projects have a base locale");
    let mut code = RuntimeCode::default();
    code.generated(project_locale_import(base_locale, target));
    code.typed(
        "import { baseLocale, normalizeLocale, type Locale } from \"./locale\";\n",
        "import { baseLocale, normalizeLocale } from \"./locale.js\";\n",
    );
    code.shared("export {\n  locales,\n  baseLocale,\n  localeDirections,\n  isLocale,\n  normalizeLocale,\n  getTextDirection,\n");
    code.typed(
        "} from \"./locale\";\nexport type { Locale, TextDirection } from \"./locale\";\nexport type * from \"./shared\";\nimport type { LinguiniMessages } from \"./messages\";\nexport type { LinguiniMessages } from \"./messages\";\n\n",
        "} from \"./locale.js\";\n\n/** @typedef {import(\"./locale.js\").Locale} Locale */\n",
    );
    if !target.is_typescript() {
        code.generated(format!(
            "/** @typedef {{typeof {}}} LinguiniMessages */\n",
            locale_identifier(base_locale)
        ));
    }
    code.typed(
        "export const localeModules: Partial<Record<Locale, LinguiniMessages>> = {\n",
        "/** @typedef {LinguiniMessages} Linguini */\n/**\n * @typedef {object} LinguiniProviderOptions\n * @property {(() => Locale)=} getLocale\n * @property {(() => Locale)=} resolveLanguage\n */\n\n/** @type {Partial<Record<Locale, LinguiniMessages>>} */\nexport const localeModules = {\n",
    );
    code.generated(project_locale_modules(base_locale));
    code.typed("} as const;\n\n", "};\n\n");
    code.typed(
        "export const localeLoaders = {\n",
        "/** @type {Record<Locale, () => Promise<LinguiniMessages>>} */\nexport const localeLoaders = {\n",
    );
    code.generated(project_locale_loaders(locales, base_locale, target));
    code.typed(
        "} as const;\n\ntype LinguiniLanguage = Locale;\nexport type Linguini = LinguiniMessages;\n\ntype LinguiniLanguageInput = LinguiniLanguage;\n\n",
        "};\n\n",
    );
    code.generated(super::runtime::runtime(target));
    code.render(target)
}
