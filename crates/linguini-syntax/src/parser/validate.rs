use std::collections::BTreeMap;

use crate::{
    Annotation, EnumDeclaration, Expression, ExpressionKind, FormEntry, FunctionBranch,
    FunctionBranchValue, FunctionDeclaration, InlineFunctionInput, LocaleDeclaration, LocaleFile,
    LocaleValue, MapBranch, MessageGroup, MessageImplementationGroup, MessageSignature, Name,
    SchemaDeclaration, SchemaFile, Span, TextPart, TextPattern,
};

use super::ParseError;

const RESERVED_NAMES: &[&str] = &[
    "enum", "fn", "form", "impl", "let", "override", "raw", "type",
];

pub(super) fn validate_schema(file: &SchemaFile) -> Vec<ParseError> {
    let mut validator = Validator::default();

    for declaration in &file.declarations {
        match declaration {
            SchemaDeclaration::Enum(item) => {
                validator.enum_declaration(item);
            }
            SchemaDeclaration::TypeAlias(item) => {
                validator.pascal_name(&item.name, "type alias");
                validator.pascal_name(&item.target, "type");
                validator.annotations(&item.annotations);
            }
            SchemaDeclaration::Message(item) => {
                validator.message_signature(item);
            }
            SchemaDeclaration::Group(item) => {
                validator.schema_group(item);
            }
        }
    }

    validator.errors
}

pub(super) fn validate_locale(file: &LocaleFile) -> Vec<ParseError> {
    let mut validator = Validator::default();

    for declaration in &file.declarations {
        validator.locale_declaration(declaration);
    }

    validator.errors
}

#[derive(Default)]
struct Validator {
    errors: Vec<ParseError>,
}

impl Validator {
    fn locale_declaration(&mut self, declaration: &LocaleDeclaration) {
        match declaration {
            LocaleDeclaration::Enum(item) => {
                self.enum_declaration(item);
            }
            LocaleDeclaration::Variable(item) => {
                self.lower_name(&item.name, "variable");
                self.text_pattern(&item.value);
            }
            LocaleDeclaration::Form(item) => {
                self.pascal_name(&item.name, "form type");
                let mut variants = BTreeMap::new();
                for variant in &item.variants {
                    self.lower_name(&variant.name, "form variant");
                    self.unique_name(&mut variants, &variant.name, "form variant");
                    let mut entries = BTreeMap::new();
                    for entry in &variant.entries {
                        self.form_entry(entry, &mut entries);
                    }
                }
            }
            LocaleDeclaration::Function(item) => {
                self.function_name(&item.name);
                self.function(item);
            }
            LocaleDeclaration::Message(item) => {
                self.lower_name(&item.name, "message");
                self.text_pattern(&item.value);
            }
            LocaleDeclaration::Group(item) => {
                self.locale_group(item);
            }
            LocaleDeclaration::Override(inner) => {
                self.locale_declaration(inner);
            }
        }
    }

    fn enum_declaration(&mut self, declaration: &EnumDeclaration) {
        self.pascal_name(&declaration.name, "enum");
        if declaration.variants.is_empty() {
            self.error("enum must contain at least one variant", declaration.span);
        }
        let mut variants = BTreeMap::new();
        for variant in &declaration.variants {
            self.lower_name(variant, "enum variant");
            self.unique_name(&mut variants, variant, "enum variant");
        }
    }

    fn message_signature(&mut self, message: &MessageSignature) {
        self.lower_name(&message.name, "message");
        let mut parameters = BTreeMap::new();
        for parameter in &message.parameters {
            self.lower_name(&parameter.name, "message parameter");
            self.pascal_name(&parameter.ty, "parameter type");
            self.unique_name(&mut parameters, &parameter.name, "message parameter");
        }
    }

    fn schema_group(&mut self, group: &MessageGroup) {
        self.lower_name(&group.name, "group");
        let mut members = BTreeMap::new();
        for message in &group.messages {
            self.unique_name(&mut members, &message.name, "group member");
            self.message_signature(message);
        }
        for child in &group.groups {
            self.unique_name(&mut members, &child.name, "group member");
            self.schema_group(child);
        }
    }

    fn locale_group(&mut self, group: &MessageImplementationGroup) {
        self.lower_name(&group.name, "group");
        if group.messages.is_empty() && group.groups.is_empty() {
            self.error("locale group must not be empty", group.span);
        }
        let mut members = BTreeMap::new();
        for message in &group.messages {
            self.unique_name(&mut members, &message.name, "group member");
            self.lower_name(&message.name, "message");
            self.text_pattern(&message.value);
        }
        for child in &group.groups {
            self.unique_name(&mut members, &child.name, "group member");
            self.locale_group(child);
        }
    }

    fn function(&mut self, function: &FunctionDeclaration) {
        self.function_parameters(&function.parameters);
        self.function_branches(&function.branches);
    }

    fn function_parameters(&mut self, function_parameters: &[crate::FunctionParameter]) {
        let mut parameters = BTreeMap::new();
        let mut saw_named = false;
        for parameter in function_parameters {
            self.pascal_name(&parameter.ty, "parameter type");
            let identity = parameter.name.as_ref().unwrap_or(&parameter.ty);
            if parameter.name.is_some() {
                self.lower_name(identity, "function parameter");
                saw_named = true;
            } else if saw_named {
                self.error(
                    format!(
                        "unnamed dispatch parameter `{}` must precede named payload parameters",
                        parameter.ty.value
                    ),
                    parameter.span,
                );
            }
            self.unique_name(&mut parameters, identity, "function parameter");
        }
    }

    fn inline_function_inputs(&mut self, inputs: &[InlineFunctionInput]) {
        let mut bindings = BTreeMap::new();
        let mut saw_binding = false;
        for input in inputs {
            match input {
                InlineFunctionInput::Selector { value, span } => {
                    if saw_binding {
                        self.error(
                            "inline fn selector inputs must precede named bindings",
                            *span,
                        );
                    }
                    self.expression(value);
                }
                InlineFunctionInput::Binding { name, value, .. } => {
                    saw_binding = true;
                    self.lower_name(name, "inline fn binding");
                    self.unique_name(&mut bindings, name, "inline fn binding");
                    self.expression(value);
                }
            }
        }
    }

    fn function_branches(&mut self, branches: &[FunctionBranch]) {
        let mut keys = BTreeMap::new();
        for branch in branches {
            self.branch_key(&branch.key);
            self.unique_name(&mut keys, &branch.key, "branch");
            match &branch.value {
                FunctionBranchValue::Text(pattern) => self.text_pattern(pattern),
                FunctionBranchValue::Dispatch(children) => self.function_branches(children),
            }
        }
    }

    fn form_entry(
        &mut self,
        entry: &FormEntry,
        entries: &mut BTreeMap<String, (&'static str, Span)>,
    ) {
        match entry {
            FormEntry::Attribute(attribute) => {
                self.unique_name(entries, &attribute.name, "form entry");
                self.function_parameters(&attribute.parameters);
                if starts_uppercase(&attribute.name.value) {
                    self.pascal_name(&attribute.name, "form category");
                } else {
                    self.lower_name(&attribute.name, "form attribute");
                }
                if !attribute.parameters.is_empty()
                    && !matches!(attribute.value, LocaleValue::Map(_))
                {
                    self.error(
                        "form attribute parameters require a branch map",
                        attribute.span,
                    );
                }
                if attribute.parameters.len() > 1 && matches!(attribute.value, LocaleValue::Map(_))
                {
                    self.error(
                        "a flat form branch map supports one dispatch parameter",
                        attribute.span,
                    );
                }
                match &attribute.value {
                    LocaleValue::Text(pattern) => self.text_pattern(pattern),
                    LocaleValue::Map(branches) => self.map_branches(branches),
                    LocaleValue::Object(children) => {
                        let mut nested = BTreeMap::new();
                        for child in children {
                            self.form_entry(child, &mut nested);
                        }
                    }
                }
            }
            FormEntry::Branch(branch) => {
                for key in &branch.keys {
                    self.branch_key(key);
                }
                self.text_pattern(&branch.value);
            }
        }
    }

    fn map_branches(&mut self, branches: &[MapBranch]) {
        for branch in branches {
            for key in &branch.keys {
                self.branch_key(key);
            }
            self.text_pattern(&branch.value);
        }
    }

    fn text_pattern(&mut self, pattern: &TextPattern) {
        for part in &pattern.parts {
            if let TextPart::Placeholder(placeholder) = part {
                self.expression(&placeholder.expression);
            }
        }
    }

    fn expression(&mut self, expression: &Expression) {
        match &expression.kind {
            ExpressionKind::InlineFunction { inputs, branches } => {
                if !expression.path.is_empty() {
                    self.error(
                        "inline fn expression cannot contain a path",
                        expression.span,
                    );
                }
                if !expression.arguments.is_empty() {
                    self.error(
                        "inline fn expression cannot contain call arguments",
                        expression.span,
                    );
                }
                if branches.is_empty() {
                    self.error(
                        "inline fn expression requires at least one branch",
                        expression.span,
                    );
                }
                self.inline_function_inputs(inputs);
                self.function_branches(branches);
            }
            ExpressionKind::Reference | ExpressionKind::Call => {
                if expression.path.is_empty() {
                    self.error("expression path must not be empty", expression.span);
                } else {
                    for name in &expression.path {
                        self.not_reserved(name, "expression segment");
                    }
                }
                if expression.path.len() > 2 {
                    self.error(
                        "expression paths support at most one property segment",
                        expression.span,
                    );
                }
            }
        }
        match &expression.kind {
            ExpressionKind::Reference if !expression.arguments.is_empty() => self.error(
                "reference expression cannot contain call arguments",
                expression.span,
            ),
            ExpressionKind::Reference
            | ExpressionKind::Call
            | ExpressionKind::InlineFunction { .. } => {}
        }
        for argument in &expression.arguments {
            self.expression(argument);
        }
        self.annotations(&expression.annotations);
    }

    fn annotations(&mut self, annotations: &[Annotation]) {
        for annotation in annotations {
            let mut arguments = BTreeMap::new();
            for argument in &annotation.arguments {
                self.lower_name(&argument.name, "annotation argument");
                self.unique_name(&mut arguments, &argument.name, "annotation argument");
            }
        }
    }

    fn unique_name(
        &mut self,
        names: &mut BTreeMap<String, (&'static str, Span)>,
        name: &Name,
        kind: &'static str,
    ) {
        self.unique(names, name, kind, kind);
    }

    fn unique(
        &mut self,
        names: &mut BTreeMap<String, (&'static str, Span)>,
        name: &Name,
        kind: &'static str,
        description: &'static str,
    ) {
        if let Some((previous_kind, _)) = names.get(&name.value) {
            self.error(
                format!(
                    "duplicate {description} `{}`; name already used by {previous_kind}",
                    name.value
                ),
                name.span,
            );
        } else {
            names.insert(name.value.clone(), (kind, name.span));
        }
    }

    fn branch_key(&mut self, name: &Name) {
        if name.value != "_" {
            self.lower_name(name, "branch");
        }
    }

    fn function_name(&mut self, name: &Name) {
        self.not_reserved(name, "function");
        if !is_lower_name(&name.value) && !is_pascal_name(&name.value) {
            self.error(
                format!(
                    "function `{}` must use lowercase_snake_case or PascalCase",
                    name.value
                ),
                name.span,
            );
        }
    }

    fn lower_name(&mut self, name: &Name, kind: &str) {
        self.not_reserved(name, kind);
        if !is_lower_name(&name.value) {
            self.error(
                format!("{kind} `{}` must use lowercase_snake_case", name.value),
                name.span,
            );
        }
    }

    fn pascal_name(&mut self, name: &Name, kind: &str) {
        self.not_reserved(name, kind);
        if !is_pascal_name(&name.value) {
            self.error(
                format!("{kind} `{}` must use PascalCase", name.value),
                name.span,
            );
        }
    }

    fn not_reserved(&mut self, name: &Name, kind: &str) {
        if RESERVED_NAMES.contains(&name.value.as_str()) {
            self.error(
                format!("{kind} name `{}` is reserved", name.value),
                name.span,
            );
        }
    }

    fn error(&mut self, message: impl Into<String>, span: Span) {
        self.errors.push(ParseError {
            message: message.into(),
            span,
        });
    }
}

fn starts_uppercase(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_uppercase)
}

fn is_lower_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || (first.is_alphabetic() && !first.is_uppercase()))
        && chars.all(|character| {
            character == '_'
                || (character.is_alphabetic() && !character.is_uppercase())
                || character.is_numeric()
        })
}

fn is_pascal_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_uppercase() && chars.all(|character| character.is_alphanumeric())
}
