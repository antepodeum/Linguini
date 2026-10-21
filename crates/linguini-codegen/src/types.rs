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
}
