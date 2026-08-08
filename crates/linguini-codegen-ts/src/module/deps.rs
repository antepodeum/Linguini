//! Semantic dependency closure for a single schema message.
//!
//! This module deliberately stays crate-private.  The closure is the common input for the
//! single-message compiler and the HMR dependency registry, while the public compiler API is
//! added once those consumers have a stable contract.

use std::collections::{BTreeMap, BTreeSet};

use linguini_ir::{
    IrBranch, IrExpression, IrExpressionKind, IrForm, IrFormEntry, IrFunction, IrFunctionBranch,
    IrFunctionBranchValue, IrInlineFunctionInput, IrModule, IrSymbolKind, IrText, IrTextPart,
    IrValue,
};
use linguini_syntax::SourceId;

use super::{TypeScriptCodegenError, ValidatedTypeScriptProject};

/// Stable symbol identity used by future source-to-message invalidation consumers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MessageDependencySymbol {
    SchemaEnum(String),
    SchemaTypeAlias(String),
    SchemaMessage(String),
    SchemaGroup(String),
    LocaleEnum(String),
    LocaleMessage(String),
    LocaleVariable(String),
    LocaleForm(String),
    LocaleFunction(String),
}

/// Deterministic dependency metadata for one compiled message.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MessageDependencyMetadata {
    pub(crate) symbols: Vec<MessageDependencySymbol>,
    pub(crate) source_ids: Vec<SourceId>,
}

/// Minimal validated IR required to compile one message for one fallback-composed locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MessageDependencyClosure {
    pub(crate) locale: String,
    pub(crate) message: String,
    pub(crate) schema: IrModule,
    pub(crate) locale_module: IrModule,
    pub(crate) metadata: MessageDependencyMetadata,
}

impl MessageDependencyClosure {
    pub(crate) fn schema(&self) -> &IrModule {
        &self.schema
    }

    pub(crate) fn locale_module(&self) -> &IrModule {
        &self.locale_module
    }

    pub(crate) fn symbols(&self) -> &[MessageDependencySymbol] {
        &self.metadata.symbols
    }

    pub(crate) fn source_ids(&self) -> &[SourceId] {
        &self.metadata.source_ids
    }
}

/// Computes a closure only from [`ValidatedTypeScriptProject`], preserving the IR validation
/// boundary for production callers.
pub(crate) fn message_dependency_closure(
    project: &ValidatedTypeScriptProject<'_>,
    locale: &str,
    message: &str,
) -> Result<MessageDependencyClosure, TypeScriptCodegenError> {
    let locale_entry = project
        .locales
        .iter()
        .find(|entry| entry.locale == locale)
        .ok_or_else(|| TypeScriptCodegenError::UnknownLocale {
            locale: locale.to_owned(),
        })?;
    let schema_message = project
        .schema
        .messages
        .iter()
        .find(|candidate| candidate.name == message)
        .ok_or_else(|| TypeScriptCodegenError::UnknownMessage {
            message: message.to_owned(),
        })?;
    let implementation = locale_entry
        .module
        .messages
        .iter()
        .find(|candidate| candidate.name == message)
        .ok_or_else(|| TypeScriptCodegenError::MissingMessageImplementation {
            locale: locale.to_owned(),
            message: message.to_owned(),
        })?;

    let mut walker = ClosureWalker::new(project.schema, &locale_entry.module);
    walker.add_schema_message(schema_message);

    walker.add_locale_message(implementation);
    if let Some(body) = implementation.body.as_ref() {
        let context = walker.message_context(schema_message);
        walker.visit_text(body, &context);
    }

    walker.finish(locale.to_owned(), message.to_owned())
}

struct ClosureWalker<'a> {
    schema: &'a IrModule,
    locale: &'a IrModule,
    aliases: BTreeMap<&'a str, &'a linguini_ir::IrTypeAlias>,
    schema_enums: BTreeMap<&'a str, &'a linguini_ir::IrEnum>,
    locale_enums: BTreeMap<&'a str, &'a linguini_ir::IrEnum>,
    variables: BTreeMap<&'a str, &'a linguini_ir::IrVariable>,
    forms: BTreeMap<&'a str, &'a IrForm>,
    functions: BTreeMap<&'a str, &'a IrFunction>,
    selected: BTreeSet<MessageDependencySymbol>,
    visiting_variables: BTreeSet<String>,
    visiting_forms: BTreeSet<String>,
    visiting_functions: BTreeSet<String>,
    visiting_types: BTreeSet<String>,
}

impl<'a> ClosureWalker<'a> {
    fn new(schema: &'a IrModule, locale: &'a IrModule) -> Self {
        Self {
            schema,
            locale,
            aliases: schema
                .type_aliases
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            schema_enums: schema
                .enums
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            locale_enums: locale
                .enums
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            variables: locale
                .variables
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            forms: locale
                .forms
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            functions: locale
                .functions
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            selected: BTreeSet::new(),
            visiting_variables: BTreeSet::new(),
            visiting_forms: BTreeSet::new(),
            visiting_functions: BTreeSet::new(),
            visiting_types: BTreeSet::new(),
        }
    }

    fn add_schema_message(&mut self, message: &'a linguini_ir::IrMessage) {
        self.selected
            .insert(MessageDependencySymbol::SchemaMessage(message.name.clone()));
        self.add_group_ancestors(&message.name);
        for parameter in &message.parameters {
            self.visit_type(&parameter.ty);
        }
    }

    fn add_locale_message(&mut self, message: &'a linguini_ir::IrMessage) {
        self.selected
            .insert(MessageDependencySymbol::LocaleMessage(message.name.clone()));
    }

    fn add_group_ancestors(&mut self, name: &str) {
        let mut parts = name.split('.').collect::<Vec<_>>();
        parts.pop();
        for end in 1..=parts.len() {
            let group = parts[..end].join(".");
            if self.schema.groups.iter().any(|item| item.name == group) {
                self.selected
                    .insert(MessageDependencySymbol::SchemaGroup(group));
            }
        }
    }

    fn message_context(&self, message: &linguini_ir::IrMessage) -> BTreeMap<String, String> {
        let mut context = self
            .variables
            .keys()
            .map(|name| ((*name).to_owned(), "String".to_owned()))
            .collect::<BTreeMap<_, _>>();
        for parameter in &message.parameters {
            context.insert(parameter.name.clone(), parameter.ty.clone());
        }
        context
    }

    fn visit_type(&mut self, ty: &str) {
        if !self.visiting_types.insert(ty.to_owned()) {
            return;
        }
        if let Some(alias) = self.aliases.get(ty) {
            self.selected
                .insert(MessageDependencySymbol::SchemaTypeAlias(alias.name.clone()));
            let target = alias.target.clone();
            self.visit_type(&target);
        } else if self.schema_enums.contains_key(ty) {
            self.selected
                .insert(MessageDependencySymbol::SchemaEnum(ty.to_owned()));
        } else if self.locale_enums.contains_key(ty) {
            self.selected
                .insert(MessageDependencySymbol::LocaleEnum(ty.to_owned()));
        }
        self.visiting_types.remove(ty);
    }

    fn visit_text(&mut self, text: &IrText, context: &BTreeMap<String, String>) {
        for part in &text.parts {
            if let IrTextPart::Placeholder(expression) = part {
                self.visit_expression(expression, context);
            }
        }
    }

    fn visit_expression(&mut self, expression: &IrExpression, context: &BTreeMap<String, String>) {
        for argument in &expression.arguments {
            self.visit_expression(argument, context);
        }
        match &expression.kind {
            IrExpressionKind::InlineFunction { inputs, branches } => {
                let mut branch_context = context.clone();
                for input in inputs {
                    match input {
                        IrInlineFunctionInput::Binding { name, value, .. } => {
                            self.visit_expression(value, &branch_context);
                            branch_context
                                .insert(name.clone(), inferred_type(value, &branch_context));
                        }
                        IrInlineFunctionInput::Selector { value, .. } => {
                            self.visit_expression(value, &branch_context);
                        }
                    }
                }
                for branch in branches {
                    self.visit_function_branch(branch, &branch_context);
                }
            }
            IrExpressionKind::Reference => self.visit_path(&expression.path, context, false),
            IrExpressionKind::Call => self.visit_path(&expression.path, context, true),
        }
    }

    fn visit_path(&mut self, path: &[String], context: &BTreeMap<String, String>, called: bool) {
        if path.is_empty() {
            return;
        }
        let full = path.join(".");
        if self.functions.contains_key(full.as_str()) {
            self.visit_function(full.as_str());
            return;
        }
        if self.forms.contains_key(full.as_str()) {
            self.visit_form(full.as_str());
            return;
        }
        if let Some(root_type) = context.get(&path[0]) {
            if path.len() > 1 || called {
                self.visit_form(root_type);
            } else if self.variables.contains_key(path[0].as_str()) {
                self.visit_variable(path[0].as_str());
            }
        }
    }

    fn visit_variable(&mut self, name: &str) {
        let Some(variable) = self.variables.get(name).copied() else {
            return;
        };
        self.selected
            .insert(MessageDependencySymbol::LocaleVariable(name.to_owned()));
        if !self.visiting_variables.insert(name.to_owned()) {
            return;
        }
        let context = self
            .variables
            .keys()
            .map(|item| ((*item).to_owned(), "String".to_owned()))
            .collect::<BTreeMap<_, _>>();
        self.visit_text(&variable.value, &context);
        self.visiting_variables.remove(name);
    }

    fn visit_function(&mut self, name: &str) {
        let Some(function) = self.functions.get(name).copied() else {
            return;
        };
        self.selected
            .insert(MessageDependencySymbol::LocaleFunction(name.to_owned()));
        if !self.visiting_functions.insert(name.to_owned()) {
            return;
        }
        let mut context = self
            .variables
            .keys()
            .map(|item| ((*item).to_owned(), "String".to_owned()))
            .collect::<BTreeMap<_, _>>();
        for parameter in &function.parameters {
            self.visit_type(&parameter.ty);
            if let Some(name) = &parameter.name {
                context.insert(name.clone(), parameter.ty.clone());
            }
        }
        for branch in &function.branches {
            self.visit_function_branch(branch, &context);
        }
        self.visiting_functions.remove(name);
    }

    fn visit_function_branch(
        &mut self,
        branch: &IrFunctionBranch,
        context: &BTreeMap<String, String>,
    ) {
        match &branch.value {
            IrFunctionBranchValue::Text(text) => self.visit_text(text, context),
            IrFunctionBranchValue::Dispatch(children) => {
                for child in children {
                    self.visit_function_branch(child, context);
                }
            }
        }
    }

    fn visit_form(&mut self, ty: &str) {
        let resolved = self.resolve_type_name(ty);
        let Some(form) = self.forms.get(resolved.as_str()).copied() else {
            return;
        };
        self.selected
            .insert(MessageDependencySymbol::LocaleForm(form.name.clone()));
        if !self.visiting_forms.insert(form.name.clone()) {
            return;
        }
        let context = self
            .variables
            .keys()
            .map(|item| ((*item).to_owned(), "String".to_owned()))
            .collect::<BTreeMap<_, _>>();
        for variant in &form.variants {
            self.visit_form_entries(&variant.entries, &context);
        }
        self.visiting_forms.remove(&form.name);
    }

    fn resolve_type_name(&self, ty: &str) -> String {
        let mut current = ty.to_owned();
        let mut visited = BTreeSet::new();
        while visited.insert(current.clone()) {
            let Some(alias) = self.aliases.get(current.as_str()) else {
                break;
            };
            current = alias.target.clone();
        }
        current
    }

    fn visit_form_entries(
        &mut self,
        entries: &[IrFormEntry],
        inherited_context: &BTreeMap<String, String>,
    ) {
        for entry in entries {
            match entry {
                IrFormEntry::Attribute {
                    parameters, value, ..
                } => {
                    let mut context = inherited_context.clone();
                    for parameter in parameters {
                        self.visit_type(&parameter.ty);
                        if let Some(name) = &parameter.name {
                            context.insert(name.clone(), parameter.ty.clone());
                        }
                    }
                    self.visit_value(value, &context);
                }
                IrFormEntry::Branch(branch) => {
                    self.visit_text(&branch.value, inherited_context);
                }
            }
        }
    }

    fn visit_value(&mut self, value: &IrValue, context: &BTreeMap<String, String>) {
        match value {
            IrValue::Text(text) => self.visit_text(text, context),
            IrValue::Map(branches) => {
                for branch in branches {
                    self.visit_text(&branch.value, context);
                }
            }
            IrValue::Object(entries) => self.visit_form_entries(entries, context),
        }
    }

    fn finish(
        self,
        locale: String,
        message: String,
    ) -> Result<MessageDependencyClosure, TypeScriptCodegenError> {
        let selected = self.selected;
        let schema = slice_schema(self.schema, &selected);
        let locale_module = slice_locale(self.locale, &selected);
        let metadata = metadata_for(self.schema, self.locale, &selected);
        Ok(MessageDependencyClosure {
            locale,
            message,
            schema,
            locale_module,
            metadata,
        })
    }
}

fn inferred_type(expression: &IrExpression, context: &BTreeMap<String, String>) -> String {
    match &expression.kind {
        IrExpressionKind::Reference if expression.path.len() == 1 => context
            .get(&expression.path[0])
            .cloned()
            .unwrap_or_else(|| "String".to_owned()),
        IrExpressionKind::Call if expression.path.len() == 1 && expression.path[0] == "plural" => {
            "Plural".to_owned()
        }
        _ => "String".to_owned(),
    }
}

fn slice_schema(module: &IrModule, selected: &BTreeSet<MessageDependencySymbol>) -> IrModule {
    let mut output = module.clone();
    output
        .enums
        .retain(|item| selected.contains(&MessageDependencySymbol::SchemaEnum(item.name.clone())));
    output.type_aliases.retain(|item| {
        selected.contains(&MessageDependencySymbol::SchemaTypeAlias(item.name.clone()))
    });
    output.messages.retain(|item| {
        selected.contains(&MessageDependencySymbol::SchemaMessage(item.name.clone()))
    });
    output
        .groups
        .retain(|item| selected.contains(&MessageDependencySymbol::SchemaGroup(item.name.clone())));
    output.variables.clear();
    output.forms.clear();
    output.functions.clear();
    output
        .origins
        .retain(|origin| selected_schema_origin(origin, selected));
    output
}

fn slice_locale(module: &IrModule, selected: &BTreeSet<MessageDependencySymbol>) -> IrModule {
    let mut output = module.clone();
    output
        .enums
        .retain(|item| selected.contains(&MessageDependencySymbol::LocaleEnum(item.name.clone())));
    output.variables.retain(|item| {
        selected.contains(&MessageDependencySymbol::LocaleVariable(item.name.clone()))
    });
    output
        .forms
        .retain(|item| selected.contains(&MessageDependencySymbol::LocaleForm(item.name.clone())));
    output.functions.retain(|item| {
        selected.contains(&MessageDependencySymbol::LocaleFunction(item.name.clone()))
    });
    output.messages.retain(|item| {
        selected.contains(&MessageDependencySymbol::LocaleMessage(item.name.clone()))
    });
    output.groups.clear();
    output.type_aliases.clear();
    output
        .origins
        .retain(|origin| selected_locale_origin(origin, selected));
    output
}

fn selected_schema_origin(
    origin: &linguini_ir::IrOrigin,
    selected: &BTreeSet<MessageDependencySymbol>,
) -> bool {
    let symbol = match origin.kind {
        IrSymbolKind::Enum => MessageDependencySymbol::SchemaEnum(origin.name.clone()),
        IrSymbolKind::TypeAlias => MessageDependencySymbol::SchemaTypeAlias(origin.name.clone()),
        IrSymbolKind::Message => MessageDependencySymbol::SchemaMessage(origin.name.clone()),
        IrSymbolKind::Group => MessageDependencySymbol::SchemaGroup(origin.name.clone()),
        IrSymbolKind::Variable | IrSymbolKind::Form | IrSymbolKind::Function => return false,
    };
    selected.contains(&symbol)
}

fn selected_locale_origin(
    origin: &linguini_ir::IrOrigin,
    selected: &BTreeSet<MessageDependencySymbol>,
) -> bool {
    let symbol = match origin.kind {
        IrSymbolKind::Enum => MessageDependencySymbol::LocaleEnum(origin.name.clone()),
        IrSymbolKind::Message => MessageDependencySymbol::LocaleMessage(origin.name.clone()),
        IrSymbolKind::Variable => MessageDependencySymbol::LocaleVariable(origin.name.clone()),
        IrSymbolKind::Form => MessageDependencySymbol::LocaleForm(origin.name.clone()),
        IrSymbolKind::Function => MessageDependencySymbol::LocaleFunction(origin.name.clone()),
        IrSymbolKind::TypeAlias | IrSymbolKind::Group => return false,
    };
    selected.contains(&symbol)
}

fn metadata_for(
    schema: &IrModule,
    locale: &IrModule,
    selected: &BTreeSet<MessageDependencySymbol>,
) -> MessageDependencyMetadata {
    let mut source_ids = BTreeSet::new();
    collect_origin_source_ids(schema, selected, selected_schema_origin, &mut source_ids);
    collect_origin_source_ids(locale, selected, selected_locale_origin, &mut source_ids);
    for message in &schema.messages {
        if selected.contains(&MessageDependencySymbol::SchemaMessage(
            message.name.clone(),
        )) {
            if let Some(body) = &message.body {
                collect_text_sources(body, &mut source_ids);
            }
        }
    }
    for message in &locale.messages {
        if selected.contains(&MessageDependencySymbol::LocaleMessage(
            message.name.clone(),
        )) {
            if let Some(body) = &message.body {
                collect_text_sources(body, &mut source_ids);
            }
        }
    }
    for variable in &locale.variables {
        if selected.contains(&MessageDependencySymbol::LocaleVariable(
            variable.name.clone(),
        )) {
            collect_text_sources(&variable.value, &mut source_ids);
        }
    }
    for function in &locale.functions {
        if selected.contains(&MessageDependencySymbol::LocaleFunction(
            function.name.clone(),
        )) {
            for branch in &function.branches {
                collect_function_branch_sources(branch, &mut source_ids);
            }
        }
    }
    for form in &locale.forms {
        if selected.contains(&MessageDependencySymbol::LocaleForm(form.name.clone())) {
            for variant in &form.variants {
                for entry in &variant.entries {
                    _collect_form_sources(entry, &mut source_ids);
                }
            }
        }
    }
    MessageDependencyMetadata {
        symbols: selected.iter().cloned().collect(),
        source_ids: source_ids.into_iter().collect(),
    }
}

fn collect_origin_source_ids(
    module: &IrModule,
    selected: &BTreeSet<MessageDependencySymbol>,
    selector: fn(&linguini_ir::IrOrigin, &BTreeSet<MessageDependencySymbol>) -> bool,
    output: &mut BTreeSet<SourceId>,
) {
    let mut latest = BTreeMap::<(&str, IrSymbolKind), usize>::new();
    for (index, origin) in module.origins.iter().enumerate() {
        latest.insert((origin.name.as_str(), origin.kind), index);
    }
    for (index, origin) in module.origins.iter().enumerate() {
        if latest.get(&(origin.name.as_str(), origin.kind)) == Some(&index)
            && selector(origin, selected)
        {
            output.insert(origin.span.source);
        }
    }
}

fn collect_text_sources(text: &IrText, output: &mut BTreeSet<SourceId>) {
    output.insert(text.span.source);
    for part in &text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            collect_expression_sources(expression, output);
        }
    }
}

fn collect_expression_sources(expression: &IrExpression, output: &mut BTreeSet<SourceId>) {
    output.insert(expression.span.source);
    for argument in &expression.arguments {
        collect_expression_sources(argument, output);
    }
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        for input in inputs {
            match input {
                IrInlineFunctionInput::Binding { value, span, .. }
                | IrInlineFunctionInput::Selector { value, span } => {
                    output.insert(span.source);
                    collect_expression_sources(value, output);
                }
            }
        }
        for branch in branches {
            collect_function_branch_sources(branch, output);
        }
    }
}

fn collect_function_branch_sources(branch: &IrFunctionBranch, output: &mut BTreeSet<SourceId>) {
    output.insert(branch.span.source);
    match &branch.value {
        IrFunctionBranchValue::Text(text) => collect_text_sources(text, output),
        IrFunctionBranchValue::Dispatch(branches) => {
            for branch in branches {
                collect_function_branch_sources(branch, output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{message_dependency_closure, MessageDependencySymbol};
    use crate::{TypeScriptLocaleModule, TypeScriptProjectOptions, ValidatedTypeScriptProject};
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale, parse_locale_in, parse_schema, parse_schema_in, SourceId};

    fn project_for(schema_text: &str, locale_text: &str) -> ValidatedTypeScriptProject<'static> {
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in(schema_text, SourceId(7)).expect("schema"),
        )));
        let locale = lower_locale(&parse_locale_in(locale_text, SourceId(8)).expect("locale"));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }]));
        let options = TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        ValidatedTypeScriptProject::try_new(schema, locales, &options).expect("validated project")
    }

    fn project() -> ValidatedTypeScriptProject<'static> {
        project_for(
            "type Text = String\nroot(value: Text)\ndrop\n",
            "let helper = Hello\nroot = {helper} {value}\ndrop = Drop\n",
        )
    }

    #[test]
    fn closure_keeps_only_transitive_symbols_and_is_sorted() {
        let project = project();
        let closure = message_dependency_closure(&project, "en", "root").expect("closure");
        assert_eq!(
            closure
                .schema()
                .messages
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            vec!["root"]
        );
        assert_eq!(
            closure
                .locale_module()
                .variables
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            vec!["helper"]
        );
        assert!(!closure
            .locale_module()
            .messages
            .iter()
            .any(|item| item.name == "drop"));
        assert!(closure
            .symbols()
            .windows(2)
            .all(|items| items[0] <= items[1]));
        assert!(closure
            .symbols()
            .contains(&MessageDependencySymbol::SchemaTypeAlias("Text".to_owned())));
        assert_eq!(closure.source_ids(), &[SourceId(7), SourceId(8)]);
    }

    #[test]
    fn closure_rejects_unknown_paths_precisely() {
        let project = project();
        assert!(matches!(
            message_dependency_closure(&project, "fr", "root"),
            Err(crate::TypeScriptCodegenError::UnknownLocale { locale }) if locale == "fr"
        ));
        assert!(matches!(
            message_dependency_closure(&project, "en", "missing"),
            Err(crate::TypeScriptCodegenError::UnknownMessage { message }) if message == "missing"
        ));
        let missing = project_for("root\n", "");
        assert!(matches!(
            message_dependency_closure(&missing, "en", "root"),
            Err(crate::TypeScriptCodegenError::MissingMessageImplementation { locale, message })
                if locale == "en" && message == "root"
        ));
    }

    #[test]
    fn closure_keeps_parameter_forms_functions_inline_branches_and_alias_formatters() {
        let project = project_for(
            "type Price = Number @currency(code = \"EUR\")\nenum Fruit { apple }\nsummary(fruit: Fruit, price: Price)\ndrop\n",
            "fn Render(value: String) { _ => {value} }\nimpl Fruit {\n  apple {\n    one => apple\n    other => apples\n    label = Apple\n  }\n}\nsummary = {fn(fruit, copy: Render(fruit.label)) { apple => {copy} _ => fallback }}\ndrop = Drop\n",
        );
        let closure = message_dependency_closure(&project, "en", "summary").expect("closure");
        assert_eq!(closure.schema().messages.len(), 1);
        assert_eq!(closure.schema().type_aliases.len(), 1);
        assert_eq!(closure.schema().enums.len(), 1);
        assert_eq!(closure.locale_module().forms.len(), 1);
        assert_eq!(closure.locale_module().functions.len(), 1);
        assert!(closure
            .symbols()
            .contains(&MessageDependencySymbol::LocaleForm("Fruit".into())));
        assert!(closure
            .symbols()
            .contains(&MessageDependencySymbol::LocaleFunction("Render".into())));
    }

    #[test]
    fn variable_cycles_terminate_with_unique_symbols() {
        let schema = lower_schema(&parse_schema("root\n").expect("schema"));
        let locale =
            lower_locale(&parse_locale("let a = {b}\nlet b = {a}\nroot = {a}\n").expect("locale"));
        let mut walker = super::ClosureWalker::new(&schema, &locale);
        walker.visit_variable("a");
        let closure = walker.finish("en".into(), "root".into()).expect("closure");
        let variables = closure
            .symbols()
            .iter()
            .filter(|symbol| matches!(symbol, MessageDependencySymbol::LocaleVariable(_)))
            .count();
        assert_eq!(variables, 2);
    }

    #[test]
    fn origin_scopes_do_not_cross_leak_same_named_symbols() {
        let schema = lower_schema(
            &parse_schema_in("enum Shared { one }\nroot\n", SourceId(7)).expect("schema"),
        );
        let locale = lower_locale(
            &linguini_syntax::parse_locale_in("enum Shared { one }\nroot = Root\n", SourceId(8))
                .expect("locale"),
        );
        let mut walker = super::ClosureWalker::new(&schema, &locale);
        walker
            .selected
            .insert(MessageDependencySymbol::SchemaEnum("Shared".into()));
        let closure = walker.finish("en".into(), "root".into()).expect("closure");
        assert_eq!(closure.schema().enums.len(), 1);
        assert!(closure.locale_module().enums.is_empty());
        assert_eq!(closure.source_ids(), &[SourceId(7)]);
    }
}

#[allow(dead_code)]
fn _collect_form_sources(entry: &IrFormEntry, output: &mut BTreeSet<SourceId>) {
    match entry {
        IrFormEntry::Attribute { value, .. } => match value {
            IrValue::Text(text) => collect_text_sources(text, output),
            IrValue::Map(branches) => branches.iter().for_each(|branch| {
                output.insert(branch.span.source);
                collect_text_sources(&branch.value, output);
            }),
            IrValue::Object(entries) => entries
                .iter()
                .for_each(|entry| _collect_form_sources(entry, output)),
        },
        IrFormEntry::Branch(IrBranch { value, span, .. }) => {
            output.insert(span.source);
            collect_text_sources(value, output);
        }
    }
}
