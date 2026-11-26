use linguini_core::TypeKind;

/// Resolve an alias chain before selecting runtime form or primitive operations.
/// Semantic validation rejects cycles; the bound also makes this safe on raw IR.
pub fn resolve_type_name<'a>(schema: &'a linguini_ir::IrModule, name: &'a str) -> &'a str {
    let mut current = name;
    for _ in 0..schema.type_aliases().len() {
        let Some(alias) = schema
            .type_aliases()
            .iter()
            .find(|alias| alias.name == current)
        else {
            break;
        };
        current = &alias.target;
    }
    current
}

/// Infer a validated expression's value type from its local bindings and explicit formatters.
/// Unresolved/member references return `None`; full reference validation belongs to the IR.
/// Automatic display formatting does not change the type of a forwarded argument.
pub fn expression_value_type(
    expression: &linguini_ir::IrExpression,
    context: &std::collections::BTreeMap<String, String>,
) -> Option<String> {
    use linguini_ir::{is_plural_intrinsic, IrExpressionKind, IrFormatterKind};
    if expression
        .formatters
        .iter()
        .any(|formatter| !matches!(formatter.kind, IrFormatterKind::Unknown(_)))
    {
        return Some("String".to_owned());
    }
    match &expression.kind {
        IrExpressionKind::InlineFunction { .. } => Some("String".to_owned()),
        IrExpressionKind::Call
            if expression.path.len() == 1 && is_plural_intrinsic(&expression.path[0]) =>
        {
            Some("Plural".to_owned())
        }
        IrExpressionKind::Call => Some("String".to_owned()),
        IrExpressionKind::Reference if expression.path.len() == 1 => {
            context.get(&expression.path[0]).cloned()
        }
        IrExpressionKind::Reference => None,
    }
}

/// Backend-neutral public value types accepted by generated localization APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeModel {
    /// Unicode text.
    String,
    /// Boolean input.
    Boolean,
    /// Numeric input whose accepted host representations belong to the backend.
    NumberInput,
    /// Date input whose accepted host representations belong to the backend.
    DateInput,
    /// A source-level named type, before target-specific identifier escaping.
    Named(String),
}

impl TypeModel {
    /// Resolves source primitives while retaining user-defined type identities.
    pub fn from_source_name(name: &str) -> Self {
        match TypeKind::from_name(name) {
            Some(TypeKind::String) => Self::String,
            Some(TypeKind::Boolean) => Self::Boolean,
            Some(TypeKind::Number | TypeKind::Decimal) => Self::NumberInput,
            Some(TypeKind::Date) => Self::DateInput,
            None => Self::Named(name.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TypeModel;

    #[test]
    fn source_types_preserve_semantics_without_target_spellings() {
        for (name, expected) in [
            ("String", TypeModel::String),
            ("Boolean", TypeModel::Boolean),
            ("Number", TypeModel::NumberInput),
            ("Decimal", TypeModel::NumberInput),
            ("Date", TypeModel::DateInput),
            ("shop.Money", TypeModel::Named("shop.Money".into())),
        ] {
            assert_eq!(TypeModel::from_source_name(name), expected);
        }
    }
    #[test]
    fn scoped_value_types_include_explicit_formatting_without_formatting_forwarded_values() {
        use linguini_ir::{lower_locale, IrTextPart};
        use linguini_syntax::parse_locale;
        use std::collections::BTreeMap;
        let context = BTreeMap::from([("value".into(), "Price".into())]);
        for (source, expected) in [
            ("value", Some("Price")),
            ("value @number", Some("String")),
            ("value @unknown", Some("Price")),
            ("Plural(value)", Some("Plural")),
            ("Echo(value)", Some("String")),
            ("plural(value)", Some("String")),
            ("value.field", None),
        ] {
            let locale = lower_locale(&parse_locale(&format!("root = {{{source}}}\n")).unwrap());
            let IrTextPart::Placeholder(expression) =
                &locale.messages()[0].body.as_ref().unwrap().parts[0]
            else {
                panic!("expression")
            };
            assert_eq!(
                super::expression_value_type(expression, &context).as_deref(),
                expected,
                "{source}"
            );
        }
    }
}
