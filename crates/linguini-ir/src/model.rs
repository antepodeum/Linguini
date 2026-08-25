pub use linguini_core::FormatterKind as IrFormatterKind;
use linguini_syntax::Span;

/// Lowered declarations for one schema file, one locale file, or one merged
/// project view.
///
/// Declaration vectors are sealed: code outside this crate reads them through
/// accessors and composes modules through [`IrModule::try_append`] or
/// [`IrModuleBuilder`]. Both composition paths reject duplicate symbol names,
/// so an invalid duplicate-carrying state cannot be constructed publicly.
/// Lowering itself keeps crate-private lossless access because parser recovery
/// must represent partially valid input until semantic diagnosis runs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IrModule {
    pub(crate) enums: Vec<IrEnum>,
    pub(crate) type_aliases: Vec<IrTypeAlias>,
    pub(crate) variables: Vec<IrVariable>,
    pub(crate) messages: Vec<IrMessage>,
    /// Explicit declared message namespaces in canonical path order.
    ///
    /// A group is not inferred from dots in message names: project namespaces and declared
    /// groups share the same path syntax but have different documentation and source identity.
    pub(crate) groups: Vec<IrGroup>,
    pub(crate) forms: Vec<IrForm>,
    pub(crate) functions: Vec<IrFunction>,
    /// Lossless declaration provenance, including entries superseded by `override`.
    pub(crate) origins: Vec<IrOrigin>,
}

/// One declaration kind already containing a symbol name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrSymbolConflict {
    pub kind: IrSymbolKind,
    pub name: String,
}

impl std::fmt::Display for IrSymbolConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            IrSymbolKind::Enum => "enum",
            IrSymbolKind::TypeAlias => "type alias",
            IrSymbolKind::Variable => "variable",
            IrSymbolKind::Message => "message",
            IrSymbolKind::Form => "form",
            IrSymbolKind::Function => "function",
            IrSymbolKind::Group => "group",
        };
        write!(f, "duplicate {kind} symbol `{}`", self.name)
    }
}

impl std::error::Error for IrSymbolConflict {}

macro_rules! declaration_slice {
    ($accessor:ident, $field:ident, $item:ty) => {
        pub fn $accessor(&self) -> &[$item] {
            &self.$field
        }
    };
}

macro_rules! builder_push {
    ($push:ident, $field:ident, $item:ty, $kind:expr) => {
        /// Pushes one declaration; duplicates are rejected by [`Self::build`].
        pub fn $push(mut self, value: $item) -> Self {
            self.module.$field.push(value);
            self
        }
    };
}

impl IrModule {
    declaration_slice!(enums, enums, IrEnum);
    declaration_slice!(type_aliases, type_aliases, IrTypeAlias);
    declaration_slice!(variables, variables, IrVariable);
    declaration_slice!(messages, messages, IrMessage);
    declaration_slice!(groups, groups, IrGroup);
    declaration_slice!(forms, forms, IrForm);
    declaration_slice!(functions, functions, IrFunction);

    /// Lossless declaration provenance, including superseded overrides.
    ///
    /// Unlike declaration kinds, provenance intentionally records multiple
    /// entries per name and is therefore never uniqueness-checked.
    pub fn origins(&self) -> &[IrOrigin] {
        &self.origins
    }

    /// Reports whether `name` is declared as `kind`.
    pub fn contains_symbol(&self, kind: IrSymbolKind, name: &str) -> bool {
        match kind {
            IrSymbolKind::Enum => self.enums.iter().any(|item| item.name == name),
            IrSymbolKind::TypeAlias => self.type_aliases.iter().any(|item| item.name == name),
            IrSymbolKind::Variable => self.variables.iter().any(|item| item.name == name),
            IrSymbolKind::Message => self.messages.iter().any(|item| item.name == name),
            IrSymbolKind::Group => self.groups.iter().any(|item| item.name == name),
            IrSymbolKind::Form => self.forms.iter().any(|item| item.name == name),
            IrSymbolKind::Function => self.functions.iter().any(|item| item.name == name),
        }
    }

    /// Reports whether `name` is declared as any non-group kind.
    ///
    /// Group paths may coexist with a nested member sharing a prefix, but a
    /// message, variable, form, function, enum, or type alias owns its exact
    /// name exclusively within one module.
    pub fn contains_non_group_symbol(&self, name: &str) -> bool {
        IrSymbolKind::non_group_kinds()
            .into_iter()
            .any(|kind| self.contains_symbol(kind, name))
    }

    fn check_conflicts(&self, other: &IrModule) -> Result<(), IrSymbolConflict> {
        // One scan enforces both directions of the contract: a source may not
        // repeat a `(kind, name)` internally, and it may not collide with the
        // declarations this module already owns.
        let mut seen = std::collections::BTreeSet::new();
        macro_rules! check_kind {
            ($field:ident, $kind:expr) => {
                for item in &other.$field {
                    if !seen.insert(($kind, item.name.clone()))
                        || self.contains_symbol($kind, &item.name)
                    {
                        return Err(IrSymbolConflict {
                            kind: $kind,
                            name: item.name.clone(),
                        });
                    }
                }
            };
        }

        check_kind!(enums, IrSymbolKind::Enum);
        check_kind!(type_aliases, IrSymbolKind::TypeAlias);
        check_kind!(variables, IrSymbolKind::Variable);
        check_kind!(messages, IrSymbolKind::Message);
        check_kind!(groups, IrSymbolKind::Group);
        check_kind!(forms, IrSymbolKind::Form);
        check_kind!(functions, IrSymbolKind::Function);
        Ok(())
    }

    /// Moves every declaration of `other` into `self`.
    ///
    /// The move is atomic: it fails when `other` declares a symbol whose
    /// `(kind, name)` already exists in `self`, and a failed call leaves `self`
    /// untouched. One name may exist in different declaration kinds, but never
    /// twice within one kind. Provenance moves with the declarations because it
    /// is append-only by contract.
    pub fn try_append(&mut self, other: IrModule) -> Result<(), IrSymbolConflict> {
        self.check_conflicts(&other)?;
        self.enums.extend(other.enums);
        self.type_aliases.extend(other.type_aliases);
        self.variables.extend(other.variables);
        self.messages.extend(other.messages);
        self.groups.extend(other.groups);
        self.forms.extend(other.forms);
        self.functions.extend(other.functions);
        self.origins.extend(other.origins);
        Ok(())
    }
}

/// Validated constructor for [`IrModule`] values outside this crate.
///
/// Pushes during construction stay infallible so producers can assemble
/// declarations freely; [`IrModuleBuilder::build`] rejects duplicate
/// `(kind, name)` pairs before any module escapes construction.
#[derive(Debug, Clone, Default)]
pub struct IrModuleBuilder {
    module: IrModule,
}

impl IrModuleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds the builder with a copy of every declaration in `module`.
    pub fn seeded(module: &IrModule) -> Self {
        Self {
            module: module.clone(),
        }
    }

    builder_push!(push_enum, enums, IrEnum, IrSymbolKind::Enum);
    builder_push!(
        push_type_alias,
        type_aliases,
        IrTypeAlias,
        IrSymbolKind::TypeAlias
    );
    builder_push!(push_variable, variables, IrVariable, IrSymbolKind::Variable);
    builder_push!(push_message, messages, IrMessage, IrSymbolKind::Message);
    builder_push!(push_group, groups, IrGroup, IrSymbolKind::Group);
    builder_push!(push_form, forms, IrForm, IrSymbolKind::Form);
    builder_push!(push_function, functions, IrFunction, IrSymbolKind::Function);

    /// Appends provenance without uniqueness checks; provenance is append-only.
    pub fn push_origin(mut self, value: IrOrigin) -> Self {
        self.module.origins.push(value);
        self
    }

    pub fn clear_enums(mut self) -> Self {
        self.module.enums.clear();
        self
    }

    pub fn clear_variables(mut self) -> Self {
        self.module.variables.clear();
        self
    }

    pub fn clear_messages(mut self) -> Self {
        self.module.messages.clear();
        self
    }

    pub fn clear_groups(mut self) -> Self {
        self.module.groups.clear();
        self
    }

    pub fn clear_forms(mut self) -> Self {
        self.module.forms.clear();
        self
    }

    pub fn clear_functions(mut self) -> Self {
        self.module.functions.clear();
        self
    }

    /// Keeps only messages whose canonical path satisfies `predicate`.
    pub fn retain_messages(mut self, mut predicate: impl FnMut(&IrMessage) -> bool) -> Self {
        self.module.messages.retain(|message| predicate(message));
        self
    }

    /// Keeps only groups whose canonical path satisfies `predicate`.
    pub fn retain_groups(mut self, mut predicate: impl FnMut(&IrGroup) -> bool) -> Self {
        self.module.groups.retain(|group| predicate(group));
        self
    }

    /// Keeps only declarations (including provenance) whose `(kind, name)`
    /// satisfies `predicate`.
    pub fn retain_symbols(mut self, mut predicate: impl FnMut(IrSymbolKind, &str) -> bool) -> Self {
        self.module
            .enums
            .retain(|item| predicate(IrSymbolKind::Enum, &item.name));
        self.module
            .type_aliases
            .retain(|item| predicate(IrSymbolKind::TypeAlias, &item.name));
        self.module
            .variables
            .retain(|item| predicate(IrSymbolKind::Variable, &item.name));
        self.module
            .messages
            .retain(|item| predicate(IrSymbolKind::Message, &item.name));
        self.module
            .groups
            .retain(|item| predicate(IrSymbolKind::Group, &item.name));
        self.module
            .forms
            .retain(|item| predicate(IrSymbolKind::Form, &item.name));
        self.module
            .functions
            .retain(|item| predicate(IrSymbolKind::Function, &item.name));
        self.module
            .origins
            .retain(|origin| predicate(origin.kind, &origin.name));
        self
    }

    /// Rewrites every message declaration in place during construction.
    ///
    /// Transformations run before [`Self::build`] validates uniqueness, so a
    /// rename that creates a collision still fails construction visibly.
    pub fn update_messages(mut self, update: impl FnMut(&mut IrMessage)) -> Self {
        self.module.messages.iter_mut().for_each(update);
        self
    }

    /// Rewrites every form declaration in place during construction.
    pub fn update_forms(mut self, update: impl FnMut(&mut IrForm)) -> Self {
        self.module.forms.iter_mut().for_each(update);
        self
    }

    /// Rewrites every provenance entry in place during construction.
    pub fn update_origins(mut self, update: impl FnMut(&mut IrOrigin)) -> Self {
        self.module.origins.iter_mut().for_each(update);
        self
    }

    /// Consumes the builder, rejecting duplicate declaration names.
    pub fn build(self) -> Result<IrModule, IrSymbolConflict> {
        let mut validated = IrModule::default();
        validated.try_append(self.module)?;
        Ok(validated)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaIr(pub(crate) IrModule);

impl SchemaIr {
    pub fn as_module(&self) -> &IrModule {
        &self.0
    }

    pub fn into_module(self) -> IrModule {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocaleIr(pub(crate) IrModule);

impl LocaleIr {
    pub fn as_module(&self) -> &IrModule {
        &self.0
    }

    pub fn into_module(self) -> IrModule {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IrSymbolKind {
    Enum,
    TypeAlias,
    Variable,
    Message,
    Form,
    Function,
    Group,
}

impl IrSymbolKind {
    /// Every declaration kind that owns its exact name exclusively.
    ///
    /// Groups are excluded: a group path is a namespace container and may
    /// coexist with nested members sharing that path as a prefix.
    pub fn non_group_kinds() -> [IrSymbolKind; 6] {
        [
            IrSymbolKind::Enum,
            IrSymbolKind::TypeAlias,
            IrSymbolKind::Variable,
            IrSymbolKind::Message,
            IrSymbolKind::Form,
            IrSymbolKind::Function,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrOrigin {
    pub kind: IrSymbolKind,
    pub name: String,
    pub span: Span,
    pub is_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrTypeAlias {
    pub name: String,
    pub target: String,
    pub docs: Vec<String>,
    pub formatters: Vec<IrFormatter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrEnum {
    pub name: String,
    pub docs: Vec<String>,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrMessage {
    pub name: String,
    pub docs: Vec<String>,
    pub parameters: Vec<IrParameter>,
    pub body: Option<IrText>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrGroup {
    pub name: String,
    pub docs: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrParameter {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrVariable {
    pub name: String,
    pub docs: Vec<String>,
    pub value: IrText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrForm {
    pub name: String,
    pub docs: Vec<String>,
    pub variants: Vec<IrFormVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFormVariant {
    pub name: String,
    pub entries: Vec<IrFormEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrFormEntry {
    Attribute {
        name: String,
        parameters: Vec<IrFunctionParameter>,
        value: IrValue,
    },
    Branch(IrBranch),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrValue {
    Text(IrText),
    Map(Vec<IrBranch>),
    Object(Vec<IrFormEntry>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunction {
    pub kind: IrFunctionKind,
    pub name: String,
    pub docs: Vec<String>,
    pub parameters: Vec<IrFunctionParameter>,
    pub branches: Vec<IrFunctionBranch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrFunctionKind {
    Form,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunctionParameter {
    pub name: Option<String>,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunctionBranch {
    pub key: String,
    pub value: IrFunctionBranchValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrFunctionBranchValue {
    Text(IrText),
    Dispatch(Vec<IrFunctionBranch>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBranch {
    pub keys: Vec<String>,
    pub value: IrText,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrText {
    pub parts: Vec<IrTextPart>,
    pub mode: IrTextBlockMode,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrTextBlockMode {
    Inline,
    Dedented,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrTextPart {
    Text(String),
    Placeholder(IrExpression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrExpression {
    pub kind: IrExpressionKind,
    pub path: Vec<String>,
    pub arguments: Vec<IrExpression>,
    pub formatters: Vec<IrFormatter>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrExpressionKind {
    Reference,
    Call,
    /// Immediately dispatches selector expressions and exposes trailing named
    /// bindings to its branch body. `IrExpression::path` and
    /// `IrExpression::arguments` are empty for this variant.
    InlineFunction {
        inputs: Vec<IrInlineFunctionInput>,
        branches: Vec<IrFunctionBranch>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrInlineFunctionInput {
    Binding {
        name: String,
        value: IrExpression,
        span: Span,
    },
    Selector {
        value: IrExpression,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFormatter {
    pub kind: IrFormatterKind,
    pub arguments: Vec<IrFormatterArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFormatterArgument {
    pub name: String,
    pub value: String,
}
