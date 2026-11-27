//! Builds expression-analysis inputs without treating inherited source bodies as current diagnostics.

use super::{
    collect_form_entry_expression_inputs, collect_form_properties,
    collect_function_branch_expression_inputs, named_function_parameters, qualified_name,
    resolve_schema_type, ExpressionAnalysis, FormProperty, FormSignature, FunctionSignature,
    MessageToAnalyze, Variable,
};
use linguini_syntax::{FormEntry, LocaleDeclaration};
use std::collections::{BTreeMap, BTreeSet};

impl ExpressionAnalysis {
    pub(super) fn collect_declaration(
        &mut self,
        declaration: &LocaleDeclaration,
        namespace: Option<&str>,
        schema_messages: &BTreeMap<String, &linguini_syntax::MessageSignature>,
        enum_names: &BTreeSet<String>,
        type_aliases: &BTreeMap<&str, &str>,
        include_bodies: bool,
    ) {
        match declaration {
            LocaleDeclaration::Variable(variable) => {
                self.variables.push(Variable::new(
                    &variable.name.value,
                    "String",
                    variable.name.span,
                ));
                if include_bodies {
                    self.messages.push(MessageToAnalyze::new(
                        format!("variable `{}`", variable.name.value),
                        variable.value.clone(),
                        Vec::new(),
                    ));
                }
            }
            LocaleDeclaration::Function(function) => {
                self.functions.push(FunctionSignature::typed(
                    &function.name.value,
                    function
                        .parameters
                        .iter()
                        .map(|parameter| resolve_schema_type(&parameter.ty.value, type_aliases)),
                    function.span,
                ));
                if include_bodies {
                    let parameters = named_function_parameters(&function.parameters, type_aliases);
                    collect_function_branch_expression_inputs(
                        &format!("function `{}`", function.name.value),
                        &function.branches,
                        &parameters,
                        &mut self.messages,
                    );
                }
            }
            LocaleDeclaration::Form(form) => {
                let mut properties = BTreeMap::new();
                if !form.variants.is_empty()
                    && form.variants.iter().all(|variant| {
                        variant
                            .entries
                            .iter()
                            .any(|entry| matches!(entry, FormEntry::Branch(_)))
                    })
                {
                    properties.insert(String::new(), FormProperty::callable(form.span));
                }
                for variant in &form.variants {
                    collect_form_properties(
                        &variant.entries,
                        enum_names,
                        type_aliases,
                        &mut properties,
                    );
                    if include_bodies {
                        collect_form_entry_expression_inputs(
                            &format!(
                                "form `{}` variant `{}`",
                                form.name.value, variant.name.value
                            ),
                            &variant.entries,
                            &[],
                            type_aliases,
                            &mut self.messages,
                        );
                    }
                }
                self.forms.push(FormSignature::new(
                    resolve_schema_type(&form.name.value, type_aliases),
                    properties.into_values().collect(),
                    form.span,
                ));
            }
            LocaleDeclaration::Message(message) if include_bodies => {
                let name = qualified_name(namespace, &message.name.value);
                let parameters = schema_messages
                    .get(&name)
                    .map(|signature| {
                        signature
                            .parameters
                            .iter()
                            .map(|parameter| {
                                Variable::new(
                                    &parameter.name.value,
                                    resolve_schema_type(&parameter.ty.value, type_aliases),
                                    parameter.span,
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                self.messages.push(MessageToAnalyze::new(
                    name,
                    message.value.clone(),
                    parameters,
                ));
            }
            LocaleDeclaration::Group(group) if include_bodies => {
                let group_name = qualified_name(namespace, &group.name.value);
                for message in &group.messages {
                    let declaration = LocaleDeclaration::Message(message.clone());
                    self.collect_declaration(
                        &declaration,
                        Some(&group_name),
                        schema_messages,
                        enum_names,
                        type_aliases,
                        include_bodies,
                    );
                }
                for child in &group.groups {
                    let declaration = LocaleDeclaration::Group(child.clone());
                    self.collect_declaration(
                        &declaration,
                        Some(&group_name),
                        schema_messages,
                        enum_names,
                        type_aliases,
                        include_bodies,
                    );
                }
            }
            LocaleDeclaration::Override(inner) => self.collect_declaration(
                inner,
                namespace,
                schema_messages,
                enum_names,
                type_aliases,
                include_bodies,
            ),
            LocaleDeclaration::Enum(_)
            | LocaleDeclaration::Message(_)
            | LocaleDeclaration::Group(_) => {}
        }
    }
}
