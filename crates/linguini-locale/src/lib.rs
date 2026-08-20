use linguini_analyzer::Diagnostic;
use linguini_syntax::{
    parse_locale_with_recovery, DocComment, EnumDeclaration, FormDeclaration, FormEntry,
    FunctionBranch, FunctionBranchValue, FunctionDeclaration, LocaleDeclaration, LocaleFile,
    LocaleValue, MapBranch, MessageImplementation, MessageImplementationGroup, Name, Span,
};
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocaleScope {
    pub enums: BTreeMap<String, LocaleSymbol>,
    pub functions: BTreeMap<String, LocaleSymbol>,
    pub forms: BTreeMap<String, LocaleSymbol>,
    pub groups: BTreeMap<String, LocaleSymbol>,
    pub messages: BTreeMap<String, LocaleSymbol>,
    pub variables: BTreeMap<String, LocaleSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleSymbol {
    pub name: String,
    pub docs: Vec<String>,
    pub span: Span,
    pub source_index: usize,
    pub source_path: PathBuf,
}

#[derive(Debug)]
pub enum LocaleLoadError {
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for LocaleLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for LocaleLoadError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleScopeSource {
    pub path: PathBuf,
    pub source: String,
}

pub fn load_locale_scope_paths(
    paths: &[PathBuf],
) -> Result<(LocaleScope, Vec<Diagnostic>), LocaleLoadError> {
    let mut sources = Vec::new();
    for path in paths {
        let source = fs::read_to_string(path).map_err(|source| LocaleLoadError::Io {
            path: path.clone(),
            source,
        })?;
        sources.push(LocaleScopeSource {
            path: path.clone(),
            source,
        });
    }
    Ok(load_locale_scope(&sources))
}

pub fn load_locale_scope(sources: &[LocaleScopeSource]) -> (LocaleScope, Vec<Diagnostic>) {
    let mut loader = LocaleScopeLoader::default();
    let hierarchy_is_valid = loader.validate_source_hierarchy(sources);

    for (source_index, source) in sources.iter().enumerate() {
        let parsed = parse_locale_with_recovery(&source.source);
        let has_errors = !parsed.errors.is_empty();
        loader.diagnostics.extend(
            parsed
                .errors
                .into_iter()
                .map(|error| diagnostic_in_source(error.message, error.span, &source.path)),
        );

        if hierarchy_is_valid && !has_errors {
            let Some(file) = parsed.ast else {
                continue;
            };
            loader.merge_file(source_index, &source.path, &file);
        }
    }
    loader.finish()
}

#[derive(Default)]
struct LocaleScopeLoader {
    declarations: BTreeMap<String, DeclaredSymbol>,
    diagnostics: Vec<Diagnostic>,
}

impl LocaleScopeLoader {
    fn validate_source_hierarchy(&mut self, sources: &[LocaleScopeSource]) -> bool {
        let mut is_valid = true;

        for pair in sources.windows(2) {
            let parent = &pair[0].path;
            let child = &pair[1].path;

            if parent.file_name() != child.file_name() {
                self.diagnostics
                    .push(locale_mismatch_diagnostic(parent, child));
                is_valid = false;
                continue;
            }

            let parent_directory = parent.parent().unwrap_or_else(|| Path::new(""));
            let child_directory = child.parent().unwrap_or_else(|| Path::new(""));
            if parent_directory == child_directory || !child_directory.starts_with(parent_directory)
            {
                self.diagnostics
                    .push(invalid_source_order_diagnostic(parent, child));
                is_valid = false;
            }
        }

        is_valid
    }

    fn finish(self) -> (LocaleScope, Vec<Diagnostic>) {
        let mut scope = LocaleScope::default();
        for declaration in self.declarations.into_values() {
            insert_symbol(&mut scope, declaration.kind, declaration.symbol);
        }
        (scope, self.diagnostics)
    }

    fn merge_file(&mut self, source_index: usize, source_path: &Path, file: &LocaleFile) {
        for declaration in &file.declarations {
            self.merge_declaration(source_index, source_path, false, declaration);
        }
    }

    fn merge_declaration(
        &mut self,
        source_index: usize,
        source_path: &Path,
        is_override: bool,
        declaration: &LocaleDeclaration,
    ) {
        match declaration {
            LocaleDeclaration::Enum(declaration) => {
                self.validate_enum_members(source_path, declaration);
                self.register(
                    source_index,
                    source_path,
                    is_override,
                    &declaration.name,
                    &declaration.docs,
                    ScopeKind::Enum,
                );
            }
            LocaleDeclaration::Variable(declaration) => {
                self.register(
                    source_index,
                    source_path,
                    is_override,
                    &declaration.name,
                    &declaration.docs,
                    ScopeKind::Variable,
                );
            }
            LocaleDeclaration::Form(declaration) => {
                self.validate_form_members(source_path, declaration);
                self.register(
                    source_index,
                    source_path,
                    is_override,
                    &declaration.name,
                    &declaration.docs,
                    ScopeKind::Form,
                );
            }
            LocaleDeclaration::Function(declaration) => {
                self.validate_function_members(source_path, declaration);
                self.register(
                    source_index,
                    source_path,
                    is_override,
                    &declaration.name,
                    &declaration.docs,
                    ScopeKind::Function,
                );
            }
            LocaleDeclaration::Message(declaration) => {
                self.register_message(source_index, source_path, is_override, declaration)
            }
            LocaleDeclaration::Group(declaration) => {
                self.register_group(source_index, source_path, is_override, None, declaration);
            }
            LocaleDeclaration::Override(declaration) => {
                self.merge_declaration(source_index, source_path, true, declaration)
            }
        }
    }

    fn register_message(
        &mut self,
        source_index: usize,
        source_path: &Path,
        is_override: bool,
        declaration: &MessageImplementation,
    ) {
        self.register(
            source_index,
            source_path,
            is_override,
            &declaration.name,
            &declaration.docs,
            ScopeKind::Message,
        );
    }

    fn register_group(
        &mut self,
        source_index: usize,
        source_path: &Path,
        is_override: bool,
        parent: Option<&str>,
        declaration: &MessageImplementationGroup,
    ) {
        let group_name = match parent {
            Some(parent) => format!("{parent}.{}", declaration.name.value),
            None => declaration.name.value.clone(),
        };
        let name = Name {
            value: group_name.clone(),
            span: declaration.name.span,
        };
        if !self.register(
            source_index,
            source_path,
            is_override,
            &name,
            &declaration.docs,
            ScopeKind::Group,
        ) {
            return;
        }

        for message in &declaration.messages {
            let name = Name {
                value: format!("{group_name}.{}", message.name.value),
                span: message.name.span,
            };
            self.register(
                source_index,
                source_path,
                is_override,
                &name,
                &message.docs,
                ScopeKind::Message,
            );
        }

        for child in &declaration.groups {
            self.register_group(
                source_index,
                source_path,
                is_override,
                Some(&group_name),
                child,
            );
        }
    }

    fn register(
        &mut self,
        source_index: usize,
        source_path: &Path,
        is_override: bool,
        name: &Name,
        docs: &[DocComment],
        kind: ScopeKind,
    ) -> bool {
        let symbol = LocaleSymbol {
            name: name.value.clone(),
            docs: doc_texts(docs),
            span: name.span,
            source_index,
            source_path: source_path.to_path_buf(),
        };

        match self.declarations.entry(name.value.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(DeclaredSymbol { kind, symbol });
                true
            }
            Entry::Occupied(entry) if entry.get().symbol.source_index == source_index => {
                self.diagnostics.push(duplicate_diagnostic(
                    &name.value,
                    &symbol,
                    &entry.get().symbol,
                ));
                false
            }
            Entry::Occupied(mut entry) if is_override => {
                entry.insert(DeclaredSymbol { kind, symbol });
                true
            }
            Entry::Occupied(entry) => {
                self.diagnostics.push(invalid_shadow_diagnostic(
                    &name.value,
                    &symbol,
                    &entry.get().symbol,
                ));
                false
            }
        }
    }

    fn validate_enum_members(&mut self, source_path: &Path, declaration: &EnumDeclaration) {
        validate_unique_names(
            declaration.variants.iter(),
            "variant",
            &format!("enum `{}`", declaration.name.value),
            source_path,
            &mut self.diagnostics,
        );
    }

    fn validate_form_members(&mut self, source_path: &Path, declaration: &FormDeclaration) {
        validate_unique_names(
            declaration.variants.iter().map(|variant| &variant.name),
            "variant",
            &format!("form `{}`", declaration.name.value),
            source_path,
            &mut self.diagnostics,
        );

        for variant in &declaration.variants {
            validate_form_entries(
                &format!(
                    "form `{}` variant `{}`",
                    declaration.name.value, variant.name.value
                ),
                &variant.entries,
                source_path,
                &mut self.diagnostics,
            );
        }
    }

    fn validate_function_members(&mut self, source_path: &Path, declaration: &FunctionDeclaration) {
        let container = format!("function `{}`", declaration.name.value);
        validate_unique_names(
            declaration
                .parameters
                .iter()
                .filter_map(|parameter| parameter.name.as_ref()),
            "parameter",
            &container,
            source_path,
            &mut self.diagnostics,
        );
        validate_function_branches(
            &container,
            &declaration.branches,
            source_path,
            &mut self.diagnostics,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Enum,
    Function,
    Form,
    Group,
    Message,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclaredSymbol {
    kind: ScopeKind,
    symbol: LocaleSymbol,
}

fn insert_symbol(scope: &mut LocaleScope, kind: ScopeKind, symbol: LocaleSymbol) {
    let symbols = match kind {
        ScopeKind::Enum => &mut scope.enums,
        ScopeKind::Function => &mut scope.functions,
        ScopeKind::Form => &mut scope.forms,
        ScopeKind::Group => &mut scope.groups,
        ScopeKind::Message => &mut scope.messages,
        ScopeKind::Variable => &mut scope.variables,
    };
    symbols.insert(symbol.name.clone(), symbol);
}

fn validate_unique_names<'a>(
    names: impl IntoIterator<Item = &'a Name>,
    member_kind: &str,
    container: &str,
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut first_spans = BTreeMap::new();
    for name in names {
        match first_spans.entry(name.value.as_str()) {
            Entry::Vacant(entry) => {
                entry.insert(name.span);
            }
            Entry::Occupied(entry) => diagnostics.push(duplicate_member_diagnostic(
                member_kind,
                container,
                &name.value,
                name.span,
                *entry.get(),
                source_path,
            )),
        }
    }
}

fn validate_function_branches(
    container: &str,
    branches: &[FunctionBranch],
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_unique_names(
        branches.iter().map(|branch| &branch.key),
        "branch",
        container,
        source_path,
        diagnostics,
    );

    for branch in branches {
        if let FunctionBranchValue::Dispatch(children) = &branch.value {
            validate_function_branches(container, children, source_path, diagnostics);
        }
    }
}

fn validate_form_entries(
    container: &str,
    entries: &[FormEntry],
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_unique_names(
        entries.iter().filter_map(|entry| match entry {
            FormEntry::Attribute(attribute) => Some(&attribute.name),
            FormEntry::Branch(_) => None,
        }),
        "attribute",
        container,
        source_path,
        diagnostics,
    );

    validate_map_branches(
        container,
        entries.iter().filter_map(|entry| match entry {
            FormEntry::Attribute(_) => None,
            FormEntry::Branch(branch) => Some(branch),
        }),
        source_path,
        diagnostics,
    );

    for entry in entries {
        let FormEntry::Attribute(attribute) = entry else {
            continue;
        };
        let nested_container = format!("{container} attribute `{}`", attribute.name.value);
        match &attribute.value {
            LocaleValue::Text(_) => {}
            LocaleValue::Map(branches) => {
                validate_map_branches(&nested_container, branches.iter(), source_path, diagnostics);
            }
            LocaleValue::Object(entries) => {
                validate_form_entries(&nested_container, entries, source_path, diagnostics);
            }
        }
    }
}

fn validate_map_branches<'a>(
    container: &str,
    branches: impl IntoIterator<Item = &'a MapBranch>,
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut first_spans = BTreeMap::new();
    for branch in branches {
        let pattern = branch
            .keys
            .iter()
            .map(|key| key.value.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        match first_spans.entry(pattern.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(branch.span);
            }
            Entry::Occupied(entry) => diagnostics.push(duplicate_member_diagnostic(
                "branch",
                container,
                &pattern,
                branch.span,
                *entry.get(),
                source_path,
            )),
        }
    }
}

fn doc_texts(docs: &[DocComment]) -> Vec<String> {
    docs.iter().map(|doc| doc.text.trim().to_owned()).collect()
}

fn diagnostic_in_source(message: impl Into<String>, span: Span, source_path: &Path) -> Diagnostic {
    Diagnostic::error(message, span).with_note(format!("source: {}", source_path.display()))
}

fn duplicate_diagnostic(
    name: &str,
    declaration: &LocaleSymbol,
    first: &LocaleSymbol,
) -> Diagnostic {
    diagnostic_in_source(
        format!("duplicate locale declaration `{name}`"),
        declaration.span,
        &declaration.source_path,
    )
    .with_related(
        first.span,
        format!(
            "first declaration is here in {}",
            first.source_path.display()
        ),
    )
}

fn invalid_shadow_diagnostic(
    name: &str,
    declaration: &LocaleSymbol,
    parent: &LocaleSymbol,
) -> Diagnostic {
    diagnostic_in_source(
        format!("locale declaration `{name}` shadows a parent declaration without `override`"),
        declaration.span,
        &declaration.source_path,
    )
    .with_related(
        parent.span,
        format!(
            "parent declaration is here in {}",
            parent.source_path.display()
        ),
    )
}

fn duplicate_member_diagnostic(
    member_kind: &str,
    container: &str,
    name: &str,
    span: Span,
    first_span: Span,
    source_path: &Path,
) -> Diagnostic {
    diagnostic_in_source(
        format!("duplicate {member_kind} `{name}` in {container}"),
        span,
        source_path,
    )
    .with_related(first_span, "first member is here")
}

fn locale_mismatch_diagnostic(parent: &Path, child: &Path) -> Diagnostic {
    Diagnostic::error(
        format!(
            "locale scope source {} does not match parent locale source {}",
            child.display(),
            parent.display()
        ),
        Span::new(0, 0),
    )
    .without_source()
}

fn invalid_source_order_diagnostic(parent: &Path, child: &Path) -> Diagnostic {
    Diagnostic::error(
        format!(
            "locale scope source {} is not a child of preceding source {}",
            child.display(),
            parent.display()
        ),
        Span::new(0, 0),
    )
    .without_source()
}

#[cfg(test)]
mod tests {
    use super::{load_locale_scope, load_locale_scope_paths, LocaleScopeSource};
    use linguini_test_support::temp_project_dir;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn loads_root_parent_and_child_scope_files() {
        let project =
            temp_project_dir("loads_root_parent_and_child_scope_files").expect("temporary project");
        let root = project.path().join("locale/ru.lgl");
        let parent = project.path().join("locale/shop/ru.lgl");
        let child = project.path().join("locale/shop/delivery/ru.lgl");
        fs::create_dir_all(child.parent().expect("child parent")).expect("dirs");
        fs::create_dir_all(parent.parent().expect("parent parent")).expect("dirs");
        fs::write(&root, "enum gender { other }\n").expect("root");
        fs::write(&parent, "fn delivered(gender) {\n  else => ok\n}\n").expect("parent");
        fs::write(&child, "delivery = Done\n").expect("child");

        let (scope, diagnostics) =
            load_locale_scope_paths(&[root, parent, child]).expect("load scope");

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(scope.enums.contains_key("gender"));
        assert!(scope.functions.contains_key("delivered"));
        assert!(scope.messages.contains_key("delivery"));
    }

    #[test]
    fn child_scope_can_use_parent_declarations() {
        let sources = vec![
            source("locale/ru.lgl", "enum gender { other }\n"),
            source(
                "locale/shop/ru.lgl",
                "fn delivered(gender) {\n  else => ok\n}\n",
            ),
            source("locale/shop/delivery/ru.lgl", "delivery = Done\n"),
        ];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(scope.enums["gender"].source_index, 0);
        assert_eq!(scope.functions["delivered"].source_index, 1);
        assert_eq!(scope.messages["delivery"].source_index, 2);
    }

    #[test]
    fn detects_duplicate_declarations_in_same_scope_file() {
        let sources = vec![source(
            "locale/ru.lgl",
            "enum gender { other }\nenum gender { other }\n",
        )];
        let (_scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "duplicate locale declaration `gender`"
        );
        assert_eq!(diagnostics[0].related.len(), 1);
    }

    #[test]
    fn rejects_child_shadowing_without_override() {
        let sources = vec![
            source("locale/ru.lgl", "enum gender { other }\n"),
            source("locale/shop/ru.lgl", "enum gender { other }\n"),
        ];
        let (_scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("without `override`"));
        assert_eq!(diagnostics[0].related.len(), 1);
    }

    #[test]
    fn explicit_override_replaces_parent_declaration() {
        let sources = vec![
            source("locale/ru.lgl", "enum Gender { other }\n"),
            source("locale/shop/ru.lgl", "override enum Gender { male }\n"),
        ];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(scope.enums["Gender"].source_index, 1);
    }

    #[test]
    fn registers_variables_separately_from_messages() {
        let sources = vec![source(
            "locale/ru.lgl",
            "let cart_label = In cart\ncart = Cart\n",
        )];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(scope.variables.contains_key("cart_label"));
        assert!(!scope.messages.contains_key("cart_label"));
        assert!(scope.messages.contains_key("cart"));
    }

    #[test]
    fn cross_kind_override_replaces_parent_index_atomically() {
        let sources = vec![
            source("locale/ru.lgl", "notice = Parent\n"),
            source("locale/shop/ru.lgl", "override enum notice { other }\n"),
        ];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(scope.enums["notice"].source_index, 1);
        assert!(!scope.messages.contains_key("notice"));
        assert!(!scope.variables.contains_key("notice"));
    }

    #[test]
    fn rejects_same_source_override_without_replacing_original() {
        let sources = vec![source(
            "locale/ru.lgl",
            "notice = Original\noverride enum notice { other }\n",
        )];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "duplicate locale declaration `notice`"
        );
        assert!(scope.messages.contains_key("notice"));
        assert!(!scope.enums.contains_key("notice"));
    }

    #[test]
    fn rejected_same_source_group_override_does_not_leak_members() {
        let sources = vec![source(
            "locale/ru.lgl",
            "account {\n  title = Original\n}\n\
             override account {\n  subtitle = Leaked\n}\n",
        )];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert!(scope.groups.contains_key("account"));
        assert!(scope.messages.contains_key("account.title"));
        assert!(!scope.messages.contains_key("account.subtitle"));
    }

    #[test]
    fn recovered_invalid_source_does_not_shadow_parent() {
        let sources = vec![
            source("locale/ru.lgl", "notice = Parent\n"),
            source("locale/shop/ru.lgl", "override notice = Broken\n#\n"),
        ];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(scope.messages["notice"].source_index, 0);
    }

    #[test]
    fn rejects_sources_that_are_not_ordered_parent_before_child() {
        let sources = vec![
            source("locale/shop/ru.lgl", "child = Child\n"),
            source("locale/ru.lgl", "root = Root\n"),
        ];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("is not a child"));
        assert_eq!(scope, Default::default());
    }

    #[test]
    fn rejects_mixed_locale_source_hierarchies() {
        let sources = vec![
            source("locale/en.lgl", "root = Root\n"),
            source("locale/shop/ru.lgl", "child = Child\n"),
        ];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("does not match"));
        assert_eq!(scope, Default::default());
    }

    #[test]
    fn reports_cross_source_identity_in_shadow_diagnostic() {
        let sources = vec![
            source("locale/ru.lgl", "notice = Parent\n"),
            source("locale/shop/ru.lgl", "notice = Child\n"),
        ];
        let (_scope, diagnostics) = load_locale_scope(&sources);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].note.as_deref(),
            Some("source: locale/shop/ru.lgl")
        );
        assert!(diagnostics[0].related[0].message.contains("locale/ru.lgl"));
    }

    #[test]
    fn detects_duplicate_enum_form_and_function_members() {
        let sources = vec![source(
            "locale/ru.lgl",
            "enum Tone { formal, formal }\n\
             impl Fruit {\n\
               apple {\n\
                 emoji = red\n\
                 emoji = green\n\
                 form label(Plural) {\n\
                   one => one\n\
                   one => another\n\
                 }\n\
               }\n\
               apple { emoji = duplicate }\n\
             }\n\
             fn note(item: String, item: String) {\n\
               one => first\n\
               one => second\n\
             }\n",
        )];
        let (_scope, diagnostics) = load_locale_scope(&sources);
        let messages = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>();

        assert!(messages
            .iter()
            .any(|message| message.contains("duplicate variant `formal` in enum `Tone`")));
        assert!(messages
            .iter()
            .any(|message| message.contains("duplicate variant `apple` in form `Fruit`")));
        assert!(messages.iter().any(|message| {
            message.contains("duplicate attribute `emoji` in form `Fruit` variant `apple`")
        }));
        assert!(messages.iter().any(|message| {
            message.contains("duplicate branch `one` in form `Fruit` variant `apple`")
        }));
        assert!(messages
            .iter()
            .any(|message| { message.contains("duplicate parameter `item` in function `note`") }));
        assert!(messages
            .iter()
            .any(|message| message.contains("duplicate branch `one` in function `note`")));
    }

    #[test]
    fn registers_forms_and_grouped_messages() {
        let sources = vec![source(
            "locale/ru.lgl",
            "impl Fruit {\n  apple {\n    form nom(Plural) {\n      _ => apple\n    }\n  }\n}\nemail {\n  label = Email\n}\n",
        )];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(scope.forms.contains_key("Fruit"));
        assert!(scope.groups.contains_key("email"));
        assert!(!scope.messages.contains_key("email"));
        assert!(scope.messages.contains_key("email.label"));
    }

    #[test]
    fn registers_nested_groups_with_canonical_message_paths() {
        let sources = vec![source(
            "locale/ru.lgl",
            "account {\n  profile {\n    title = Profile\n  }\n}\n",
        )];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(scope.groups.contains_key("account"));
        assert!(scope.groups.contains_key("account.profile"));
        assert!(scope.messages.contains_key("account.profile.title"));
    }

    fn source(path: &str, source: &str) -> LocaleScopeSource {
        LocaleScopeSource {
            path: PathBuf::from(path),
            source: source.to_owned(),
        }
    }
}
