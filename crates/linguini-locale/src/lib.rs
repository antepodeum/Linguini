use linguini_analyzer::Diagnostic;
use linguini_syntax::{
    parse_locale_with_recovery, DocComment, LocaleDeclaration, LocaleFile, MessageImplementation,
    MessageImplementationGroup, Name, Span,
};
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const CRATE_PURPOSE: &str = "locale AST and scope loading";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocaleScope {
    pub enums: BTreeMap<String, LocaleSymbol>,
    pub functions: BTreeMap<String, LocaleSymbol>,
    pub forms: BTreeMap<String, LocaleSymbol>,
    pub messages: BTreeMap<String, LocaleSymbol>,
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
    for (source_index, source) in sources.iter().enumerate() {
        let parsed = parse_locale_with_recovery(&source.source);
        loader.diagnostics.extend(
            parsed
                .errors
                .into_iter()
                .map(|error| Diagnostic::error(error.message, error.span)),
        );

        if let Some(file) = parsed.ast {
            loader.merge_file(source_index, &source.path, &file);
        }
    }
    (loader.scope, loader.diagnostics)
}

#[derive(Default)]
struct LocaleScopeLoader {
    scope: LocaleScope,
    declarations: BTreeMap<String, LocaleSymbol>,
    diagnostics: Vec<Diagnostic>,
}

impl LocaleScopeLoader {
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
            LocaleDeclaration::Enum(declaration) => self.register(
                source_index,
                source_path,
                is_override,
                &declaration.name,
                &declaration.docs,
                ScopeKind::Enum,
            ),
            LocaleDeclaration::Variable(declaration) => self.register(
                source_index,
                source_path,
                is_override,
                &declaration.name,
                &declaration.docs,
                ScopeKind::Message,
            ),
            LocaleDeclaration::Form(declaration) => self.register(
                source_index,
                source_path,
                is_override,
                &declaration.name,
                &declaration.docs,
                ScopeKind::Form,
            ),
            LocaleDeclaration::Function(declaration) => self.register(
                source_index,
                source_path,
                is_override,
                &declaration.name,
                &declaration.docs,
                ScopeKind::Function,
            ),
            LocaleDeclaration::Message(declaration) => {
                self.register_message(source_index, source_path, is_override, declaration)
            }
            LocaleDeclaration::Group(declaration) => {
                self.register_group(source_index, source_path, is_override, declaration);
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
        declaration: &MessageImplementationGroup,
    ) {
        self.register(
            source_index,
            source_path,
            is_override,
            &declaration.name,
            &declaration.docs,
            ScopeKind::Message,
        );

        for message in &declaration.messages {
            let name = Name {
                value: format!("{}.{}", declaration.name.value, message.name.value),
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
    }

    fn register(
        &mut self,
        source_index: usize,
        source_path: &Path,
        is_override: bool,
        name: &Name,
        docs: &[DocComment],
        kind: ScopeKind,
    ) {
        let symbol = LocaleSymbol {
            name: name.value.clone(),
            docs: doc_texts(docs),
            span: name.span,
            source_index,
            source_path: source_path.to_path_buf(),
        };

        match self.declarations.entry(name.value.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(symbol.clone());
                self.insert_symbol(kind, symbol);
            }
            Entry::Occupied(mut entry) if is_override => {
                entry.insert(symbol.clone());
                self.insert_symbol(kind, symbol);
            }
            Entry::Occupied(entry) if entry.get().source_index == source_index => {
                self.diagnostics.push(duplicate_diagnostic(
                    &name.value,
                    name.span,
                    entry.get().span,
                ));
            }
            Entry::Occupied(entry) => {
                self.diagnostics.push(invalid_shadow_diagnostic(
                    &name.value,
                    name.span,
                    entry.get().span,
                ));
            }
        }
    }

    fn insert_symbol(&mut self, kind: ScopeKind, symbol: LocaleSymbol) {
        let symbols = match kind {
            ScopeKind::Enum => &mut self.scope.enums,
            ScopeKind::Function => &mut self.scope.functions,
            ScopeKind::Form => &mut self.scope.forms,
            ScopeKind::Message => &mut self.scope.messages,
        };
        symbols.insert(symbol.name.clone(), symbol);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Enum,
    Function,
    Form,
    Message,
}

fn doc_texts(docs: &[DocComment]) -> Vec<String> {
    docs.iter().map(|doc| doc.text.trim().to_owned()).collect()
}

fn duplicate_diagnostic(name: &str, span: Span, first_span: Span) -> Diagnostic {
    Diagnostic::error(format!("duplicate locale declaration `{name}`"), span)
        .with_related(first_span, "first declaration is here")
}

fn invalid_shadow_diagnostic(name: &str, span: Span, parent_span: Span) -> Diagnostic {
    Diagnostic::error(
        format!("locale declaration `{name}` shadows a parent declaration without `override`"),
        span,
    )
    .with_related(parent_span, "parent declaration is here")
}

#[cfg(test)]
mod tests {
    use super::{load_locale_scope, load_locale_scope_paths, LocaleScopeSource};
    use linguini_test_support::temp_project_dir;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn loads_root_parent_and_child_scope_files() {
        let project = temp_project_dir("loads_root_parent_and_child_scope_files");
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
    fn registers_forms_and_grouped_messages() {
        let sources = vec![source(
            "locale/ru.lgl",
            "impl Fruit {\n  apple {\n    form nom(Plural) {\n      _ => apple\n    }\n  }\n}\nemail {\n  label = Email\n}\n",
        )];
        let (scope, diagnostics) = load_locale_scope(&sources);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(scope.forms.contains_key("Fruit"));
        assert!(scope.messages.contains_key("email"));
        assert!(scope.messages.contains_key("email.label"));
    }

    fn source(path: &str, source: &str) -> LocaleScopeSource {
        LocaleScopeSource {
            path: PathBuf::from(path),
            source: source.to_owned(),
        }
    }
}
