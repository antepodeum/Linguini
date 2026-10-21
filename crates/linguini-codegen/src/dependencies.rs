//! Target-independent transitive message dependencies and source provenance.

use std::collections::{BTreeMap, BTreeSet};

use linguini_ir::{
    IrBranch, IrExpression, IrExpressionKind, IrForm, IrFormEntry, IrFunction, IrFunctionBranch,
    IrFunctionBranchValue, IrInlineFunctionInput, IrModule, IrModuleBuilder, IrSymbolKind, IrText,
    IrTextPart, IrValue,
};
use linguini_syntax::SourceId;

/// Scoped symbol identity for deterministic semantic slicing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MessageDependencySymbol {
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
struct MessageDependencyMetadata {
    source_ids: Vec<SourceId>,
}

/// Minimal validated IR required to compile one message for one fallback-composed locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageDependencyClosure {
    locale: String,
    message: String,
    schema: IrModule,
    locale_module: IrModule,
    metadata: MessageDependencyMetadata,
}

impl MessageDependencyClosure {
    /// The effective locale selected by the caller.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// The original schema message name.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Schema declarations needed by this message, retaining source identities.
    pub fn schema(&self) -> &IrModule {
        &self.schema
    }

    /// Locale declarations transitively referenced by this message.
    pub fn locale_module(&self) -> &IrModule {
        &self.locale_module
    }

    /// Distinct source identities in deterministic order.
    pub fn source_ids(&self) -> &[SourceId] {
        &self.metadata.source_ids
    }
}

/// Selects the transitive semantic IR needed to generate one message.
///
/// The caller supplies an already resolved locale module; locale discovery and fallback
/// composition belong to the host compiler, while slicing is shared by every backend.
pub fn message_dependency_closure(
    schema: &IrModule,
    locale: &str,
    locale_module: &IrModule,
    message: &str,
) -> Result<MessageDependencyClosure, MessageDependencyError> {
    let schema_message = schema
        .messages()
        .iter()
        .find(|item| item.name == message)
        .ok_or_else(|| MessageDependencyError::UnknownMessage {
            message: message.to_owned(),
        })?;
    let implementation = locale_module
        .messages()
        .iter()
        .find(|item| item.name == message)
        .ok_or_else(|| MessageDependencyError::MissingMessageImplementation {
            locale: locale.to_owned(),
            message: message.to_owned(),
        })?;
    let mut walker = ClosureWalker::new(schema, locale_module);
    walker.add_schema_message(schema_message);
    walker.add_locale_message(implementation);
    if let Some(body) = &implementation.body {
        let context = walker.message_context(schema_message);
        walker.visit_text(body, &context);
    }
    Ok(walker.finish(locale.to_owned(), message.to_owned()))
}

/// A requested message cannot be selected from the supplied semantic modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageDependencyError {
    /// The requested message does not exist in the supplied schema.
    UnknownMessage { message: String },
    /// The resolved locale lacks the requested implementation.
    MissingMessageImplementation { locale: String, message: String },
}

impl std::fmt::Display for MessageDependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMessage { message } => write!(f, "unknown message: {message}"),
            Self::MissingMessageImplementation { locale, message } => write!(
                f,
                "locale {locale} has no implementation for message {message}"
            ),
        }
    }
}

impl std::error::Error for MessageDependencyError {}

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
                .type_aliases()
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            schema_enums: schema
                .enums()
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            locale_enums: locale
                .enums()
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            variables: locale
                .variables()
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            forms: locale
                .forms()
                .iter()
                .map(|item| (item.name.as_str(), item))
                .collect(),
            functions: locale
                .functions()
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
            if self.schema.groups().iter().any(|item| item.name == group) {
                self.selected
                    .insert(MessageDependencySymbol::SchemaGroup(group));
            }
        }
    }

    fn message_context(&self, message: &linguini_ir::IrMessage) -> BTreeMap<String, String> {
        message
            .parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
            .collect()
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
                            self.visit_expression(value, context);
                            branch_context.insert(name.clone(), inferred_type(value, context));
                        }
                        IrInlineFunctionInput::Selector { value, .. } => {
                            self.visit_expression(value, context);
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
        if let Some(root_type) = context.get(&path[0]) {
            if path.len() > 1 || called {
                self.visit_form(root_type);
            }
            return;
        }
        let full = path.join(".");
        if self.functions.contains_key(full.as_str()) {
            self.visit_function(full.as_str());
        } else if self.forms.contains_key(full.as_str()) {
            self.visit_form(full.as_str());
        } else if !called && self.variables.contains_key(full.as_str()) {
            self.visit_variable(full.as_str());
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
        let context = BTreeMap::new();
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
        let mut context = BTreeMap::new();
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
        let context = BTreeMap::new();
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

    fn finish(self, locale: String, message: String) -> MessageDependencyClosure {
        let selected = self.selected;
        let schema = slice_schema(self.schema, &selected);
        let locale_module = slice_locale(self.locale, &selected);
        let metadata = metadata_for(self.schema, self.locale, &selected);
        MessageDependencyClosure {
            locale,
            message,
            schema,
            locale_module,
            metadata,
        }
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
    let keep = |kind: IrSymbolKind, name: &str| {
        let symbol = match kind {
            IrSymbolKind::Enum => MessageDependencySymbol::SchemaEnum(name.to_owned()),
            IrSymbolKind::TypeAlias => MessageDependencySymbol::SchemaTypeAlias(name.to_owned()),
            IrSymbolKind::Message => MessageDependencySymbol::SchemaMessage(name.to_owned()),
            IrSymbolKind::Group => MessageDependencySymbol::SchemaGroup(name.to_owned()),
            IrSymbolKind::Variable | IrSymbolKind::Form | IrSymbolKind::Function => return false,
        };
        selected.contains(&symbol)
    };
    IrModuleBuilder::seeded(module)
        .retain_symbols(keep)
        .build()
        .expect("schema slicing preserves unique declaration names")
}

fn slice_locale(module: &IrModule, selected: &BTreeSet<MessageDependencySymbol>) -> IrModule {
    let keep = |kind: IrSymbolKind, name: &str| {
        let symbol = match kind {
            IrSymbolKind::Enum => MessageDependencySymbol::LocaleEnum(name.to_owned()),
            IrSymbolKind::Message => MessageDependencySymbol::LocaleMessage(name.to_owned()),
            IrSymbolKind::Variable => MessageDependencySymbol::LocaleVariable(name.to_owned()),
            IrSymbolKind::Form => MessageDependencySymbol::LocaleForm(name.to_owned()),
            IrSymbolKind::Function => MessageDependencySymbol::LocaleFunction(name.to_owned()),
            IrSymbolKind::TypeAlias | IrSymbolKind::Group => return false,
        };
        selected.contains(&symbol)
    };
    IrModuleBuilder::seeded(module)
        .retain_symbols(keep)
        .build()
        .expect("locale slicing preserves unique declaration names")
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
    for message in schema.messages() {
        if selected.contains(&MessageDependencySymbol::SchemaMessage(
            message.name.clone(),
        )) {
            if let Some(body) = &message.body {
                collect_text_sources(body, &mut source_ids);
            }
        }
    }
    for message in locale.messages() {
        if selected.contains(&MessageDependencySymbol::LocaleMessage(
            message.name.clone(),
        )) {
            if let Some(body) = &message.body {
                collect_text_sources(body, &mut source_ids);
            }
        }
    }
    for variable in locale.variables() {
        if selected.contains(&MessageDependencySymbol::LocaleVariable(
            variable.name.clone(),
        )) {
            collect_text_sources(&variable.value, &mut source_ids);
        }
    }
    for function in locale.functions() {
        if selected.contains(&MessageDependencySymbol::LocaleFunction(
            function.name.clone(),
        )) {
            for branch in &function.branches {
                collect_function_branch_sources(branch, &mut source_ids);
            }
        }
    }
    for form in locale.forms() {
        if selected.contains(&MessageDependencySymbol::LocaleForm(form.name.clone())) {
            for variant in &form.variants {
                for entry in &variant.entries {
                    _collect_form_sources(entry, &mut source_ids);
                }
            }
        }
    }
    MessageDependencyMetadata {
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
    for (index, origin) in module.origins().iter().enumerate() {
        latest.insert((origin.name.as_str(), origin.kind), index);
    }
    for (index, origin) in module.origins().iter().enumerate() {
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

#[cfg(test)]
mod tests {
    use super::{message_dependency_closure, MessageDependencyError, MessageDependencySymbol};
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale, parse_locale_in, parse_schema, parse_schema_in, SourceId};

    #[test]
    fn closure_selects_transitive_ir_and_preserves_source_identity() {
        let schema = lower_schema(
            &parse_schema_in("type Text = String\nroot(value: Text)\ndrop\n", SourceId(7)).unwrap(),
        );
        let locale = lower_locale(
            &parse_locale_in(
                "let helper = Hello\nroot = {helper} {value}\ndrop = Drop\n",
                SourceId(8),
            )
            .unwrap(),
        );
        let closure = message_dependency_closure(&schema, "en", &locale, "root").unwrap();
        assert_eq!(closure.locale(), "en");
        assert_eq!(closure.message(), "root");
        assert_eq!(
            closure
                .schema()
                .messages()
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["root"]
        );
        assert_eq!(
            closure
                .locale_module()
                .variables()
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["helper"]
        );
        assert!(!closure
            .locale_module()
            .messages()
            .iter()
            .any(|item| item.name == "drop"));
        assert_eq!(closure.schema().type_aliases()[0].name, "Text");
        assert_eq!(closure.source_ids(), &[SourceId(7), SourceId(8)]);
    }

    #[test]
    fn dependency_errors_preserve_the_requested_message_and_locale() {
        let schema = lower_schema(&parse_schema("root\n").unwrap());
        let locale = lower_locale(&parse_locale("other = Other\n").unwrap());
        assert_eq!(
            message_dependency_closure(&schema, "fr", &locale, "missing"),
            Err(MessageDependencyError::UnknownMessage {
                message: "missing".into()
            })
        );
        assert_eq!(
            message_dependency_closure(&schema, "fr", &locale, "root"),
            Err(MessageDependencyError::MissingMessageImplementation {
                locale: "fr".into(),
                message: "root".into()
            })
        );
    }

    #[test]
    fn variable_cycles_terminate_with_unique_symbols() {
        let schema = lower_schema(&parse_schema("root\n").expect("schema"));
        let locale =
            lower_locale(&parse_locale("let a = {b}\nlet b = {a}\nroot = {a}\n").expect("locale"));
        let mut walker = super::ClosureWalker::new(&schema, &locale);
        walker.visit_variable("a");
        let closure = walker.finish("en".into(), "root".into());
        assert_eq!(closure.locale_module().variables().len(), 2);
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
        let closure = walker.finish("en".into(), "root".into());
        assert_eq!(closure.schema().enums().len(), 1);
        assert!(closure.locale_module().enums().is_empty());
        assert_eq!(closure.source_ids(), &[SourceId(7)]);
    }
}
