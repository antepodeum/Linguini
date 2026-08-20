use crate::DiagnosticFormat;
use linguini_analyzer::{
    render_diagnostics_with_sources_and_color, Diagnostic, DiagnosticCategory, DiagnosticSeverity,
    DiagnosticSource, QuickFix, QuickFixAction, Replacement,
};
use linguini_syntax::{ParseError, SourceId, Span};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

use super::io::path_for_output;

const SARIF_SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";

#[derive(Debug, Clone)]
struct SourceDocument {
    path: String,
    source: String,
}

#[derive(Debug, Clone)]
struct DiagnosticBatch {
    fallback: SourceDocument,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub(crate) struct ProjectDiagnostics {
    sources: BTreeMap<SourceId, SourceDocument>,
    batches: Vec<DiagnosticBatch>,
}

impl ProjectDiagnostics {
    pub(crate) fn register_source(
        &mut self,
        source_id: SourceId,
        root: &Path,
        path: &Path,
        source: &str,
    ) {
        self.sources.insert(
            source_id,
            SourceDocument {
                path: path_for_output(root, path),
                source: source.to_owned(),
            },
        );
    }

    pub(crate) fn push(
        &mut self,
        root: &Path,
        path: &Path,
        source: &str,
        diagnostics: &[Diagnostic],
    ) {
        if diagnostics.is_empty() {
            return;
        }
        self.batches.push(DiagnosticBatch {
            fallback: SourceDocument {
                path: path_for_output(root, path),
                source: source.to_owned(),
            },
            diagnostics: diagnostics.to_vec(),
        });
    }

    pub(crate) fn push_parse_errors(
        &mut self,
        root: &Path,
        path: &Path,
        source: &str,
        note: &str,
        errors: Vec<ParseError>,
    ) {
        let diagnostics = errors
            .into_iter()
            .map(|error| {
                Diagnostic::error(error.message, error.span)
                    .with_code("linguini.syntax")
                    .with_category(DiagnosticCategory::Syntax)
                    .with_note(note)
            })
            .collect::<Vec<_>>();
        self.push(root, path, source, &diagnostics);
    }

    pub(crate) fn has_errors(&self) -> bool {
        self.diagnostics()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }

    pub(crate) fn has_non_errors(&self) -> bool {
        self.diagnostics()
            .any(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error)
    }

    pub(crate) fn render_human(&self, errors: bool) -> String {
        let mut output = String::new();
        for batch in &self.batches {
            let diagnostics = batch
                .diagnostics
                .iter()
                .filter(|diagnostic| (diagnostic.severity == DiagnosticSeverity::Error) == errors)
                .cloned()
                .collect::<Vec<_>>();
            if diagnostics.is_empty() {
                continue;
            }
            let sources = self
                .sources
                .iter()
                .map(|(source_id, document)| DiagnosticSource {
                    source_id: *source_id,
                    path: &document.path,
                    source: &document.source,
                })
                .collect::<Vec<_>>();
            output.push_str(
                &render_diagnostics_with_sources_and_color(
                    &batch.fallback.path,
                    &batch.fallback.source,
                    &sources,
                    &diagnostics,
                    false,
                )
                .unwrap_or_else(|error| {
                    format!(
                        "failed to render diagnostics for {}: {error}",
                        batch.fallback.path
                    )
                }),
            );
        }
        output
    }

    pub(crate) fn render_machine(
        &self,
        format: DiagnosticFormat,
        failed: bool,
    ) -> Result<String, serde_json::Error> {
        let value = match format {
            DiagnosticFormat::Human => unreachable!("human diagnostics use the text renderer"),
            DiagnosticFormat::Json => self.json_document(failed),
            DiagnosticFormat::Sarif => self.sarif_document(failed),
        };
        let mut output = serde_json::to_string_pretty(&value)?;
        output.push('\n');
        Ok(output)
    }

    fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.batches
            .iter()
            .flat_map(|batch| batch.diagnostics.iter())
    }

    fn json_document(&self, failed: bool) -> Value {
        let diagnostics = self
            .batches
            .iter()
            .flat_map(|batch| {
                batch
                    .diagnostics
                    .iter()
                    .map(|diagnostic| self.json_diagnostic(diagnostic, &batch.fallback))
            })
            .collect::<Vec<_>>();
        let mut errors = 0;
        let mut warnings = 0;
        let mut advice = 0;
        for diagnostic in self.diagnostics() {
            match diagnostic.severity {
                DiagnosticSeverity::Error => errors += 1,
                DiagnosticSeverity::Warning => warnings += 1,
                DiagnosticSeverity::Advice => advice += 1,
            }
        }

        json!({
            "version": 1,
            "tool": {
                "name": "linguini",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "success": !failed,
            "summary": {
                "errors": errors,
                "warnings": warnings,
                "advice": advice,
            },
            "diagnostics": diagnostics,
        })
    }

    fn json_diagnostic(&self, diagnostic: &Diagnostic, fallback: &SourceDocument) -> Value {
        let primary = diagnostic
            .source_span
            .map(|span| (self.document_for_span(span, fallback), span));
        let primary_document = primary.map_or(fallback, |(document, _)| document);
        let related = diagnostic
            .related
            .iter()
            .map(|related| {
                let document = self.document_for_span(related.span, fallback);
                json!({
                    "message": related.message,
                    "path": document.path,
                    "range": json_range(&document.source, related.span),
                })
            })
            .collect::<Vec<_>>();
        let fixes = diagnostic
            .quick_fixes
            .iter()
            .map(|fix| self.json_quick_fix(fix, fallback))
            .collect::<Vec<_>>();

        let mut value = Map::new();
        value.insert("code".to_owned(), json!(diagnostic.code));
        value.insert(
            "category".to_owned(),
            json!(category_name(diagnostic.category)),
        );
        if let Some(lint_name) = diagnostic.lint_name {
            value.insert("lintName".to_owned(), json!(lint_name));
        }
        value.insert(
            "severity".to_owned(),
            json!(severity_name(diagnostic.severity)),
        );
        value.insert("message".to_owned(), json!(diagnostic.message));
        value.insert("path".to_owned(), json!(primary_document.path));
        if let Some((document, span)) = primary {
            value.insert("range".to_owned(), json_range(&document.source, span));
        }
        if let Some(note) = &diagnostic.note {
            value.insert("note".to_owned(), json!(note));
        }
        if !related.is_empty() {
            value.insert("related".to_owned(), Value::Array(related));
        }
        if !fixes.is_empty() {
            value.insert("fixes".to_owned(), Value::Array(fixes));
        }
        Value::Object(value)
    }

    fn json_quick_fix(&self, fix: &QuickFix, fallback: &SourceDocument) -> Value {
        let mut value = Map::new();
        value.insert("title".to_owned(), json!(fix.title));
        match &fix.action {
            QuickFixAction::Hint => {
                value.insert("kind".to_owned(), json!("hint"));
            }
            QuickFixAction::Command { id } => {
                value.insert("kind".to_owned(), json!("command"));
                value.insert("id".to_owned(), json!(id));
            }
            QuickFixAction::Replace { id, replacement } => {
                let document = self.document_for_span(replacement.span, fallback);
                value.insert("kind".to_owned(), json!("replace"));
                if let Some(id) = id {
                    value.insert("id".to_owned(), json!(id));
                }
                value.insert("path".to_owned(), json!(document.path));
                value.insert(
                    "range".to_owned(),
                    json_range(&document.source, replacement.span),
                );
                value.insert("replacement".to_owned(), json!(replacement.text));
            }
        }
        Value::Object(value)
    }

    fn sarif_document(&self, failed: bool) -> Value {
        let mut rules_by_code = BTreeMap::<&str, &Diagnostic>::new();
        for diagnostic in self.diagnostics() {
            rules_by_code.entry(diagnostic.code).or_insert(diagnostic);
        }
        let rule_indices = rules_by_code
            .keys()
            .enumerate()
            .map(|(index, code)| (*code, index))
            .collect::<BTreeMap<_, _>>();
        let rules = rules_by_code
            .values()
            .map(|diagnostic| {
                json!({
                    "id": diagnostic.code,
                    "name": diagnostic.code,
                    "shortDescription": {
                        "text": diagnostic.message,
                    },
                    "defaultConfiguration": {
                        "level": sarif_level(diagnostic.severity),
                    },
                    "properties": {
                        "category": category_name(diagnostic.category),
                        "lintName": diagnostic.lint_name,
                    },
                })
            })
            .collect::<Vec<_>>();
        let results =
            self.batches
                .iter()
                .flat_map(|batch| {
                    batch.diagnostics.iter().map(|diagnostic| {
                        self.sarif_result(diagnostic, &batch.fallback, &rule_indices)
                    })
                })
                .collect::<Vec<_>>();

        json!({
            "$schema": SARIF_SCHEMA,
            "version": "2.1.0",
            "runs": [{
                "columnKind": "unicodeCodePoints",
                "tool": {
                    "driver": {
                        "name": "Linguini",
                        "semanticVersion": env!("CARGO_PKG_VERSION"),
                        "informationUri": env!("CARGO_PKG_REPOSITORY"),
                        "rules": rules,
                    },
                },
                "invocations": [{
                    "executionSuccessful": !failed,
                }],
                "results": results,
            }],
        })
    }

    fn sarif_result(
        &self,
        diagnostic: &Diagnostic,
        fallback: &SourceDocument,
        rule_indices: &BTreeMap<&str, usize>,
    ) -> Value {
        let primary = diagnostic
            .source_span
            .map(|span| (self.document_for_span(span, fallback), span));
        let location = match primary {
            Some((document, span)) => sarif_location(document, Some(span)),
            None => sarif_location(fallback, None),
        };
        let related_locations = diagnostic
            .related
            .iter()
            .enumerate()
            .map(|(index, related)| {
                let document = self.document_for_span(related.span, fallback);
                let mut location = sarif_location(document, Some(related.span));
                let object = location
                    .as_object_mut()
                    .expect("SARIF location is an object");
                object.insert("id".to_owned(), json!(index + 1));
                object.insert("message".to_owned(), json!({ "text": related.message }));
                location
            })
            .collect::<Vec<_>>();
        let fixes = diagnostic
            .quick_fixes
            .iter()
            .filter_map(|fix| self.sarif_fix(fix, fallback))
            .collect::<Vec<_>>();
        let quick_fixes = diagnostic
            .quick_fixes
            .iter()
            .map(|fix| self.json_quick_fix(fix, fallback))
            .collect::<Vec<_>>();

        let mut properties = Map::new();
        properties.insert(
            "category".to_owned(),
            json!(category_name(diagnostic.category)),
        );
        if let Some(lint_name) = diagnostic.lint_name {
            properties.insert("lintName".to_owned(), json!(lint_name));
        }
        if let Some(note) = &diagnostic.note {
            properties.insert("note".to_owned(), json!(note));
        }
        if !quick_fixes.is_empty() {
            properties.insert("quickFixes".to_owned(), Value::Array(quick_fixes));
        }

        let mut result = Map::new();
        result.insert("ruleId".to_owned(), json!(diagnostic.code));
        result.insert("ruleIndex".to_owned(), json!(rule_indices[diagnostic.code]));
        result.insert("level".to_owned(), json!(sarif_level(diagnostic.severity)));
        result.insert("message".to_owned(), json!({ "text": diagnostic.message }));
        result.insert("locations".to_owned(), Value::Array(vec![location]));
        if !related_locations.is_empty() {
            result.insert(
                "relatedLocations".to_owned(),
                Value::Array(related_locations),
            );
        }
        if !fixes.is_empty() {
            result.insert("fixes".to_owned(), Value::Array(fixes));
        }
        if !properties.is_empty() {
            result.insert("properties".to_owned(), Value::Object(properties));
        }
        Value::Object(result)
    }

    fn sarif_fix(&self, fix: &QuickFix, fallback: &SourceDocument) -> Option<Value> {
        let QuickFixAction::Replace { id, replacement } = &fix.action else {
            return None;
        };
        let document = self.document_for_span(replacement.span, fallback);
        let mut value = Map::new();
        value.insert("description".to_owned(), json!({ "text": fix.title }));
        value.insert(
            "artifactChanges".to_owned(),
            Value::Array(vec![sarif_artifact_change(document, replacement)]),
        );
        if let Some(id) = id {
            value.insert("properties".to_owned(), json!({ "id": id }));
        }
        Some(Value::Object(value))
    }

    fn document_for_span<'a>(
        &'a self,
        span: Span,
        fallback: &'a SourceDocument,
    ) -> &'a SourceDocument {
        self.sources.get(&span.source).unwrap_or(fallback)
    }
}

fn json_range(source: &str, span: Span) -> Value {
    let range = source_range(source, span);
    json!({
        "start": {
            "line": range.start.line,
            "column": range.start.column,
            "byteOffset": range.start.offset,
        },
        "end": {
            "line": range.end.line,
            "column": range.end.column,
            "byteOffset": range.end.offset,
        },
        "byteLength": range.end.offset - range.start.offset,
    })
}

fn sarif_location(document: &SourceDocument, span: Option<Span>) -> Value {
    let mut physical = Map::new();
    physical.insert(
        "artifactLocation".to_owned(),
        json!({ "uri": artifact_uri(&document.path) }),
    );
    if let Some(span) = span {
        physical.insert("region".to_owned(), sarif_region(&document.source, span));
    }
    json!({
        "physicalLocation": Value::Object(physical),
    })
}

fn sarif_artifact_change(document: &SourceDocument, replacement: &Replacement) -> Value {
    json!({
        "artifactLocation": {
            "uri": artifact_uri(&document.path),
        },
        "replacements": [{
            "deletedRegion": sarif_region(&document.source, replacement.span),
            "insertedContent": {
                "text": replacement.text,
            },
        }],
    })
}

fn sarif_region(source: &str, span: Span) -> Value {
    let range = source_range(source, span);
    json!({
        "startLine": range.start.line,
        "startColumn": range.start.column,
        "endLine": range.end.line,
        "endColumn": range.end.column,
        "charOffset": source[..range.start.offset].chars().count(),
        "charLength": source[range.start.offset..range.end.offset].chars().count(),
        "byteOffset": range.start.offset,
        "byteLength": range.end.offset - range.start.offset,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourcePosition {
    line: usize,
    column: usize,
    offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceRange {
    start: SourcePosition,
    end: SourcePosition,
}

fn source_range(source: &str, span: Span) -> SourceRange {
    let start = clamp_char_boundary(source, span.start);
    let end = clamp_char_boundary(source, span.end.max(start));
    SourceRange {
        start: source_position(source, start),
        end: source_position(source, end),
    }
}

fn clamp_char_boundary(source: &str, offset: usize) -> usize {
    let mut offset = offset.min(source.len());
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn source_position(source: &str, offset: usize) -> SourcePosition {
    let prefix = &source[..offset];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    SourcePosition {
        line: prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
        column: source[line_start..offset].chars().count() + 1,
        offset,
    }
}

fn artifact_uri(path: &str) -> String {
    let mut uri = String::new();
    for byte in path.replace('\\', "/").bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            uri.push(char::from(byte));
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
}

fn category_name(category: DiagnosticCategory) -> &'static str {
    match category {
        DiagnosticCategory::Syntax => "syntax",
        DiagnosticCategory::Semantic => "semantic",
        DiagnosticCategory::Lint => "lint",
        DiagnosticCategory::Project => "project",
    }
}

fn severity_name(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "error",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Advice => "advice",
    }
}

fn sarif_level(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "error",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Advice => "note",
    }
}

#[cfg(test)]
mod tests {
    use super::{artifact_uri, source_range, ProjectDiagnostics};
    use linguini_analyzer::Diagnostic;
    use linguini_syntax::{SourceId, Span};
    use std::path::Path;

    #[test]
    fn human_renderer_uses_paths_for_cross_file_related_spans() {
        let root = Path::new("/project");
        let first_path = root.join("schema/first.lgs");
        let second_path = root.join("schema/second.lgs");
        let mut diagnostics = ProjectDiagnostics::default();
        diagnostics.register_source(SourceId(1), root, &first_path, "hello\n");
        diagnostics.register_source(SourceId(3), root, &second_path, "hello\n");
        diagnostics.push(
            root,
            &second_path,
            "hello\n",
            &[Diagnostic::error(
                "duplicate schema declaration `hello`",
                Span::in_source(SourceId(3), 0, 5),
            )
            .with_related(
                Span::in_source(SourceId(1), 0, 5),
                "first declaration is here",
            )],
        );

        let rendered = diagnostics.render_human(true);

        assert!(rendered.contains("schema/second.lgs"));
        assert!(rendered.contains("schema/first.lgs"));
        assert!(rendered.contains("first declaration is here"));
        assert!(!rendered.contains("related source #"));
    }

    #[test]
    fn ranges_use_one_based_unicode_positions_and_byte_offsets() {
        let source = "aé\nβ";
        let range = source_range(source, Span::new(1, 3));

        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.column, 2);
        assert_eq!(range.start.offset, 1);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.column, 3);
        assert_eq!(range.end.offset, 3);
    }

    #[test]
    fn artifact_uris_escape_non_uri_bytes() {
        assert_eq!(
            artifact_uri("locales/hello world/ru л.lgl"),
            "locales/hello%20world/ru%20%D0%BB.lgl"
        );
    }
}
