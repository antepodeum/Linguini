use linguini_core::TypeKind;

use super::names::safe_identifier;

/// Backend-neutral public value types accepted by generated localization APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeModel {
    String,
    Boolean,
    NumberInput,
    DateInput,
    Named(String),
}

impl TypeModel {
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

pub fn render_typescript_type(model: &TypeModel) -> String {
    match model {
        TypeModel::String => "string".to_owned(),
        TypeModel::Boolean => "boolean".to_owned(),
        TypeModel::NumberInput => "number | bigint | string".to_owned(),
        TypeModel::DateInput => "Date | number | string".to_owned(),
        TypeModel::Named(name) => safe_identifier(name),
    }
}

pub fn render_jsdoc_type(model: &TypeModel) -> String {
    match model {
        TypeModel::String => "string".to_owned(),
        TypeModel::Boolean => "boolean".to_owned(),
        TypeModel::NumberInput => "number | bigint | string".to_owned(),
        TypeModel::DateInput => "Date | number | string".to_owned(),
        TypeModel::Named(name) => safe_identifier(name),
    }
}

#[cfg(test)]
mod tests {
    use super::{render_jsdoc_type, render_typescript_type, TypeModel};

    #[test]
    fn source_types_lower_once_for_typescript_and_jsdoc_renderers() {
        let cases = [
            ("String", TypeModel::String, "string"),
            ("Boolean", TypeModel::Boolean, "boolean"),
            ("Number", TypeModel::NumberInput, "number | bigint | string"),
            (
                "Decimal",
                TypeModel::NumberInput,
                "number | bigint | string",
            ),
            ("Date", TypeModel::DateInput, "Date | number | string"),
            (
                "shop.Money",
                TypeModel::Named("shop.Money".to_owned()),
                "__lgl_name_73686F702E4D6F6E6579",
            ),
        ];

        for (source, expected_model, expected_output) in cases {
            let model = TypeModel::from_source_name(source);
            assert_eq!(model, expected_model);
            assert_eq!(render_typescript_type(&model), expected_output);
            assert_eq!(render_jsdoc_type(&model), expected_output);
        }
    }
}
