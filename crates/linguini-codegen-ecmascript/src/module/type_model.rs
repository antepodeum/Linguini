pub use linguini_codegen::TypeModel;

use super::names::safe_type_identifier;

pub fn render_typescript_type(model: &TypeModel) -> String {
    let value = match model {
        TypeModel::String => minijinja::context! { kind => "string" },
        TypeModel::Boolean => minijinja::context! { kind => "boolean" },
        TypeModel::NumberInput => minijinja::context! { kind => "number_input" },
        TypeModel::DateInput => minijinja::context! { kind => "date_input" },
        TypeModel::Named(name) => {
            minijinja::context! { kind => "named", name => safe_type_identifier(name) }
        }
    };
    crate::templates::render("types/expression", value)
}

pub fn render_jsdoc_type(model: &TypeModel) -> String {
    render_typescript_type(model)
}

pub(super) fn render_string_union(variants: &[String]) -> String {
    crate::templates::render(
        "types/expression",
        minijinja::context! { kind => "string_union", variants },
    )
}

#[cfg(test)]
mod tests {
    use super::{render_jsdoc_type, render_typescript_type, TypeModel};

    #[test]
    fn source_types_lower_once_for_typescript_and_jsdoc_renderers() {
        let cases = [
            ("String", TypeModel::String, "string"),
            ("Boolean", TypeModel::Boolean, "boolean"),
            (
                "string",
                TypeModel::Named("string".into()),
                "__lgl_type_string",
            ),
            (
                "undefined",
                TypeModel::Named("undefined".into()),
                "__lgl_type_undefined",
            ),
            (
                "keyof",
                TypeModel::Named("keyof".into()),
                "__lgl_type_keyof",
            ),
            (
                "intrinsic",
                TypeModel::Named("intrinsic".into()),
                "intrinsic",
            ),
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
                "__lgl_name_shop$Money",
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
