use linguini_syntax::{SourceId, Span};
use std::collections::BTreeMap;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaStatement {
    pub code: String,
    pub source_span: Option<Span>,
}

impl EcmaStatement {
    pub fn generated(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            source_span: None,
        }
    }

    pub fn mapped(code: impl Into<String>, source_span: Span) -> Self {
        Self {
            code: code.into(),
            source_span: Some(source_span),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EcmaModule {
    pub imports: Vec<EcmaImport>,
    pub statements: Vec<EcmaStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedEcmaModule {
    pub code: String,
    pub source_map: String,
}

impl EcmaModule {
    /// Renders one ESM module without attaching a source-map trailer.
    ///
    /// Project generators use this during the structured-backend migration; source-mapped
    /// physical leaves call [`Self::render`] with their ordered source records.
    pub fn render_code(&self) -> String {
        let mut code = String::new();
        render_module_body(self, &mut code, |_| {});
        code
    }

    pub fn render(&self, file_name: &str, sources: &[EcmaSource]) -> RenderedEcmaModule {
        let mut code = String::new();
        let mut mappings = Vec::new();
        render_module_body(self, &mut code, |mapping| mappings.push(mapping));

        let map_name = format!("{file_name}.map");
        code.push_str("//# sourceMappingURL=");
        code.push_str(map_name.rsplit('/').next().unwrap_or(&map_name));
        code.push('\n');

        RenderedEcmaModule {
            code,
            source_map: render_source_map(file_name, sources, &mappings),
        }
    }
}

fn render_module_body(
    module: &EcmaModule,
    output: &mut String,
    mut record_mapping: impl FnMut((usize, Span)),
) {
    for item in &module.imports {
        render_import(item, output);
    }
    if !module.imports.is_empty() && !module.statements.is_empty() {
        output.push('\n');
    }

    for statement in &module.statements {
        let generated_line = output.bytes().filter(|byte| *byte == b'\n').count();
        if let Some(span) = statement.source_span {
            record_mapping((generated_line, span));
        }
        output.push_str(statement.code.trim_end_matches('\n'));
        output.push('\n');
    }
}

fn render_import(item: &EcmaImport, output: &mut String) {
    output.push_str("import ");
    match &item.bindings {
        EcmaImportBindings::SideEffect => {
            output.push_str(&json_string(&item.specifier));
            output.push_str(";\n");
            return;
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
    }
    output.push_str(" from ");
    output.push_str(&json_string(&item.specifier));
    output.push_str(";\n");
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
    use super::{EcmaImport, EcmaModule, EcmaNamedImport, EcmaSource, EcmaStatement};
    use linguini_syntax::{SourceId, Span};

    #[test]
    fn renders_structured_esm_and_source_identity() {
        let source = EcmaSource::new(
            SourceId(7),
            "schema/main.lgs",
            "/// Café\nhello(name: String)\n",
        );
        let module = EcmaModule {
            imports: vec![EcmaImport::named(
                "./runtime.js",
                vec![EcmaNamedImport::new("select", "select")],
            )],
            statements: vec![EcmaStatement::mapped(
                "export function hello(name) { return select(name); }",
                Span::in_source(SourceId(7), 10, 29),
            )],
        };

        let rendered = module.render("messages/hello.js", &[source]);

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
        let module = EcmaModule {
            imports: vec![EcmaImport::named(
                "./runtime.js",
                vec![EcmaNamedImport::new("format", "format")],
            )],
            statements: vec![EcmaStatement::generated(
                "export const message = format(\"hello\");\n",
            )],
        };

        assert_eq!(
            module.render_code(),
            "import { format } from \"./runtime.js\";\n\nexport const message = format(\"hello\");\n"
        );
        assert!(!module.render_code().contains("sourceMappingURL"));
    }
}
