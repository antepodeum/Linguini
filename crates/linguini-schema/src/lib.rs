use linguini_analyzer::Diagnostic;
use linguini_syntax::{
    DocComment, MessageSignature, Name, Parameter, SchemaDeclaration, SchemaFile, Span,
    TypeAliasDeclaration,
};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

pub const CRATE_PURPOSE: &str = "schema AST and symbol table";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaSymbols {
    pub enums: BTreeMap<String, EnumSymbol>,
    pub type_aliases: BTreeMap<String, TypeAliasSymbol>,
    pub messages: BTreeMap<String, MessageSymbol>,
    pub groups: BTreeMap<String, GroupSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumSymbol {
    pub name: String,
    pub variants: BTreeMap<String, VariantSymbol>,
    pub docs: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantSymbol {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAliasSymbol {
    pub name: String,
    pub target: String,
    pub docs: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSymbol {
    pub name: String,
    pub group: Option<String>,
    pub parameters: Vec<ParameterSymbol>,
    pub docs: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterSymbol {
    pub name: String,
    pub ty: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupSymbol {
    pub name: String,
    pub messages: Vec<String>,
    pub docs: Vec<String>,
    pub span: Span,
}

pub fn build_schema_symbols(schema: &SchemaFile) -> (SchemaSymbols, Vec<Diagnostic>) {
    let mut builder = SchemaSymbolBuilder::default();
    builder.register_declarations(schema);
    builder.resolve_type_references();
    (builder.symbols, builder.diagnostics)
}

#[derive(Default)]
struct SchemaSymbolBuilder {
    symbols: SchemaSymbols,
    declarations: BTreeMap<String, Span>,
    diagnostics: Vec<Diagnostic>,
}

impl SchemaSymbolBuilder {
    fn register_declarations(&mut self, schema: &SchemaFile) {
        for declaration in &schema.declarations {
            match declaration {
                SchemaDeclaration::Enum(declaration) => {
                    if self.register_name(&declaration.name) {
                        self.symbols.enums.insert(
                            declaration.name.value.clone(),
                            EnumSymbol {
                                name: declaration.name.value.clone(),
                                variants: variants(&declaration.variants),
                                docs: doc_texts(&declaration.docs),
                                span: declaration.span,
                            },
                        );
                    }
                }
                SchemaDeclaration::TypeAlias(declaration) => self.register_type_alias(declaration),
                SchemaDeclaration::Message(declaration) => {
                    self.register_message(None, declaration);
                }
                SchemaDeclaration::Group(declaration) => {
                    if self.register_name(&declaration.name) {
                        let group_name = declaration.name.value.clone();
                        let mut messages = Vec::new();
                        for message in &declaration.messages {
                            let full_name = grouped_name(&group_name, &message.name.value);
                            messages.push(full_name.clone());
                            self.register_grouped_message(&group_name, full_name, message);
                        }
                        self.symbols.groups.insert(
                            group_name.clone(),
                            GroupSymbol {
                                name: group_name,
                                messages,
                                docs: doc_texts(&declaration.docs),
                                span: declaration.span,
                            },
                        );
                    }
                }
            }
        }
    }

    fn register_type_alias(&mut self, declaration: &TypeAliasDeclaration) {
        if self.register_name(&declaration.name) {
            self.symbols.type_aliases.insert(
                declaration.name.value.clone(),
                TypeAliasSymbol {
                    name: declaration.name.value.clone(),
                    target: declaration.target.value.clone(),
                    docs: doc_texts(&declaration.docs),
                    span: declaration.span,
                },
            );
        }
    }

    fn register_message(&mut self, group: Option<&str>, declaration: &MessageSignature) {
        if self.register_name(&declaration.name) {
            self.insert_message(declaration.name.value.clone(), group, declaration);
        }
    }

    fn register_grouped_message(
        &mut self,
        group: &str,
        full_name: String,
        declaration: &MessageSignature,
    ) {
        match self.symbols.messages.entry(full_name.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(message_symbol(full_name, Some(group), declaration));
            }
            Entry::Occupied(first) => self.diagnostics.push(duplicate_diagnostic(
                &full_name,
                declaration.name.span,
                first.get().span,
            )),
        }
    }

    fn insert_message(
        &mut self,
        name: String,
        group: Option<&str>,
        declaration: &MessageSignature,
    ) {
        self.symbols
            .messages
            .insert(name.clone(), message_symbol(name, group, declaration));
    }

    fn register_name(&mut self, name: &Name) -> bool {
        match self.declarations.entry(name.value.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(name.span);
                true
            }
            Entry::Occupied(first) => {
                self.diagnostics
                    .push(duplicate_diagnostic(&name.value, name.span, *first.get()));
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
                    .push(unknown_type_diagnostic(&alias.target, alias.span));
            }
        }

        let messages: Vec<_> = self.symbols.messages.values().cloned().collect();
        for message in messages {
            for parameter in message.parameters {
                if !known.contains(&parameter.ty) {
                    self.diagnostics.push(
                        unknown_type_diagnostic(&parameter.ty, parameter.span)
                            .with_related(message.span, "while checking this message"),
                    );
                }
            }
        }
    }

    fn known_types(&self) -> BTreeSet<String> {
        ["String", "Number", "Decimal", "Date", "Boolean"]
            .into_iter()
            .map(str::to_owned)
            .chain(self.symbols.enums.keys().cloned())
            .chain(self.symbols.type_aliases.keys().cloned())
            .collect()
    }
}

fn variants(variants: &[Name]) -> BTreeMap<String, VariantSymbol> {
    variants
        .iter()
        .map(|variant| {
            (
                variant.value.clone(),
                VariantSymbol {
                    name: variant.value.clone(),
                    span: variant.span,
                },
            )
        })
        .collect()
}

fn message_symbol(
    name: String,
    group: Option<&str>,
    declaration: &MessageSignature,
) -> MessageSymbol {
    MessageSymbol {
        name,
        group: group.map(str::to_owned),
        parameters: parameters(&declaration.parameters),
        docs: doc_texts(&declaration.docs),
        span: declaration.span,
    }
}

fn parameters(parameters: &[Parameter]) -> Vec<ParameterSymbol> {
    parameters
        .iter()
        .map(|parameter| ParameterSymbol {
            name: parameter.name.value.clone(),
            ty: parameter.ty.value.clone(),
            span: parameter.span,
        })
        .collect()
}

fn doc_texts(docs: &[DocComment]) -> Vec<String> {
    docs.iter().map(|doc| doc.text.trim().to_owned()).collect()
}

fn grouped_name(group: &str, message: &str) -> String {
    format!("{group}.{message}")
}

fn duplicate_diagnostic(name: &str, span: Span, first_span: Span) -> Diagnostic {
    Diagnostic::error(format!("duplicate schema declaration `{name}`"), span)
        .with_related(first_span, "first declaration is here")
}

fn unknown_type_diagnostic(name: &str, span: Span) -> Diagnostic {
    Diagnostic::error(format!("unknown schema type `{name}`"), span)
}

#[cfg(test)]
mod tests {
    use super::build_schema_symbols;
    use linguini_syntax::parse_schema;

    #[test]
    fn registers_schema_fixture_symbols() {
        let source = include_str!("../../../tests/fixtures/golden/schema/shop.lgs");
        let schema = parse_schema(source).expect("schema parses");
        let (symbols, diagnostics) = build_schema_symbols(&schema);

        assert!(diagnostics.is_empty());
        assert!(symbols.enums["Fruit"].variants.contains_key("apple"));
        assert_eq!(symbols.type_aliases["Money"].target, "Decimal");
        assert!(symbols.messages.contains_key("delivery"));
        assert!(symbols.messages.contains_key("email_input.label"));
        assert_eq!(
            symbols.messages["delivery"].docs,
            vec!["Displayed on the product delivery confirmation card."]
        );
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
    fn reports_unknown_schema_type() {
        let schema = parse_schema("paint(color: Color)\n").expect("schema parses");
        let (_symbols, diagnostics) = build_schema_symbols(&schema);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "unknown schema type `Color`");
    }

    #[test]
    fn stores_group_doc_comments() {
        let schema = parse_schema("/// Input fields\nemail { label() }\n").expect("schema parses");
        let (symbols, diagnostics) = build_schema_symbols(&schema);

        assert!(diagnostics.is_empty());
        assert_eq!(symbols.groups["email"].docs, vec!["Input fields"]);
    }
}
