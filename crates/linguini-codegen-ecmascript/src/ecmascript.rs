mod edges;
mod literals;
mod source_map;
use edges::{render_import, render_re_export};
#[cfg(test)]
pub(crate) use literals::escape_string;
pub(crate) use literals::string_literal;

use linguini_syntax::{SourceId, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmaModuleKind {
    Declaration,
    JavaScript,
}

impl EcmaModuleKind {
    pub fn is_declaration(self) -> bool {
        matches!(self, Self::Declaration)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaSource {
    pub id: SourceId,
    pub path: String,
    pub contents: String,
}

impl EcmaSource {
    pub fn new(id: SourceId, path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            contents: contents.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EcmaImportBindings {
    SideEffect,
    Default(String),
    Namespace(String),
    Named(Vec<EcmaNamedImport>),
    TypeNamed(Vec<EcmaNamedImport>),
    NamedWithTypes {
        values: Vec<EcmaNamedImport>,
        types: Vec<EcmaNamedImport>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaNamedImport {
    pub imported: String,
    pub local: String,
}

impl EcmaNamedImport {
    pub fn new(imported: impl Into<String>, local: impl Into<String>) -> Self {
        Self {
            imported: imported.into(),
            local: local.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaImport {
    pub specifier: String,
    pub bindings: EcmaImportBindings,
}

impl EcmaImport {
    pub fn named(specifier: impl Into<String>, bindings: Vec<EcmaNamedImport>) -> Self {
        Self {
            specifier: specifier.into(),
            bindings: EcmaImportBindings::Named(bindings),
        }
    }

    pub fn named_with_types(
        specifier: impl Into<String>,
        values: Vec<EcmaNamedImport>,
        types: Vec<EcmaNamedImport>,
    ) -> Self {
        Self {
            specifier: specifier.into(),
            bindings: EcmaImportBindings::NamedWithTypes { values, types },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EcmaReExportBindings {
    All,
    TypeAll,
    Named(Vec<EcmaNamedImport>),
    TypeNamed(Vec<EcmaNamedImport>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaReExport {
    pub specifier: String,
    pub bindings: EcmaReExportBindings,
}

impl EcmaReExport {
    pub fn all(specifier: impl Into<String>) -> Self {
        Self {
            specifier: specifier.into(),
            bindings: EcmaReExportBindings::All,
        }
    }

    pub fn type_all(specifier: impl Into<String>) -> Self {
        Self {
            specifier: specifier.into(),
            bindings: EcmaReExportBindings::TypeAll,
        }
    }

    pub fn named(specifier: impl Into<String>, bindings: Vec<EcmaNamedImport>) -> Self {
        Self {
            specifier: specifier.into(),
            bindings: EcmaReExportBindings::Named(bindings),
        }
    }

    pub fn type_named(specifier: impl Into<String>, bindings: Vec<EcmaNamedImport>) -> Self {
        Self {
            specifier: specifier.into(),
            bindings: EcmaReExportBindings::TypeNamed(bindings),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmaStatementKind {
    Documentation,
    TypeDeclaration,
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaStatement {
    pub code: String,
    pub source_span: Option<Span>,
    pub kind: EcmaStatementKind,
}

impl EcmaStatement {
    pub fn generated(code: impl Into<String>) -> Self {
        Self::value(code, None)
    }

    pub fn documentation(code: impl Into<String>, source_span: Option<Span>) -> Self {
        Self {
            code: code.into(),
            source_span,
            kind: EcmaStatementKind::Documentation,
        }
    }

    pub fn type_declaration(code: impl Into<String>, source_span: Option<Span>) -> Self {
        Self {
            code: code.into(),
            source_span,
            kind: EcmaStatementKind::TypeDeclaration,
        }
    }

    pub fn value(code: impl Into<String>, source_span: Option<Span>) -> Self {
        Self {
            code: code.into(),
            source_span,
            kind: EcmaStatementKind::Value,
        }
    }

    pub fn mapped(code: impl Into<String>, source_span: Span) -> Self {
        Self::value(code, Some(source_span))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaModuleOutput {
    target: EcmaModuleKind,
    path: String,
    declaration_path: Option<String>,
}

impl EcmaModuleOutput {
    pub fn new(
        target: EcmaModuleKind,
        path: impl Into<String>,
        declaration_path: Option<String>,
    ) -> Self {
        let path = path.into();
        match target {
            EcmaModuleKind::Declaration => assert!(
                path.ends_with(".d.ts"),
                "declaration output must end in .d.ts"
            ),
            EcmaModuleKind::JavaScript => assert!(
                path.ends_with(".js"),
                "JavaScript ECMAScript output must end in .js"
            ),
        }
        assert!(
            declaration_path
                .as_deref()
                .is_none_or(|path| path.ends_with(".d.ts")),
            "ECMAScript declaration companion must end in .d.ts"
        );
        Self {
            target,
            path,
            declaration_path,
        }
    }

    pub fn source_map_path(&self) -> String {
        format!("{}.map", self.path)
    }

    pub fn target(&self) -> EcmaModuleKind {
        self.target
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn declaration_path(&self) -> Option<&str> {
        self.declaration_path.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaModule {
    output: EcmaModuleOutput,
    imports: Vec<EcmaImport>,
    re_exports: Vec<EcmaReExport>,
    statements: Vec<ModuleStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleStatement {
    statement: EcmaStatement,
    line_mappings: Vec<(usize, Span)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedEcmaModule {
    pub output: EcmaModuleOutput,
    pub code: String,
    pub source_map: String,
}

impl EcmaModule {
    pub fn new(output: EcmaModuleOutput) -> Self {
        Self {
            output,
            imports: Vec::new(),
            re_exports: Vec::new(),
            statements: Vec::new(),
        }
    }

    pub fn output(&self) -> &EcmaModuleOutput {
        &self.output
    }

    pub fn push_import(&mut self, mut item: EcmaImport) {
        if !self.output.target.is_declaration() {
            if let EcmaImportBindings::NamedWithTypes { values, .. } = item.bindings {
                if values.is_empty() {
                    return;
                }
                item.bindings = EcmaImportBindings::Named(values);
            }
        }
        if self.output.target.is_declaration()
            || !matches!(item.bindings, EcmaImportBindings::TypeNamed(_))
        {
            self.imports.push(item);
        }
    }

    pub fn extend_imports(&mut self, items: impl IntoIterator<Item = EcmaImport>) {
        for item in items {
            self.push_import(item);
        }
    }

    pub fn push_re_export(&mut self, item: EcmaReExport) {
        if self.output.target.is_declaration()
            || !matches!(
                item.bindings,
                EcmaReExportBindings::TypeAll | EcmaReExportBindings::TypeNamed(_)
            )
        {
            self.re_exports.push(item);
        }
    }

    pub fn push_statement(&mut self, statement: EcmaStatement) {
        self.push_statement_with_mappings(statement, Vec::new());
    }

    pub(crate) fn push_statement_with_mappings(
        &mut self,
        statement: EcmaStatement,
        line_mappings: Vec<(usize, Span)>,
    ) {
        if self.output.target.is_declaration()
            || statement.kind != EcmaStatementKind::TypeDeclaration
        {
            self.statements.push(ModuleStatement {
                statement,
                line_mappings,
            });
        }
    }

    pub(crate) fn source_ids(&self) -> Vec<SourceId> {
        self.statements
            .iter()
            .flat_map(|item| {
                item.statement
                    .source_span
                    .iter()
                    .chain(item.line_mappings.iter().map(|(_, span)| span))
            })
            .map(|span| span.source)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Renders one ESM module without attaching a source-map trailer.
    ///
    /// Project generators use this for source-only output. Source-mapped artifacts call
    /// [`Self::render`] with their ordered source records.
    pub fn render_code(&self) -> String {
        let mut code = String::new();
        render_module_body(self, &mut code, |_| {});
        code
    }

    pub fn render(&self, sources: &[EcmaSource]) -> RenderedEcmaModule {
        let mut code = String::new();
        let mut mappings = Vec::new();
        render_module_body(self, &mut code, |mapping| mappings.push(mapping));

        let map_name = self.output.source_map_path();
        code.push_str("//# sourceMappingURL=");
        code.push_str(map_name.rsplit('/').next().unwrap_or(&map_name));
        code.push('\n');

        RenderedEcmaModule {
            output: self.output.clone(),
            code,
            source_map: source_map::render(self.output.path(), sources, &mappings),
        }
    }
}

fn render_module_body(
    module: &EcmaModule,
    output: &mut String,
    mut record_mapping: impl FnMut((usize, Span)),
) {
    let mut rendered_module_edge = false;
    for item in &module.imports {
        rendered_module_edge |= render_import(item, module.output.target, output);
    }
    for item in &module.re_exports {
        rendered_module_edge |= render_re_export(item, module.output.target, output);
    }
    let renders_statement = module.statements.iter().any(|statement| {
        module.output.target.is_declaration()
            || statement.statement.kind != EcmaStatementKind::TypeDeclaration
    });
    if rendered_module_edge && renders_statement {
        output.push('\n');
    }

    let mut generated_line = output.bytes().filter(|byte| *byte == b'\n').count();
    for item in &module.statements {
        let statement = &item.statement;
        if !module.output.target.is_declaration()
            && statement.kind == EcmaStatementKind::TypeDeclaration
        {
            continue;
        }
        if let Some(span) = statement.source_span {
            record_mapping((generated_line, span));
        }
        for (offset, span) in &item.line_mappings {
            record_mapping((generated_line + offset, *span));
        }
        let code = statement.code.trim_end_matches('\n');
        generated_line += code.bytes().filter(|byte| *byte == b'\n').count() + 1;
        output.push_str(code);
        output.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput,
        EcmaNamedImport, EcmaReExport, EcmaSource, EcmaStatement,
    };
    use linguini_syntax::{SourceId, Span};

    #[test]
    fn module_edges_link_values_and_erase_types_without_changing_initialization() {
        let mut files = vec![crate::EcmaGeneratedFile {
            path: "dependency.js".to_owned(),
            contents: "globalThis.initializations = (globalThis.initializations ?? 0) + 1; export default 'default'; export const named = 'named', unchanged = 'unchanged';".to_owned(),
        }];
        for target in [EcmaModuleKind::Declaration, EcmaModuleKind::JavaScript] {
            let path = if target.is_declaration() {
                "entry.d.ts"
            } else {
                "entry.js"
            };
            let mut module = EcmaModule::new(EcmaModuleOutput::new(target, path, None));
            for bindings in [
                EcmaImportBindings::SideEffect,
                EcmaImportBindings::Default("selected".to_owned()),
                EcmaImportBindings::Namespace("namespace".to_owned()),
                EcmaImportBindings::Named(vec![
                    EcmaNamedImport::new("named", "alias"),
                    EcmaNamedImport::new("unchanged", "unchanged"),
                ]),
                EcmaImportBindings::Named(vec![]),
            ] {
                module.push_import(EcmaImport {
                    specifier: "./dependency.js".to_owned(),
                    bindings,
                });
            }
            module.push_import(EcmaImport {
                specifier: "data:text/javascript,export default \"Café 😀\"".to_owned(),
                bindings: EcmaImportBindings::Default("escaped".to_owned()),
            });
            module.push_import(EcmaImport::named_with_types(
                "./dependency.js",
                vec![EcmaNamedImport::new("named", "mixedAlias")],
                vec![EcmaNamedImport::new("MissingType", "RenamedType")],
            ));
            module.push_import(EcmaImport {
                specifier: "./missing.js".to_owned(),
                bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
                    "Missing",
                    "TypeAlias",
                )]),
            });
            module.push_re_export(EcmaReExport::all("./dependency.js"));
            module.push_re_export(EcmaReExport::named(
                "./dependency.js",
                vec![
                    EcmaNamedImport::new("default", "forwardedDefault"),
                    EcmaNamedImport::new("named", "forwarded"),
                    EcmaNamedImport::new("unchanged", "unchanged"),
                ],
            ));
            module.push_re_export(EcmaReExport::type_all("./missing.js"));
            module.push_re_export(EcmaReExport::type_named(
                "./missing.js",
                vec![EcmaNamedImport::new("Missing", "TypeAlias")],
            ));
            if target.is_declaration() {
                let code = module.render_code();
                assert!(
                    code.contains("import type { Missing as TypeAlias } from \"./missing.js\";")
                );
                assert!(code.contains("import { named as mixedAlias, type MissingType as RenamedType } from \"./dependency.js\";"));
                assert!(code.contains("export type * from \"./missing.js\";"));
                assert!(
                    code.contains("export type { Missing as TypeAlias } from \"./missing.js\";")
                );
            } else {
                module.push_statement(EcmaStatement::generated("export const observed = [selected, namespace.named, alias, unchanged, mixedAlias, escaped];"));
                let code = module.render_code();
                assert!(!code.contains("Missing"));
                assert!(!code.contains("./missing.js"));
                assert!(code.contains("import {  } from \"./dependency.js\";"));
                files.push(crate::EcmaGeneratedFile {
                    path: path.to_owned(),
                    contents: code,
                });
            }
        }
        crate::test_runtime::execute_project(
            &files,
            r#"
import assert from 'node:assert/strict';
const entry = await import('./entry.js');
assert.deepEqual(entry.observed, ['default', 'named', 'named', 'unchanged', 'named', 'Café 😀']);
assert.equal(entry.forwardedDefault, 'default');
assert.equal(entry.forwarded, 'named');
assert.equal(entry.named, 'named');
assert.equal(entry.unchanged, 'unchanged');
assert.equal(globalThis.initializations, 1);
"#,
        );
    }

    #[test]
    fn mixed_imports_keep_values_and_drop_type_only_dependencies_in_javascript() {
        for target in [EcmaModuleKind::Declaration, EcmaModuleKind::JavaScript] {
            let path = if target.is_declaration() {
                "entry.d.ts"
            } else {
                "entry.js"
            };
            let mut module = EcmaModule::new(EcmaModuleOutput::new(target, path, None));
            module.push_import(EcmaImport::named_with_types(
                "./locale.js",
                vec![EcmaNamedImport::new("normalize", "normalizeLocale")],
                vec![EcmaNamedImport::new("Locale", "LocaleName")],
            ));
            module.push_import(EcmaImport::named_with_types(
                "./types.js",
                vec![],
                vec![EcmaNamedImport::new("OnlyType", "OnlyType")],
            ));
            let code = module.render_code();
            if target.is_declaration() {
                assert!(code.contains("import { normalize as normalizeLocale, type Locale as LocaleName } from \"./locale.js\";"));
                assert!(code.contains("import { type OnlyType } from \"./types.js\";"));
            } else {
                assert_eq!(
                    code,
                    "import { normalize as normalizeLocale } from \"./locale.js\";\n"
                );
            }
        }
    }

    #[test]
    fn renders_structured_esm_and_source_identity() {
        let source = EcmaSource::new(
            SourceId(7),
            "schema/main.lgs",
            "/// Café\nhello(name: String)\n",
        );
        let mut module = EcmaModule::new(EcmaModuleOutput::new(
            EcmaModuleKind::JavaScript,
            "messages/hello.js",
            None,
        ));
        module.push_import(EcmaImport::named(
            "./runtime.js",
            vec![EcmaNamedImport::new("select", "select")],
        ));
        module.push_statement(EcmaStatement::mapped(
            "export function hello(name) { return select(name); }",
            Span::in_source(SourceId(7), 10, 29),
        ));

        let rendered = module.render(&[source]);

        assert!(rendered
            .code
            .starts_with("import { select } from \"./runtime.js\";"));
        assert!(rendered
            .code
            .ends_with("//# sourceMappingURL=hello.js.map\n"));
        assert!(rendered
            .source_map
            .contains("\"sources\":[\"schema/main.lgs\"]"));
        assert!(rendered
            .source_map
            .contains("/// Café\\nhello(name: String)\\n"));
        assert!(!rendered.source_map.contains("\"mappings\":\";;\""));
    }

    #[test]
    fn renders_the_same_module_body_without_a_source_map_trailer() {
        let mut module = EcmaModule::new(EcmaModuleOutput::new(
            EcmaModuleKind::JavaScript,
            "messages/hello.js",
            None,
        ));
        module.push_import(EcmaImport::named(
            "./runtime.js",
            vec![EcmaNamedImport::new("format", "format")],
        ));
        module.push_statement(EcmaStatement::generated(
            "export const message = format(\"hello\");\n",
        ));

        assert_eq!(
            module.render_code(),
            "import { format } from \"./runtime.js\";\n\nexport const message = format(\"hello\");\n"
        );
        assert!(!module.render_code().contains("sourceMappingURL"));
    }

    #[test]
    fn target_owns_paths_and_excludes_types_from_javascript() {
        let mut module = EcmaModule::new(EcmaModuleOutput::new(
            EcmaModuleKind::JavaScript,
            "messages/hello.js",
            Some("messages/hello.d.ts".to_owned()),
        ));
        module.push_import(EcmaImport {
            specifier: "./shared.js".to_owned(),
            bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new("Name", "Name")]),
        });
        module.push_re_export(EcmaReExport::type_all("./shared.js"));
        module.push_statement(EcmaStatement::documentation("/** Public message. */", None));
        module.push_statement(EcmaStatement::type_declaration(
            "export type Hidden = string;",
            None,
        ));
        module.push_statement(EcmaStatement::generated(
            "export const message = \"hello\";",
        ));

        assert_eq!(module.output().source_map_path(), "messages/hello.js.map");
        assert_eq!(
            module.render_code(),
            "/** Public message. */\nexport const message = \"hello\";\n"
        );
    }
}
