use crate::model::{
    IrBranch, IrEnum, IrExpression, IrForm, IrFormEntry, IrFormVariant, IrFormatter,
    IrFormatterArgument, IrFunction, IrFunctionBranch, IrFunctionBranchValue, IrFunctionParameter,
    IrMessage, IrModule, IrParameter, IrText, IrTextPart, IrTypeAlias, IrValue,
};
use linguini_syntax::{
    Annotation, DocComment, Expression, FormEntry, FunctionBranchValue, LocaleDeclaration,
    LocaleFile, LocaleValue, MapBranch, SchemaDeclaration, SchemaFile, TextPart, TextPattern,
};

pub fn lower_schema(schema: &SchemaFile) -> IrModule {
    let mut module = IrModule::default();
    for declaration in &schema.declarations {
        match declaration {
            SchemaDeclaration::Message(message) => {
                module.messages.push(IrMessage {
                    name: message.name.value.clone(),
                    docs: docs(&message.docs),
                    parameters: message
                        .parameters
                        .iter()
                        .map(|parameter| IrParameter {
                            name: parameter.name.value.clone(),
                            ty: parameter.ty.value.clone(),
                        })
                        .collect(),
                    body: None,
                });
            }
            SchemaDeclaration::Group(group) => {
                for message in &group.messages {
                    module.messages.push(IrMessage {
                        name: format!("{}.{}", group.name.value, message.name.value),
                        docs: docs(&message.docs),
                        parameters: message
                            .parameters
                            .iter()
                            .map(|parameter| IrParameter {
                                name: parameter.name.value.clone(),
                                ty: parameter.ty.value.clone(),
                            })
                            .collect(),
                        body: None,
                    });
                }
            }
            SchemaDeclaration::Enum(declaration) => module.enums.push(IrEnum {
                name: declaration.name.value.clone(),
                docs: docs(&declaration.docs),
                variants: declaration
                    .variants
                    .iter()
                    .map(|variant| variant.value.clone())
                    .collect(),
            }),
            SchemaDeclaration::TypeAlias(declaration) => module.type_aliases.push(IrTypeAlias {
                name: declaration.name.value.clone(),
                target: declaration.target.value.clone(),
                docs: docs(&declaration.docs),
                formatters: declaration
                    .annotations
                    .iter()
                    .map(lower_formatter)
                    .collect(),
            }),
        }
    }
    module
}

pub fn lower_locale(locale: &LocaleFile) -> IrModule {
    let mut module = IrModule::default();
    for declaration in &locale.declarations {
        lower_locale_declaration(declaration, &mut module);
    }
    module
}

fn lower_locale_declaration(declaration: &LocaleDeclaration, module: &mut IrModule) {
    match declaration {
        LocaleDeclaration::Form(form) => module.forms.push(IrForm {
            name: form.name.value.clone(),
            docs: docs(&form.docs),
            variants: form
                .variants
                .iter()
                .map(|variant| IrFormVariant {
                    name: variant.name.value.clone(),
                    entries: variant.entries.iter().map(lower_form_entry).collect(),
                })
                .collect(),
        }),
        LocaleDeclaration::Function(function) => module.functions.push(IrFunction {
            name: function.name.value.clone(),
            docs: docs(&function.docs),
            parameters: function
                .parameters
                .iter()
                .map(|parameter| IrFunctionParameter {
                    name: parameter.name.as_ref().map(|name| name.value.clone()),
                    ty: parameter.ty.value.clone(),
                })
                .collect(),
            branches: function
                .branches
                .iter()
                .map(lower_function_branch)
                .collect(),
        }),
        LocaleDeclaration::Message(message) => module.messages.push(IrMessage {
            name: message.name.value.clone(),
            docs: docs(&message.docs),
            parameters: vec![],
            body: Some(lower_text(&message.value)),
        }),
        LocaleDeclaration::Group(group) => lower_group(group, module),
        LocaleDeclaration::Override(inner) => lower_locale_declaration(inner, module),
        LocaleDeclaration::Enum(_) => {}
    }
}

fn lower_function_branch(branch: &linguini_syntax::FunctionBranch) -> IrFunctionBranch {
    IrFunctionBranch {
        key: branch.key.value.clone(),
        value: match &branch.value {
            FunctionBranchValue::Text(text) => IrFunctionBranchValue::Text(lower_text(text)),
            FunctionBranchValue::Dispatch(branches) => IrFunctionBranchValue::Dispatch(
                branches.iter().map(lower_function_branch).collect(),
            ),
        },
    }
}

fn lower_group(group: &linguini_syntax::MessageImplementationGroup, module: &mut IrModule) {
    for message in &group.messages {
        module.messages.push(IrMessage {
            name: format!("{}.{}", group.name.value, message.name.value),
            docs: docs(&message.docs),
            parameters: vec![],
            body: Some(lower_text(&message.value)),
        });
    }
}

fn lower_form_entry(entry: &FormEntry) -> IrFormEntry {
    match entry {
        FormEntry::Attribute(attribute) => IrFormEntry::Attribute {
            name: attribute.name.value.clone(),
            value: lower_value(&attribute.value),
        },
        FormEntry::Branch(branch) => IrFormEntry::Branch(lower_branch(branch)),
    }
}

fn lower_value(value: &LocaleValue) -> IrValue {
    match value {
        LocaleValue::Text(text) => IrValue::Text(lower_text(text)),
        LocaleValue::Map(branches) => IrValue::Map(branches.iter().map(lower_branch).collect()),
        LocaleValue::Object(entries) => {
            IrValue::Object(entries.iter().map(lower_form_entry).collect())
        }
    }
}

fn lower_branch(branch: &MapBranch) -> IrBranch {
    IrBranch {
        keys: branch.keys.iter().map(|name| name.value.clone()).collect(),
        value: lower_text(&branch.value),
    }
}

fn lower_text(text: &TextPattern) -> IrText {
    IrText {
        parts: text
            .parts
            .iter()
            .map(|part| match part {
                TextPart::Text(raw) => IrTextPart::Text(raw.value.clone()),
                TextPart::Placeholder(placeholder) => {
                    IrTextPart::Placeholder(lower_expression(&placeholder.expression))
                }
            })
            .collect(),
    }
}

fn lower_expression(expression: &Expression) -> IrExpression {
    IrExpression {
        path: expression
            .path
            .iter()
            .map(|name| name.value.clone())
            .collect(),
        arguments: expression.arguments.iter().map(lower_expression).collect(),
        formatters: expression.annotations.iter().map(lower_formatter).collect(),
    }
}

fn lower_formatter(annotation: &Annotation) -> IrFormatter {
    IrFormatter {
        kind: annotation.kind,
        arguments: annotation
            .arguments
            .iter()
            .map(|argument| IrFormatterArgument {
                name: argument.name.value.clone(),
                value: argument.value.value.clone(),
            })
            .collect(),
    }
}

fn docs(docs: &[DocComment]) -> Vec<String> {
    docs.iter().map(|doc| doc.text.clone()).collect()
}
