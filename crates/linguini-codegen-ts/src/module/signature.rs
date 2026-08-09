//! Shared TypeScript call signatures for generated messages.
//!
//! A schema message has one source-level parameter list, but the bundler API accepts both the
//! historical positional form and a named object form.  Keeping the derivation here means the
//! runtime emitter, declaration emitter, and nested message object all render exactly the same
//! contract.

use linguini_ir::IrMessage;

use super::names::{property_key, safe_identifier, string_literal, ts_type};

const IMPLEMENTATION_ARGS: &str = "__lgl_args";

/// The source property and generated binding for one message parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MessageCallParameter {
    source_name: String,
    binding: String,
    ty: String,
}

impl MessageCallParameter {
    /// The TypeScript object-property spelling for the original source name.
    pub(crate) fn property_key(&self) -> String {
        property_key(&self.source_name)
    }

    fn tuple_label(&self) -> String {
        format!("{}: {}", self.binding, self.ty)
    }

    fn named_property(&self) -> String {
        format!("{}: {}", self.property_key(), self.ty)
    }
}

/// A normalized call contract for one schema message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MessageCallSignature {
    parameters: Vec<MessageCallParameter>,
}

impl MessageCallSignature {
    pub(crate) fn from_message(message: &IrMessage) -> Self {
        Self {
            parameters: message
                .parameters
                .iter()
                .map(|parameter| MessageCallParameter {
                    source_name: parameter.name.clone(),
                    binding: safe_identifier(&parameter.name),
                    ty: ts_type(&parameter.ty),
                })
                .collect(),
        }
    }

    pub(crate) fn is_parameterized(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Positional call parameters, for example `count: number, date: Date | number | string`.
    pub(crate) fn positional_params(&self) -> String {
        self.parameters
            .iter()
            .map(MessageCallParameter::tuple_label)
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Required named-object argument type, preserving source property names.
    pub(crate) fn named_object_type(&self) -> String {
        let properties = self
            .parameters
            .iter()
            .map(MessageCallParameter::named_property)
            .collect::<Vec<_>>()
            .join("; ");
        format!("{{ {properties} }}")
    }

    /// The implementation rest parameter accepted by the generated runtime leaf.
    ///
    /// Tuple labels deliberately use generated bindings while the object branch uses source
    /// property keys.  This gives the implementation one type-safe input shape while keeping
    /// expression references valid for reserved and punctuation-bearing source names.
    pub(crate) fn implementation_rest_params(&self) -> String {
        let positional = format!("[{}]", self.positional_params());
        let named = format!("[args: {}]", self.named_object_type());
        format!("...{IMPLEMENTATION_ARGS}: {positional} | {named}")
    }

    #[cfg(test)]
    pub(crate) fn parameter_bindings(&self) -> Vec<String> {
        self.parameters
            .iter()
            .map(|parameter| parameter.binding.clone())
            .collect()
    }

    /// Emit the one normalization/destructure statement used by implementations.
    pub(crate) fn normalized_bindings_statement(&self) -> String {
        let bindings = self
            .parameters
            .iter()
            .map(|parameter| parameter.binding.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let keys = self
            .parameters
            .iter()
            .map(|parameter| string_literal(&parameter.source_name))
            .collect::<Vec<_>>()
            .join(", ");
        let tuple_type = format!(
            "[{}]",
            self.parameters
                .iter()
                .map(|parameter| parameter.ty.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        format!(
            "const [{bindings}] = normalizeMessageArgs({IMPLEMENTATION_ARGS}, [{keys}]) as {tuple_type};"
        )
    }

    /// The public callable value type used by locale and nested message objects.
    pub(crate) fn callable_type(&self) -> String {
        if !self.is_parameterized() {
            return "string".to_owned();
        }
        format!(
            "{{ ({}): string; (args: {}): string; }}",
            self.positional_params(),
            self.named_object_type()
        )
    }

    /// Render both public overload declarations for a top-level message.
    pub(crate) fn overload_declarations(&self, name: &str, docs: &[String]) -> String {
        if !self.is_parameterized() {
            let mut output = String::new();
            super::names::emit_docs(docs, "", &mut output);
            output.push_str(&format!("export declare const {name}: string;\n\n"));
            return output;
        }

        let mut output = String::new();
        super::names::emit_docs(docs, "", &mut output);
        output.push_str(&format!(
            "export declare function {name}({}): string;\n",
            self.positional_params()
        ));
        super::names::emit_docs(docs, "", &mut output);
        output.push_str(&format!(
            "export declare function {name}(args: {}): string;\n\n",
            self.named_object_type()
        ));
        output
    }

    /// Render both overload declarations which precede one generated TypeScript implementation.
    pub(crate) fn implementation_overloads(&self, name: &str, docs: &[String]) -> String {
        if !self.is_parameterized() {
            return String::new();
        }

        let mut output = String::new();
        super::names::emit_docs(docs, "", &mut output);
        output.push_str(&format!(
            "export function {name}({}): string;\n",
            self.positional_params()
        ));
        super::names::emit_docs(docs, "", &mut output);
        output.push_str(&format!(
            "export function {name}(args: {}): string;\n",
            self.named_object_type()
        ));
        output
    }

    /// Render a callable type with docs on each call signature when embedded in an object type.
    pub(crate) fn callable_type_with_docs(&self, docs: &[String], indent: &str) -> String {
        if !self.is_parameterized() {
            return self.callable_type();
        }
        let mut output = String::from("{\n");
        super::names::emit_docs(docs, &format!("{indent}  "), &mut output);
        output.push_str(&format!(
            "{indent}  ({}): string;\n",
            self.positional_params()
        ));
        super::names::emit_docs(docs, &format!("{indent}  "), &mut output);
        output.push_str(&format!(
            "{indent}  (args: {}): string;\n{indent}}}",
            self.named_object_type()
        ));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::MessageCallSignature;
    use linguini_ir::{IrMessage, IrParameter};

    fn message(parameters: &[(&str, &str)]) -> IrMessage {
        IrMessage {
            name: "message".to_owned(),
            docs: vec!["A message".to_owned()],
            parameters: parameters
                .iter()
                .map(|(name, ty)| IrParameter {
                    name: (*name).to_owned(),
                    ty: (*ty).to_owned(),
                })
                .collect(),
            body: None,
        }
    }

    #[test]
    fn parameterless_messages_remain_values() {
        let signature = MessageCallSignature::from_message(&message(&[]));
        assert_eq!(signature.callable_type(), "string");
        assert_eq!(signature.positional_params(), "");
        assert_eq!(
            signature.overload_declarations("message", &[]),
            "export declare const message: string;\n\n"
        );
    }

    #[test]
    fn one_parameter_has_positional_and_named_shapes() {
        let signature = MessageCallSignature::from_message(&message(&[("count", "Number")]));
        assert_eq!(
            signature.positional_params(),
            "count: number | bigint | string"
        );
        assert_eq!(
            signature.named_object_type(),
            "{ count: number | bigint | string }"
        );
        assert_eq!(
            signature.implementation_rest_params(),
            "...__lgl_args: [count: number | bigint | string] | [args: { count: number | bigint | string }]"
        );
        assert_eq!(
            signature.normalized_bindings_statement(),
            "const [count] = normalizeMessageArgs(__lgl_args, [\"count\"]) as [number | bigint | string];"
        );
        assert_eq!(
            signature.callable_type(),
            "{ (count: number | bigint | string): string; (args: { count: number | bigint | string }): string; }"
        );
    }

    #[test]
    fn multiple_parameters_keep_order_and_required_properties() {
        let signature =
            MessageCallSignature::from_message(&message(&[("first", "String"), ("when", "Date")]));
        assert_eq!(
            signature.positional_params(),
            "first: string, when: Date | number | string"
        );
        assert_eq!(
            signature.named_object_type(),
            "{ first: string; when: Date | number | string }"
        );
        assert_eq!(signature.parameter_bindings(), vec!["first", "when"]);
    }

    #[test]
    fn reserved_and_punctuation_names_are_collision_safe() {
        let signature = MessageCallSignature::from_message(&message(&[
            ("default", "String"),
            ("__lgl_args", "String"),
            ("display-name", "String"),
            ("__proto__", "String"),
        ]));
        assert_eq!(
            signature.parameter_bindings(),
            vec![
                "__lgl_name_64656661756C74",
                "__lgl_name_5F5F6C676C5F61726773",
                "__lgl_name_646973706C61792D6E616D65",
                "__proto__",
            ]
        );
        assert_eq!(
            signature.named_object_type(),
            "{ \"default\": string; __lgl_args: string; \"display-name\": string; [\"__proto__\"]: string }"
        );
        assert!(signature
            .implementation_rest_params()
            .starts_with("...__lgl_args:"));
    }

    #[test]
    fn overload_docs_are_preserved_on_both_signatures() {
        let signature = MessageCallSignature::from_message(&message(&[("count", "Number")]));
        let declarations = signature.overload_declarations("message", &["A message".to_owned()]);
        assert_eq!(declarations.matches("/** A message */").count(), 2);
        assert!(declarations
            .contains("export declare function message(count: number | bigint | string): string;"));
        assert!(declarations.contains(
            "export declare function message(args: { count: number | bigint | string }): string;"
        ));
        let implementation =
            signature.implementation_overloads("message", &["A message".to_owned()]);
        assert_eq!(implementation.matches("/** A message */").count(), 2);
        assert!(implementation
            .contains("export function message(count: number | bigint | string): string;"));
        assert!(implementation.contains(
            "export function message(args: { count: number | bigint | string }): string;"
        ));
    }
}
