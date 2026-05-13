use crate::model::{
    IrBranch, IrBranchPattern, IrExpression, IrForm, IrFormEntry, IrFormVariant, IrFormatter,
    IrFormatterArgument, IrFunction, IrFunctionBranch, IrMessage, IrModule, IrParameter, IrText,
    IrTextPart, IrValue,
};
use linguini_syntax::{
    Annotation, BranchPattern, DocComment, Expression, FormEntry, LocaleDeclaration, LocaleFile,
    LocaleValue, MapBranch, SchemaDeclaration, SchemaFile, TextPart, TextPattern,
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
            SchemaDeclaration::Enum(_) | SchemaDeclaration::TypeAlias(_) => {}
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
                    selector: variant.selector.as_ref().map(|name| name.value.clone()),
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
                .map(|name| name.value.clone())
                .collect(),
            branches: function
                .branches
                .iter()
                .map(|branch| IrFunctionBranch {
                    pattern: match &branch.pattern {
                        BranchPattern::Names(names) => IrBranchPattern::Names(
                            names.iter().map(|name| name.value.clone()).collect(),
                        ),
                        BranchPattern::Else(_) => IrBranchPattern::Else,
                    },
                    value: lower_text(&branch.value),
                })
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
        name: annotation.name.value.clone(),
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
