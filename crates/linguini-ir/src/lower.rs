use crate::model::{
    IrBranch, IrEnum, IrExpression, IrExpressionKind, IrForm, IrFormEntry, IrFormVariant,
    IrFormatter, IrFormatterArgument, IrFunction, IrFunctionBranch, IrFunctionBranchValue,
    IrFunctionKind, IrFunctionParameter, IrMessage, IrModule, IrOrigin, IrParameter, IrSymbolKind,
    IrText, IrTextBlockMode, IrTextPart, IrTypeAlias, IrValue, IrVariable, LocaleIr, SchemaIr,
};
use linguini_syntax::{
    Annotation, DocComment, Expression, ExpressionKind, FormEntry, FunctionBranchValue,
    FunctionKind, LocaleDeclaration, LocaleFile, LocaleValue, MapBranch, MessageGroup,
    MessageImplementationGroup, MessageSignature, SchemaDeclaration, SchemaFile, TextBlockMode,
    TextPart, TextPattern,
};

pub fn lower_schema(schema: &SchemaFile) -> IrModule {
    let mut module = IrModule::default();
    for declaration in &schema.declarations {
        lower_schema_declaration(declaration, None, &mut module);
    }
    module
}

pub fn lower_schema_typed(schema: &SchemaFile) -> SchemaIr {
    SchemaIr(lower_schema(schema))
}

fn lower_schema_declaration(
    declaration: &SchemaDeclaration,
    namespace: Option<&str>,
    module: &mut IrModule,
) {
    match declaration {
        SchemaDeclaration::Message(message) => {
            lower_schema_message(message, namespace, module);
        }
        SchemaDeclaration::Group(group) => lower_schema_group(group, namespace, module),
        SchemaDeclaration::Enum(declaration) => {
            module.origins.push(IrOrigin {
                kind: IrSymbolKind::Enum,
                name: declaration.name.value.clone(),
                span: declaration.span,
                is_override: false,
            });
            module.enums.push(IrEnum {
                name: declaration.name.value.clone(),
                docs: docs(&declaration.docs),
                variants: declaration
                    .variants
                    .iter()
                    .map(|variant| variant.value.clone())
                    .collect(),
            });
        }
        SchemaDeclaration::TypeAlias(declaration) => {
            module.origins.push(IrOrigin {
                kind: IrSymbolKind::TypeAlias,
                name: declaration.name.value.clone(),
                span: declaration.span,
                is_override: false,
            });
            module.type_aliases.push(IrTypeAlias {
                name: declaration.name.value.clone(),
                target: declaration.target.value.clone(),
                docs: docs(&declaration.docs),
                formatters: declaration
                    .annotations
                    .iter()
                    .map(lower_formatter)
                    .collect(),
            });
        }
    }
}

fn lower_schema_message(
    message: &MessageSignature,
    namespace: Option<&str>,
    module: &mut IrModule,
) {
    let name = qualified_name(namespace, &message.name.value);
    module.origins.push(IrOrigin {
        kind: IrSymbolKind::Message,
        name: name.clone(),
        span: message.span,
        is_override: false,
    });
    module.messages.push(IrMessage {
        name,
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

fn lower_schema_group(group: &MessageGroup, namespace: Option<&str>, module: &mut IrModule) {
    let name = qualified_name(namespace, &group.name.value);
    module.origins.push(IrOrigin {
        kind: IrSymbolKind::Group,
        name: name.clone(),
        span: group.span,
        is_override: false,
    });
    for message in &group.messages {
        lower_schema_message(message, Some(&name), module);
    }
    for child in &group.groups {
        lower_schema_group(child, Some(&name), module);
    }
}

pub fn lower_locale(locale: &LocaleFile) -> IrModule {
    let mut module = IrModule::default();
    for declaration in &locale.declarations {
        lower_locale_declaration(declaration, None, false, &mut module);
    }
    module
}

pub fn lower_locale_typed(locale: &LocaleFile) -> LocaleIr {
    LocaleIr(lower_locale(locale))
}

fn lower_locale_declaration(
    declaration: &LocaleDeclaration,
    namespace: Option<&str>,
    is_override: bool,
    module: &mut IrModule,
) {
    match declaration {
        LocaleDeclaration::Form(form) => {
            let name = qualified_name(namespace, &form.name.value);
            module.origins.push(IrOrigin {
                kind: IrSymbolKind::Form,
                name: name.clone(),
                span: form.span,
                is_override,
            });
            module.forms.push(IrForm {
                name,
                docs: docs(&form.docs),
                variants: form
                    .variants
                    .iter()
                    .map(|variant| IrFormVariant {
                        name: variant.name.value.clone(),
                        entries: variant.entries.iter().map(lower_form_entry).collect(),
                    })
                    .collect(),
            });
        }
        LocaleDeclaration::Function(function) => {
            let name = qualified_name(namespace, &function.name.value);
            let kind = match function.kind {
                FunctionKind::Form => IrFunctionKind::Form,
                FunctionKind::Function => IrFunctionKind::Function,
            };
            module.origins.push(IrOrigin {
                kind: match kind {
                    IrFunctionKind::Form => IrSymbolKind::Form,
                    IrFunctionKind::Function => IrSymbolKind::Function,
                },
                name: name.clone(),
                span: function.span,
                is_override,
            });
            module.functions.push(IrFunction {
                kind,
                name,
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
            });
        }
        LocaleDeclaration::Variable(variable) => {
            let name = qualified_name(namespace, &variable.name.value);
            module.origins.push(IrOrigin {
                kind: IrSymbolKind::Variable,
                name: name.clone(),
                span: variable.span,
                is_override,
            });
            module.variables.push(IrVariable {
                name,
                docs: docs(&variable.docs),
                value: lower_text(&variable.value),
            });
        }
        LocaleDeclaration::Message(message) => {
            lower_locale_message(message, namespace, is_override, module);
        }
        LocaleDeclaration::Group(group) => {
            lower_locale_group(group, namespace, is_override, module);
        }
        LocaleDeclaration::Override(inner) => {
            remove_overridden_declaration(inner, namespace, module);
            lower_locale_declaration(inner, namespace, true, module);
        }
        LocaleDeclaration::Enum(declaration) => {
            let name = qualified_name(namespace, &declaration.name.value);
            module.origins.push(IrOrigin {
                kind: IrSymbolKind::Enum,
                name: name.clone(),
                span: declaration.span,
                is_override,
            });
            module.enums.push(IrEnum {
                name,
                docs: docs(&declaration.docs),
                variants: declaration
                    .variants
                    .iter()
                    .map(|variant| variant.value.clone())
                    .collect(),
            });
        }
    }
}

fn remove_overridden_declaration(
    declaration: &LocaleDeclaration,
    namespace: Option<&str>,
    module: &mut IrModule,
) {
    let (name, namespace_prefix) = match declaration {
        LocaleDeclaration::Enum(item) => (qualified_name(namespace, &item.name.value), false),
        LocaleDeclaration::Variable(item) => (qualified_name(namespace, &item.name.value), false),
        LocaleDeclaration::Form(item) => (qualified_name(namespace, &item.name.value), false),
        LocaleDeclaration::Function(item) => (qualified_name(namespace, &item.name.value), false),
        LocaleDeclaration::Message(item) => (qualified_name(namespace, &item.name.value), false),
        LocaleDeclaration::Group(item) => (qualified_name(namespace, &item.name.value), true),
        LocaleDeclaration::Override(inner) => {
            remove_overridden_declaration(inner, namespace, module);
            return;
        }
    };

    module.enums.retain(|item| item.name != name);
    module.variables.retain(|item| item.name != name);
    module.forms.retain(|item| item.name != name);
    module.functions.retain(|item| item.name != name);
    module.messages.retain(|item| {
        item.name != name
            && (!namespace_prefix
                || !item
                    .name
                    .strip_prefix(&name)
                    .is_some_and(|suffix| suffix.starts_with('.')))
    });
}

fn lower_locale_message(
    message: &linguini_syntax::MessageImplementation,
    namespace: Option<&str>,
    is_override: bool,
    module: &mut IrModule,
) {
    let name = qualified_name(namespace, &message.name.value);
    module.origins.push(IrOrigin {
        kind: IrSymbolKind::Message,
        name: name.clone(),
        span: message.span,
        is_override,
    });
    module.messages.push(IrMessage {
        name,
        docs: docs(&message.docs),
        parameters: vec![],
        body: Some(lower_text(&message.value)),
    });
}

fn lower_locale_group(
    group: &MessageImplementationGroup,
    namespace: Option<&str>,
    is_override: bool,
    module: &mut IrModule,
) {
    let name = qualified_name(namespace, &group.name.value);
    module.origins.push(IrOrigin {
        kind: IrSymbolKind::Group,
        name: name.clone(),
        span: group.span,
        is_override,
    });
    for message in &group.messages {
        lower_locale_message(message, Some(&name), is_override, module);
    }
    for child in &group.groups {
        lower_locale_group(child, Some(&name), is_override, module);
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
        span: branch.span,
    }
}

fn lower_form_entry(entry: &FormEntry) -> IrFormEntry {
    match entry {
        FormEntry::Attribute(attribute) => IrFormEntry::Attribute {
            name: attribute.name.value.clone(),
            parameters: attribute
                .parameters
                .iter()
                .map(|parameter| IrFunctionParameter {
                    name: parameter.name.as_ref().map(|name| name.value.clone()),
                    ty: parameter.ty.value.clone(),
                })
                .collect(),
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
        span: branch.span,
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
        mode: match text.mode {
            TextBlockMode::Inline => IrTextBlockMode::Inline,
            TextBlockMode::Dedented => IrTextBlockMode::Dedented,
            TextBlockMode::Raw => IrTextBlockMode::Raw,
        },
        span: text.span,
    }
}

fn lower_expression(expression: &Expression) -> IrExpression {
    IrExpression {
        kind: match expression.kind {
            ExpressionKind::Reference => IrExpressionKind::Reference,
            ExpressionKind::Call => IrExpressionKind::Call,
        },
        path: expression
            .path
            .iter()
            .map(|name| name.value.clone())
            .collect(),
        arguments: expression.arguments.iter().map(lower_expression).collect(),
        formatters: expression.annotations.iter().map(lower_formatter).collect(),
        span: expression.span,
    }
}

fn lower_formatter(annotation: &Annotation) -> IrFormatter {
    IrFormatter {
        kind: annotation.kind.clone(),
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
    docs.iter().map(|doc| doc.text.trim().to_owned()).collect()
}

fn qualified_name(namespace: Option<&str>, name: &str) -> String {
    match namespace {
        Some(namespace) => format!("{namespace}.{name}"),
        None => name.to_owned(),
    }
}
