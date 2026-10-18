//! Validated ECMAScript project lookup around shared semantic dependency planning.

use super::{TypeScriptCodegenError, ValidatedTypeScriptProject};
pub(crate) use linguini_codegen::MessageDependencyClosure;
use linguini_codegen::MessageDependencyError;

pub(crate) fn message_dependency_closure(
    project: &ValidatedTypeScriptProject<'_>,
    locale: &str,
    message: &str,
) -> Result<MessageDependencyClosure, TypeScriptCodegenError> {
    let locale_entry = project
        .locales
        .iter()
        .find(|entry| entry.locale == locale)
        .ok_or_else(|| TypeScriptCodegenError::UnknownLocale {
            locale: locale.to_owned(),
        })?;
    linguini_codegen::message_dependency_closure(
        project.schema,
        locale,
        &locale_entry.module,
        message,
    )
    .map_err(|error| match error {
        MessageDependencyError::UnknownMessage { message } => {
            TypeScriptCodegenError::UnknownMessage { message }
        }
        MessageDependencyError::MissingMessageImplementation { locale, message } => {
            TypeScriptCodegenError::MissingMessageImplementation { locale, message }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::message_dependency_closure;
    use crate::{TypeScriptLocaleModule, TypeScriptProjectOptions, ValidatedTypeScriptProject};
    use linguini_ir::{
        lower_locale_typed as lower_locale, lower_schema_typed as lower_schema, IrModuleBuilder,
    };
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    fn project_for(schema_text: &str, locale_text: &str) -> ValidatedTypeScriptProject<'static> {
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in(schema_text, SourceId(7)).expect("schema"),
        )));
        let locale = lower_locale(&parse_locale_in(locale_text, SourceId(8)).expect("locale"));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }]));
        let options = TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        ValidatedTypeScriptProject::try_new(schema, locales, &options).expect("validated project")
    }

    fn project() -> ValidatedTypeScriptProject<'static> {
        project_for(
            "type Text = String\nroot(value: Text)\ndrop\n",
            "let helper = Hello\nroot = {helper} {value}\ndrop = Drop\n",
        )
    }

    #[test]
    fn closure_keeps_only_transitive_symbols_and_is_sorted() {
        let project = project();
        let closure = message_dependency_closure(&project, "en", "root").expect("closure");
        assert_eq!(
            closure
                .schema()
                .messages()
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            vec!["root"]
        );
        assert_eq!(
            closure
                .locale_module()
                .variables()
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            vec!["helper"]
        );
        assert!(!closure
            .locale_module()
            .messages()
            .iter()
            .any(|item| item.name == "drop"));
        assert_eq!(closure.schema().type_aliases()[0].name, "Text");
        assert_eq!(closure.source_ids(), &[SourceId(7), SourceId(8)]);
    }

    #[test]
    fn closure_rejects_unknown_paths_precisely() {
        let project = project();
        assert!(matches!(
            message_dependency_closure(&project, "fr", "root"),
            Err(crate::TypeScriptCodegenError::UnknownLocale { locale }) if locale == "fr"
        ));
        assert!(matches!(
            message_dependency_closure(&project, "en", "missing"),
            Err(crate::TypeScriptCodegenError::UnknownMessage { message }) if message == "missing"
        ));
        let mut missing = project_for("root\n", "root = Root\n");
        missing.locales[0].module = linguini_ir::LocaleIr::try_from_module(
            IrModuleBuilder::seeded(&missing.locales[0].module)
                .clear_messages()
                .build()
                .expect("message-free locale preserves unique declaration names"),
        )
        .expect("projection remains locale IR");
        assert!(matches!(
            message_dependency_closure(&missing, "en", "root"),
            Err(crate::TypeScriptCodegenError::MissingMessageImplementation { locale, message })
                if locale == "en" && message == "root"
        ));
    }

    #[test]
    fn closure_keeps_parameter_forms_functions_inline_branches_and_alias_formatters() {
        let project = project_for(
            "type Price = Number @currency(code = \"EUR\")\nenum Fruit { apple }\nsummary(fruit: Fruit, price: Price)\ndrop\n",
            "fn Render(value: String) { _ => {value} }\nimpl Fruit {\n  apple {\n    one => apple\n    other => apples\n    label = Apple\n  }\n}\nsummary = {fn(fruit, copy: Render(fruit.label)) { apple => {copy} _ => fallback }}\ndrop = Drop\n",
        );
        let closure = message_dependency_closure(&project, "en", "summary").expect("closure");
        assert_eq!(closure.schema().messages().len(), 1);
        assert_eq!(closure.schema().type_aliases().len(), 1);
        assert_eq!(closure.schema().enums().len(), 1);
        assert_eq!(closure.locale_module().forms().len(), 1);
        assert_eq!(closure.locale_module().functions().len(), 1);
        assert_eq!(closure.locale_module().forms()[0].name, "Fruit");
        assert_eq!(closure.locale_module().functions()[0].name, "Render");
    }
}
