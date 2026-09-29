use linguini_syntax::{SourceId, Span};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmaScriptTarget {
    TypeScript,
    JavaScript,
}

impl EcmaScriptTarget {
    pub fn is_typescript(self) -> bool {
        matches!(self, Self::TypeScript)
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
    target: EcmaScriptTarget,
    path: String,
    declaration_path: Option<String>,
}

impl EcmaModuleOutput {
    pub fn new(
        target: EcmaScriptTarget,
        path: impl Into<String>,
        declaration_path: Option<String>,
    ) -> Self {
        let path = path.into();
        match target {
            EcmaScriptTarget::TypeScript => assert!(
                path.ends_with(".ts"),
                "TypeScript ECMAScript output must end in .ts"
            ),
            EcmaScriptTarget::JavaScript => assert!(
                path.ends_with(".js"),
                "JavaScript ECMAScript output must end in .js"
            ),
        }
        assert!(
            declaration_path
                .as_deref()
                .map_or(true, |path| path.ends_with(".d.ts")),
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

    pub fn target(&self) -> EcmaScriptTarget {
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
    statements: Vec<EcmaStatement>,
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
        if !self.output.target.is_typescript() {
            if let EcmaImportBindings::NamedWithTypes { values, .. } = item.bindings {
                if values.is_empty() {
                    return;
                }
                item.bindings = EcmaImportBindings::Named(values);
            }
        }
        if self.output.target.is_typescript()
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
        if self.output.target.is_typescript()
            || !matches!(
                item.bindings,
                EcmaReExportBindings::TypeAll | EcmaReExportBindings::TypeNamed(_)
            )
        {
            self.re_exports.push(item);
        }
    }

    pub fn push_statement(&mut self, statement: EcmaStatement) {
        if self.output.target.is_typescript()
            || statement.kind != EcmaStatementKind::TypeDeclaration
        {
            self.statements.push(statement);
        }
    }

    /// Renders one ESM module without attaching a source-map trailer.
    ///
    /// Project generators use this during the structured-backend migration; source-mapped
    /// physical leaves call [`Self::render`] with their ordered source records.
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
            source_map: render_source_map(self.output.path(), sources, &mappings),
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
        module.output.target.is_typescript() || statement.kind != EcmaStatementKind::TypeDeclaration
    });
    if rendered_module_edge && renders_statement {
        output.push('\n');
    }

    for statement in &module.statements {
        if !module.output.target.is_typescript()
            && statement.kind == EcmaStatementKind::TypeDeclaration
        {
            continue;
        }
        let generated_line = output.bytes().filter(|byte| *byte == b'\n').count();
        if let Some(span) = statement.source_span {
            record_mapping((generated_line, span));
        }
        output.push_str(statement.code.trim_end_matches('\n'));
        output.push('\n');
    }
}

fn render_import(item: &EcmaImport, target: EcmaScriptTarget, output: &mut String) -> bool {
    if !target.is_typescript() && matches!(item.bindings, EcmaImportBindings::TypeNamed(_)) {
        return false;
    }
    output.push_str("import ");
    match &item.bindings {
        EcmaImportBindings::SideEffect => {
            output.push_str(&json_string(&item.specifier));
            output.push_str(";\n");
            return true;
        }
        EcmaImportBindings::Default(local) => output.push_str(local),
        EcmaImportBindings::Namespace(local) => {
            output.push_str("* as ");
            output.push_str(local);
        }
        EcmaImportBindings::Named(bindings) => {
            output.push_str("{ ");
            for (index, binding) in bindings.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str(&binding.imported);
                if binding.local != binding.imported {
                    output.push_str(" as ");
                    output.push_str(&binding.local);
                }
            }
            output.push_str(" }");
        }
        EcmaImportBindings::TypeNamed(bindings) => {
            output.push_str("type { ");
            for (index, binding) in bindings.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str(&binding.imported);
                if binding.local != binding.imported {
                    output.push_str(" as ");
                    output.push_str(&binding.local);
                }
            }
            output.push_str(" }");
        }
        EcmaImportBindings::NamedWithTypes { values, types } => {
            output.push_str("{ ");
            for (index, (binding, type_only)) in values
                .iter()
                .map(|binding| (binding, false))
                .chain(types.iter().map(|binding| (binding, true)))
                .enumerate()
            {
                if index > 0 {
                    output.push_str(", ");
                }
                if type_only {
                    output.push_str("type ");
                }
                output.push_str(&binding.imported);
                if binding.local != binding.imported {
                    output.push_str(" as ");
                    output.push_str(&binding.local);
                }
            }
            output.push_str(" }");
        }
    }
    output.push_str(" from ");
    output.push_str(&json_string(&item.specifier));
    output.push_str(";\n");
    true
}

fn render_re_export(item: &EcmaReExport, target: EcmaScriptTarget, output: &mut String) -> bool {
    if !target.is_typescript()
        && matches!(
            item.bindings,
            EcmaReExportBindings::TypeAll | EcmaReExportBindings::TypeNamed(_)
        )
    {
        return false;
    }
    output.push_str("export ");
    match &item.bindings {
        EcmaReExportBindings::All => output.push('*'),
        EcmaReExportBindings::TypeAll => output.push_str("type *"),
        EcmaReExportBindings::Named(bindings) | EcmaReExportBindings::TypeNamed(bindings) => {
            if matches!(item.bindings, EcmaReExportBindings::TypeNamed(_)) {
                output.push_str("type ");
            }
            output.push_str("{ ");
            for (index, binding) in bindings.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str(&binding.imported);
                if binding.local != binding.imported {
                    output.push_str(" as ");
                    output.push_str(&binding.local);
                }
            }
            output.push_str(" }");
        }
    }
    output.push_str(" from ");
    output.push_str(&json_string(&item.specifier));
    output.push_str(";\n");
    true
}

fn render_source_map(
    file_name: &str,
    sources: &[EcmaSource],
    mappings: &[(usize, Span)],
) -> String {
    let source_indexes = sources
        .iter()
        .enumerate()
        .map(|(index, source)| (source.id, index))
        .collect::<BTreeMap<_, _>>();
    let last_generated_line = mappings
        .iter()
        .map(|(line, _)| *line)
        .max()
        .unwrap_or_default();
    let mut encoded = String::new();
    let mut mapping_index = 0;
    let mut previous_source = 0_i64;
    let mut previous_original_line = 0_i64;
    let mut previous_original_column = 0_i64;

    for generated_line in 0..=last_generated_line {
        if generated_line > 0 {
            encoded.push(';');
        }
        while let Some((line, span)) = mappings.get(mapping_index) {
            if *line != generated_line {
                break;
            }
            let Some(source_index) = source_indexes.get(&span.source).copied() else {
                mapping_index += 1;
                continue;
            };
            let Some((original_line, original_column)) =
                source_position(&sources[source_index], *span)
            else {
                mapping_index += 1;
                continue;
            };
            encode_vlq(0, &mut encoded);
            encode_vlq(source_index as i64 - previous_source, &mut encoded);
            encode_vlq(original_line as i64 - previous_original_line, &mut encoded);
            encode_vlq(
                original_column as i64 - previous_original_column,
                &mut encoded,
            );
            previous_source = source_index as i64;
            previous_original_line = original_line as i64;
            previous_original_column = original_column as i64;
            mapping_index += 1;
        }
    }

    let source_paths = sources
        .iter()
        .map(|source| json_string(&source.path))
        .collect::<Vec<_>>()
        .join(",");
    let source_contents = sources
        .iter()
        .map(|source| json_string(&source.contents))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"version\":3,\"file\":{},\"sources\":[{}],\"sourcesContent\":[{}],\"names\":[],\"mappings\":{}}}\n",
        json_string(file_name),
        source_paths,
        source_contents,
        json_string(&encoded)
    )
}

fn source_position(source: &EcmaSource, span: Span) -> Option<(usize, usize)> {
    if span.source != source.id || span.start > source.contents.len() {
        return None;
    }
    let prefix = source.contents.get(..span.start)?;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let column = source
        .contents
        .get(line_start..span.start)?
        .encode_utf16()
        .count();
    Some((line, column))
}

fn encode_vlq(value: i64, output: &mut String) {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = if value < 0 {
        ((-value) as u64) << 1 | 1
    } else {
        (value as u64) << 1
    };
    loop {
        let mut digit = (value & 31) as usize;
        value >>= 5;
        if value != 0 {
            digit |= 32;
        }
        output.push(BASE64[digit] as char);
        if value == 0 {
            break;
        }
    }
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000C}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{0000}'..='\u{001F}' | '\u{007F}' => {
                use std::fmt::Write;
                write!(output, "\\u{:04X}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            '\u{2028}' => output.push_str("\\u2028"),
            '\u{2029}' => output.push_str("\\u2029"),
            _ => output.push(character),
        }
    }
    output.push('\"');
    output
}

#[cfg(test)]
mod tests {
    use super::{
        EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
        EcmaReExport, EcmaScriptTarget, EcmaSource, EcmaStatement,
    };
    use linguini_syntax::{SourceId, Span};

    #[test]
    fn mixed_imports_keep_values_and_drop_type_only_dependencies_in_javascript() {
        for target in [EcmaScriptTarget::TypeScript, EcmaScriptTarget::JavaScript] {
            let path = if target.is_typescript() {
                "entry.ts"
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
            if target.is_typescript() {
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
            EcmaScriptTarget::JavaScript,
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
            EcmaScriptTarget::JavaScript,
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
            EcmaScriptTarget::JavaScript,
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
