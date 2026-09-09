use linguini_analyzer::{
    analyze_locale_project_coverage, analyze_locale_project_coverage_with_options, Diagnostic,
    LocaleCoverageOptions, RequiredLocaleMessage,
};
use linguini_core::{FormatterKind, TypeKind};
use linguini_syntax::{
    Annotation, DocComment, MessageGroup, MessageSignature, Name, Parameter, SchemaDeclaration,
    SchemaFile, Span, TypeAliasDeclaration,
};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

/// Validated schema symbol table produced by [`build_schema_symbols`] or
/// [`build_schema_symbols_from_files`].
///
/// Declaration maps are read-only outside this crate, so callers cannot inject symbols that
/// bypass duplicate, naming, type-reference, or alias-cycle validation.
///
/// ```compile_fail
/// use linguini_schema::SchemaSymbols;
///
/// let mut symbols = SchemaSymbols::default();
/// symbols.messages.clear();
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaSymbols {
    enums: BTreeMap<String, EnumSymbol>,
    type_aliases: BTreeMap<String, TypeAliasSymbol>,
    messages: BTreeMap<String, MessageSymbol>,
    groups: BTreeMap<String, GroupSymbol>,
}

/// Immutable result of one cross-file schema semantic pass.
///
/// Keeping symbols and diagnostics together prevents consumers from rebuilding partial indexes or
/// accidentally pairing a symbol table with diagnostics from a different source set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaDatabase {
    sources: Vec<SchemaFile>,
    symbols: SchemaSymbols,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumSymbol {
    name: String,
    variants: BTreeMap<String, VariantSymbol>,
    docs: Vec<String>,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantSymbol {
    name: String,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAliasSymbol {
    name: String,
    target: String,
    target_span: Span,
    docs: Vec<String>,
    formatters: Vec<FormatterSymbol>,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatterSymbol {
    kind: FormatterKind,
    arguments: BTreeMap<String, String>,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSymbol {
    name: String,
    group: Option<String>,
    parameters: Vec<ParameterSymbol>,
    docs: Vec<String>,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterSymbol {
    name: String,
    name_span: Span,
    ty: String,
    type_span: Span,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupSymbol {
    name: String,
    messages: Vec<String>,
    groups: Vec<String>,
    docs: Vec<String>,
    span: Span,
}

impl SchemaSymbols {
    pub fn enums(&self) -> &BTreeMap<String, EnumSymbol> {
        &self.enums
    }

    pub fn type_aliases(&self) -> &BTreeMap<String, TypeAliasSymbol> {
        &self.type_aliases
    }

    pub fn messages(&self) -> &BTreeMap<String, MessageSymbol> {
        &self.messages
    }

    pub fn groups(&self) -> &BTreeMap<String, GroupSymbol> {
        &self.groups
    }
}

impl SchemaDatabase {
    pub fn build(schema: &SchemaFile) -> Self {
        Self::build_from_files(std::slice::from_ref(schema))
    }

    pub fn build_from_files(schemas: &[SchemaFile]) -> Self {
        let mut builder = SchemaSymbolBuilder::default();
        for schema in schemas {
            builder.register_declarations(schema);
        }
        builder.resolve_type_references();
        builder.detect_alias_cycles();
        Self {
            sources: schemas.to_vec(),
            symbols: builder.symbols,
            diagnostics: builder.diagnostics,
        }
    }

    pub fn sources(&self) -> &[SchemaFile] {
        &self.sources
    }

    pub fn symbols(&self) -> &SchemaSymbols {
        &self.symbols
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn public_messages(&self) -> Vec<RequiredLocaleMessage> {
        linguini_analyzer::schema_public_messages_from_files(&self.sources)
    }

    pub fn analyze_locale(&self, locale: &linguini_syntax::LocaleFile) -> Vec<Diagnostic> {
        analyze_locale_project_coverage(&self.sources, locale)
    }

    pub fn analyze_locale_with_options(
        &self,
        locale: &linguini_syntax::LocaleFile,
        options: LocaleCoverageOptions,
    ) -> Vec<Diagnostic> {
        analyze_locale_project_coverage_with_options(&self.sources, locale, options)
    }

    pub fn into_parts(self) -> (SchemaSymbols, Vec<Diagnostic>) {
        (self.symbols, self.diagnostics)
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

impl EnumSymbol {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variants(&self) -> &BTreeMap<String, VariantSymbol> {
        &self.variants
    }

    pub fn docs(&self) -> &[String] {
        &self.docs
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl VariantSymbol {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl TypeAliasSymbol {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn target_span(&self) -> Span {
        self.target_span
    }

    pub fn docs(&self) -> &[String] {
        &self.docs
    }

    pub fn formatters(&self) -> &[FormatterSymbol] {
        &self.formatters
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl FormatterSymbol {
    pub fn kind(&self) -> &FormatterKind {
        &self.kind
    }

    pub fn arguments(&self) -> &BTreeMap<String, String> {
        &self.arguments
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl MessageSymbol {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    pub fn parameters(&self) -> &[ParameterSymbol] {
        &self.parameters
    }

    pub fn docs(&self) -> &[String] {
        &self.docs
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl ParameterSymbol {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_span(&self) -> Span {
        self.name_span
    }

    pub fn ty(&self) -> &str {
        &self.ty
    }

    pub fn type_span(&self) -> Span {
        self.type_span
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl GroupSymbol {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    pub fn docs(&self) -> &[String] {
        &self.docs
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

pub fn build_schema_symbols(schema: &SchemaFile) -> (SchemaSymbols, Vec<Diagnostic>) {
    SchemaDatabase::build(schema).into_parts()
}

/// Builds one canonical symbol table for all schema sources.
///
/// Cross-file duplicate checking is intentionally part of this operation. Callers that compile a
/// project must use this function before lowering or code generation rather than concatenating
/// independently lowered vectors.
pub fn build_schema_symbols_from_files(schemas: &[SchemaFile]) -> (SchemaSymbols, Vec<Diagnostic>) {
    SchemaDatabase::build_from_files(schemas).into_parts()
}

#[derive(Default)]
struct SchemaSymbolBuilder {
    symbols: SchemaSymbols,
    declarations: BTreeMap<String, Span>,
    diagnostics: Vec<Diagnostic>,
}

impl SchemaSymbolBuilder {
    fn register_declarations(&mut self, schema: &SchemaFile) {
        for declaration in schema.declarations() {
            match declaration {
                SchemaDeclaration::Enum(declaration) => {
                    self.validate_pascal_name(&declaration.name, "enum");
                    if self.register_name(&declaration.name.value, declaration.name.span) {
                        let variants =
                            self.variants(&declaration.name.value, &declaration.variants);
                        self.symbols.enums.insert(
                            declaration.name.value.clone(),
                            EnumSymbol {
                                name: declaration.name.value.clone(),
                                variants,
                                docs: doc_texts(&declaration.docs),
                                span: declaration.span,
                            },
                        );
                    }
                }
                SchemaDeclaration::TypeAlias(declaration) => self.register_type_alias(declaration),
                SchemaDeclaration::Message(declaration) => {
                    self.register_message(None, &declaration.name.value, declaration);
                }
                SchemaDeclaration::Group(declaration) => self.register_group(None, declaration),
            }
        }
    }

    fn register_type_alias(&mut self, declaration: &TypeAliasDeclaration) {
        self.validate_pascal_name(&declaration.name, "type alias");
        if self.register_name(&declaration.name.value, declaration.name.span) {
            let formatters = self.formatters(&declaration.annotations);
            self.symbols.type_aliases.insert(
                declaration.name.value.clone(),
                TypeAliasSymbol {
                    name: declaration.name.value.clone(),
                    target: declaration.target.value.clone(),
                    target_span: declaration.target.span,
                    docs: doc_texts(&declaration.docs),
                    formatters,
                    span: declaration.span,
                },
            );
        }
    }

    fn register_group(&mut self, parent: Option<&str>, declaration: &MessageGroup) {
        self.validate_lower_name(&declaration.name, "group");
        let name = qualified_name(parent, &declaration.name.value);
        if !self.register_name(&name, declaration.name.span) {
            return;
        }

        let mut messages = Vec::new();
        for message in &declaration.messages {
            let full_name = qualified_name(Some(&name), &message.name.value);
            if self.register_message(Some(&name), &full_name, message) {
                messages.push(full_name);
            }
        }

        let mut groups = Vec::new();
        for child in &declaration.groups {
            let child_name = qualified_name(Some(&name), &child.name.value);
            let existed = self.declarations.contains_key(&child_name);
            self.register_group(Some(&name), child);
            if !existed && self.symbols.groups.contains_key(&child_name) {
                groups.push(child_name);
            }
        }

        self.symbols.groups.insert(
            name.clone(),
            GroupSymbol {
                name,
                messages,
                groups,
                docs: doc_texts(&declaration.docs),
                span: declaration.span,
            },
        );
    }

    fn register_message(
        &mut self,
        group: Option<&str>,
        full_name: &str,
        declaration: &MessageSignature,
    ) -> bool {
        self.validate_lower_name(&declaration.name, "message");
        if !self.register_name(full_name, declaration.name.span) {
            return false;
        }

        let parameters = self.parameters(full_name, &declaration.parameters);
        self.symbols.messages.insert(
            full_name.to_owned(),
            MessageSymbol {
                name: full_name.to_owned(),
                group: group.map(str::to_owned),
                parameters,
                docs: doc_texts(&declaration.docs),
                span: declaration.span,
            },
        );
        true
    }

    fn variants(&mut self, enum_name: &str, variants: &[Name]) -> BTreeMap<String, VariantSymbol> {
        let mut output = BTreeMap::new();
        for variant in variants {
            self.validate_lower_name(variant, "enum variant");
            match output.entry(variant.value.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(VariantSymbol {
                        name: variant.value.clone(),
                        span: variant.span,
                    });
                }
                Entry::Occupied(first) => self.diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "duplicate variant `{}` in enum `{enum_name}`",
                            variant.value
                        ),
                        variant.span,
                    )
                    .with_related(first.get().span, "first variant is here"),
                ),
            }
        }
        output
    }

    fn parameters(&mut self, message_name: &str, parameters: &[Parameter]) -> Vec<ParameterSymbol> {
        let mut output = Vec::new();
        let mut names = BTreeMap::<&str, Span>::new();
        for parameter in parameters {
            self.validate_lower_name(&parameter.name, "message parameter");
            match names.entry(parameter.name.value.as_str()) {
                Entry::Vacant(entry) => {
                    entry.insert(parameter.name.span);
                    output.push(ParameterSymbol {
                        name: parameter.name.value.clone(),
                        name_span: parameter.name.span,
                        ty: parameter.ty.value.clone(),
                        type_span: parameter.ty.span,
                        span: parameter.span,
                    });
                }
                Entry::Occupied(first) => self.diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "duplicate parameter `{}` in message `{message_name}`",
                            parameter.name.value
                        ),
                        parameter.name.span,
                    )
                    .with_related(*first.get(), "first parameter is here"),
                ),
            }
        }
        output
    }

    fn formatters(&mut self, annotations: &[Annotation]) -> Vec<FormatterSymbol> {
        let mut output = Vec::new();
        for annotation in annotations {
            if let FormatterKind::Unknown(name) = &annotation.kind {
                self.diagnostics.push(Diagnostic::error(
                    format!("unknown formatter `{name}`"),
                    annotation.span,
                ));
            }

            let mut arguments = BTreeMap::new();
            for argument in &annotation.arguments {
                match arguments.entry(argument.name.value.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert(argument.value.value.clone());
                    }
                    Entry::Occupied(_) => self.diagnostics.push(Diagnostic::error(
                        format!("duplicate formatter option `{}`", argument.name.value),
                        argument.name.span,
                    )),
                }
            }
            output.push(FormatterSymbol {
                kind: annotation.kind.clone(),
                arguments,
                span: annotation.span,
            });
        }
        output
    }

    fn register_name(&mut self, name: &str, span: Span) -> bool {
        match self.declarations.entry(name.to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(span);
                true
            }
            Entry::Occupied(first) => {
                self.diagnostics
                    .push(duplicate_diagnostic(name, span, *first.get()));
                false
            }
        }
    }

    fn resolve_type_references(&mut self) {
        let known = self.known_types();
        let aliases: Vec<_> = self.symbols.type_aliases.values().cloned().collect();
        for alias in aliases {
            if !known.contains(&alias.target) {
                self.diagnostics
                    .push(unknown_type_diagnostic(&alias.target, alias.target_span));
            }
        }

        let messages: Vec<_> = self.symbols.messages.values().cloned().collect();
        for message in messages {
            for parameter in message.parameters {
                if !known.contains(&parameter.ty) {
                    self.diagnostics.push(
                        unknown_type_diagnostic(&parameter.ty, parameter.type_span)
                            .with_related(message.span, "while checking this message"),
                    );
                }
            }
        }
    }

    fn detect_alias_cycles(&mut self) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            Visiting,
            Complete,
        }

        fn visit(
            name: &str,
            aliases: &BTreeMap<String, TypeAliasSymbol>,
            states: &mut BTreeMap<String, State>,
            stack: &mut Vec<String>,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            if states.get(name) == Some(&State::Complete) {
                return;
            }
            if states.get(name) == Some(&State::Visiting) {
                let offset = stack.iter().position(|item| item == name).unwrap_or(0);
                let mut cycle = stack[offset..].to_vec();
                cycle.push(name.to_owned());
                let alias = &aliases[name];
                let mut diagnostic = Diagnostic::error(
                    format!("cyclic type alias `{}`", cycle.join(" -> ")),
                    alias.target_span,
                );
                for member in &stack[offset..] {
                    diagnostic = diagnostic
                        .with_related(aliases[member].span, format!("alias `{member}` is here"));
                }
                diagnostics.push(diagnostic);
                return;
            }

            states.insert(name.to_owned(), State::Visiting);
            stack.push(name.to_owned());
            if let Some(target) = aliases.get(name).map(|alias| alias.target.as_str()) {
                if aliases.contains_key(target) {
                    visit(target, aliases, states, stack, diagnostics);
                }
            }
            stack.pop();
            states.insert(name.to_owned(), State::Complete);
        }

        let aliases = self.symbols.type_aliases.clone();
        let mut states = BTreeMap::new();
        let mut stack = Vec::new();
        for name in aliases.keys() {
            visit(
                name,
                &aliases,
                &mut states,
                &mut stack,
                &mut self.diagnostics,
            );
        }
    }

    fn known_types(&self) -> BTreeSet<String> {
        TypeKind::all()
            .iter()
            .map(|kind| kind.as_str().to_owned())
            .chain(self.symbols.enums.keys().cloned())
            .chain(self.symbols.type_aliases.keys().cloned())
            .collect()
    }

    fn validate_lower_name(&mut self, name: &Name, kind: &str) {
        if !is_lower_name(&name.value) {
            self.diagnostics.push(Diagnostic::error(
                format!("{kind} `{}` must use lowercase_snake_case", name.value),
                name.span,
            ));
        }
    }

    fn validate_pascal_name(&mut self, name: &Name, kind: &str) {
        if !is_pascal_name(&name.value) {
            self.diagnostics.push(Diagnostic::error(
                format!("{kind} `{}` must use PascalCase", name.value),
                name.span,
            ));
        }
    }
}

fn doc_texts(docs: &[DocComment]) -> Vec<String> {
    docs.iter().map(|doc| doc.text.trim().to_owned()).collect()
}

fn qualified_name(parent: Option<&str>, name: &str) -> String {
    match parent {
        Some(parent) => format!("{parent}.{name}"),
        None => name.to_owned(),
    }
}

fn duplicate_diagnostic(name: &str, span: Span, first_span: Span) -> Diagnostic {
    Diagnostic::error(format!("duplicate schema declaration `{name}`"), span)
        .with_related(first_span, "first declaration is here")
}

fn unknown_type_diagnostic(name: &str, span: Span) -> Diagnostic {
    Diagnostic::error(format!("unknown schema type `{name}`"), span)
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
    first.is_uppercase() && chars.all(char::is_alphanumeric)
}

#[cfg(test)]
mod tests {
    use super::{build_schema_symbols, build_schema_symbols_from_files, SchemaDatabase};
    use linguini_syntax::{
        parse_locale, parse_schema, parse_schema_in, parse_schema_with_recovery, SourceId,
    };

    #[test]
    fn registers_schema_fixture_symbols() {
        let source = include_str!("../../../tests/fixtures/golden/schema/shop.lgs");
        let schema = parse_schema(source).expect("schema parses");
        let (symbols, diagnostics) = build_schema_symbols(&schema);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        assert!(symbols.enums["Fruit"].variants.contains_key("apple"));
        assert_eq!(symbols.type_aliases["Money"].target, "Decimal");
        assert_eq!(symbols.type_aliases["Money"].formatters.len(), 1);
        assert!(symbols.messages.contains_key("delivery"));
        assert!(symbols.messages.contains_key("email_input.label"));
        assert_eq!(
            symbols.messages["delivery"].docs,
            vec!["Displayed on the product delivery confirmation card."]
        );
    }

    #[test]
    fn database_keeps_cross_file_symbols_and_diagnostics_in_one_snapshot() {
        let first = parse_schema_in("hello\n", SourceId(1)).expect("first schema");
        let duplicate = parse_schema_in("hello\n", SourceId(2)).expect("duplicate schema");
        let database = SchemaDatabase::build_from_files(&[first, duplicate]);

        assert_eq!(database.symbols().messages().len(), 1);
        assert_eq!(database.diagnostics().len(), 1);
        assert_eq!(
            database.diagnostics()[0]
                .source_span
                .expect("duplicate source")
                .source,
            SourceId(2)
        );
    }

    #[test]
    fn database_analyzes_locales_against_its_owned_source_set() {
        let types = parse_schema("enum Gender { masculine, feminine }\n").expect("types schema");
        let messages =
            parse_schema("type Voice = Gender\ngreeting(voice: Voice)\n").expect("messages schema");
        let locale =
            parse_locale("greeting = {fn(voice) {\n  masculine => Dear\n  feminine => Kind\n}}\n")
                .expect("locale");
        let database = SchemaDatabase::build_from_files(&[types, messages]);

        assert_eq!(database.sources().len(), 2);
        assert_eq!(database.public_messages().len(), 1);
        assert!(database.analyze_locale(&locale).is_empty());
    }

    #[test]
    fn reports_duplicate_schema_declarations_with_related_span() {
        let schema =
            parse_schema("enum Fruit { apple }\nenum Fruit { pear }\n").expect("schema parses");
        let (_symbols, diagnostics) = build_schema_symbols(&schema);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "duplicate schema declaration `Fruit`"
        );
        assert_eq!(diagnostics[0].related.len(), 1);
    }

    #[test]
    fn reports_unknown_schema_type_at_type_token() {
        let schema = parse_schema("paint(color: Color)\n").expect("schema parses");
        let (_symbols, diagnostics) = build_schema_symbols(&schema);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "unknown schema type `Color`");
        let span = diagnostics[0]
            .source_span
            .expect("unknown type diagnostic source");
        assert_eq!(&"paint(color: Color)\n"[span.start..span.end], "Color");
    }

    #[test]
    fn stores_recursive_group_symbols() {
        let schema = parse_schema(
            "/// Shop\nshop {\n  main {\n    title\n  }\n  local {\n    title\n  }\n}\n",
        )
        .expect("schema parses");
        let (symbols, diagnostics) = build_schema_symbols(&schema);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        assert_eq!(
            symbols.groups["shop"].groups,
            vec!["shop.main", "shop.local"]
        );
        assert_eq!(
            symbols.groups["shop.main"].messages,
            vec!["shop.main.title"]
        );
        assert!(symbols.messages.contains_key("shop.local.title"));
        assert_eq!(symbols.groups["shop"].docs, vec!["Shop"]);
    }

    #[test]
    fn reports_alias_cycles_once() {
        let schema = parse_schema("type A = B\ntype B = C\ntype C = A\n").expect("schema parses");
        let (_symbols, diagnostics) = build_schema_symbols(&schema);

        let cycles = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.message.starts_with("cyclic type alias"))
            .collect::<Vec<_>>();
        assert_eq!(cycles.len(), 1, "{diagnostics:#?}");
        assert_eq!(cycles[0].related.len(), 3);
    }

    #[test]
    fn reports_cross_file_duplicates_with_source_identity() {
        let first = parse_schema_in("delivery\n", SourceId(1)).expect("first schema source parses");
        let second =
            parse_schema_in("delivery\n", SourceId(2)).expect("second schema source parses");
        let (_symbols, diagnostics) = build_schema_symbols_from_files(&[first, second]);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0]
                .source_span
                .expect("duplicate diagnostic source")
                .source,
            SourceId(2)
        );
        assert_eq!(diagnostics[0].related[0].span.source, SourceId(1));
    }

    #[test]
    fn semantic_builder_defends_against_duplicate_variants_in_recovered_ast() {
        let recovered = parse_schema_with_recovery("enum Fruit { apple, apple }\n");
        let schema = recovered.ast.expect("recovered schema AST");
        let (_symbols, diagnostics) = build_schema_symbols(&schema);

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "duplicate variant `apple` in enum `Fruit`"));
    }

    #[test]
    fn semantic_builder_defends_against_duplicate_parameters_in_recovered_ast() {
        let recovered = parse_schema_with_recovery("message(value: String, value: Number)\n");
        let schema = recovered.ast.expect("recovered schema AST");
        let (_symbols, diagnostics) = build_schema_symbols(&schema);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.message == "duplicate parameter `value` in message `message`"
        }));
    }

    #[test]
    fn reports_self_alias_cycle() {
        let schema = parse_schema("type Value = Value\n").expect("schema parses");
        let (_symbols, diagnostics) = build_schema_symbols(&schema);

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "cyclic type alias `Value -> Value`"));
    }
}
