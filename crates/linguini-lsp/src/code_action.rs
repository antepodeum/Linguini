use crate::LinguiniDocument;
use linguini_analyzer::{Diagnostic as AnalyzerDiagnostic, QuickFix};
use std::collections::{BTreeMap, HashMap};
use tower_lsp_server::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, DiagnosticRelatedInformation,
    DiagnosticSeverity, Location, NumberOrString, Position, Range, TextEdit, Url as Uri,
    WorkspaceEdit,
};

pub(crate) fn analyzer_quick_fix_actions(
    uri: &Uri,
    document: &LinguiniDocument,
    range: Range,
    diagnostics: impl IntoIterator<Item = AnalyzerDiagnostic>,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    let mut grouped_replacements: BTreeMap<String, Vec<TextEdit>> = BTreeMap::new();
    let mut all_replacements = Vec::new();
    for diagnostic in diagnostics {
        if !diagnostic_has_action_for_range(document, &diagnostic, range) {
            continue;
        }
        let lsp_diagnostic = to_lsp_diagnostic(document, &diagnostic);
        let mut first_replacement = None;
        for quick_fix in diagnostic.quick_fixes {
            if let Some(replacement) = &quick_fix.replacement {
                let edit = TextEdit {
                    range: to_range(document, replacement.span),
                    new_text: replacement.text.clone(),
                };
                grouped_replacements
                    .entry(quick_fix_group_title(&quick_fix.title))
                    .or_default()
                    .push(edit.clone());
                first_replacement.get_or_insert(edit);
            }
            if let Some(action) =
                quick_fix_code_action(uri, document, lsp_diagnostic.clone(), quick_fix)
            {
                actions.push(action);
            }
        }
        if let Some(edit) = first_replacement {
            all_replacements.push(edit);
        }
    }

    for (title, edits) in grouped_replacements {
        if edits.len() < 2 {
            continue;
        }
        if let Some(action) =
            workspace_edit_action(format!("Apply all {title} fixes in file"), uri, edits)
        {
            actions.push(action);
        }
    }

    if all_replacements.len() > 1 {
        if let Some(action) = workspace_edit_action(
            "Apply all Linguini quick fixes in file".to_owned(),
            uri,
            all_replacements,
        ) {
            actions.push(action);
        }
    }
    actions
}

pub(crate) fn to_lsp_diagnostic(
    document: &LinguiniDocument,
    diagnostic: &linguini_analyzer::Diagnostic,
) -> Diagnostic {
    to_lsp_diagnostic_with_workspace(document, diagnostic, &[])
}

pub(crate) fn to_lsp_diagnostic_with_workspace(
    document: &LinguiniDocument,
    diagnostic: &linguini_analyzer::Diagnostic,
    workspace: &[LinguiniDocument],
) -> Diagnostic {
    let related_information = diagnostic
        .related
        .iter()
        .filter_map(|related| {
            let target = if related.span.source == document.source_id {
                Some(document)
            } else {
                workspace
                    .iter()
                    .find(|candidate| candidate.source_id == related.span.source)
            }?;
            let uri = target.uri.parse::<Uri>().ok()?;
            Some(DiagnosticRelatedInformation {
                location: Location {
                    uri,
                    range: to_range(target, related.span),
                },
                message: related.message.clone(),
            })
        })
        .collect::<Vec<_>>();
    let message = diagnostic
        .note
        .as_ref()
        .map(|note| format!("{}\n\nNote: {note}", diagnostic.message))
        .unwrap_or_else(|| diagnostic.message.clone());
    let data = (!diagnostic.quick_fixes.is_empty()).then(|| {
        serde_json::json!({
            "quickFixes": diagnostic
                .quick_fixes
                .iter()
                .map(|quick_fix| serde_json::json!({
                    "id": quick_fix.id.as_deref(),
                    "title": &quick_fix.title,
                    "hasReplacement": quick_fix.replacement.is_some(),
                }))
                .collect::<Vec<_>>(),
        })
    });
    Diagnostic {
        range: to_range(document, diagnostic.span),
        severity: Some(match diagnostic.severity {
            linguini_analyzer::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
            linguini_analyzer::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
            linguini_analyzer::DiagnosticSeverity::Advice => DiagnosticSeverity::HINT,
        }),
        code: Some(NumberOrString::String(
            diagnostic_code(diagnostic).to_owned(),
        )),
        source: Some("linguini".to_owned()),
        message,
        related_information: (!related_information.is_empty()).then_some(related_information),
        data,
        ..Default::default()
    }
}

fn diagnostic_has_action_for_range(
    document: &LinguiniDocument,
    diagnostic: &linguini_analyzer::Diagnostic,
    range: Range,
) -> bool {
    if diagnostic.quick_fixes.is_empty() {
        return false;
    }
    if diagnostic.span.start == diagnostic.span.end {
        let point = to_range(document, diagnostic.span).start;
        return range.start <= point && point <= range.end;
    }
    ranges_overlap(to_range(document, diagnostic.span), range)
}

fn ranges_overlap(left: Range, right: Range) -> bool {
    if right.start == right.end {
        return left.start <= right.start && right.start < left.end;
    }
    if left.start == left.end {
        return right.start <= left.start && left.start < right.end;
    }
    left.start < right.end && right.start < left.end
}

fn quick_fix_code_action(
    uri: &Uri,
    document: &LinguiniDocument,
    diagnostic: Diagnostic,
    quick_fix: QuickFix,
) -> Option<CodeActionOrCommand> {
    let replacement = quick_fix.replacement?;
    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: to_range(document, replacement.span),
            new_text: replacement.text,
        }],
    );
    let edit = WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    };

    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: quick_fix.title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(edit),
        ..Default::default()
    }))
}

fn workspace_edit_action(
    title: String,
    uri: &Uri,
    edits: Vec<TextEdit>,
) -> Option<CodeActionOrCommand> {
    let edits = normalize_non_overlapping(edits)?;
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), edits);
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        ..Default::default()
    }))
}

fn normalize_non_overlapping(mut edits: Vec<TextEdit>) -> Option<Vec<TextEdit>> {
    edits.sort_by(|left, right| {
        (left.range.start, left.range.end, left.new_text.as_str()).cmp(&(
            right.range.start,
            right.range.end,
            right.new_text.as_str(),
        ))
    });
    edits.dedup();
    for pair in edits.windows(2) {
        let [left, right] = pair else {
            continue;
        };
        if left.range.end > right.range.start
            || (left.range.start == left.range.end
                && right.range.start == right.range.end
                && left.range.start == right.range.start)
        {
            return None;
        }
    }
    Some(edits)
}

fn diagnostic_code(diagnostic: &AnalyzerDiagnostic) -> &'static str {
    let message = diagnostic.message.as_str();
    if message.contains("syntax error") {
        "linguini.syntax"
    } else if message.contains("missing") {
        "linguini.missing"
    } else if message.contains("unknown") {
        "linguini.unknown"
    } else if message.contains("duplicate") {
        "linguini.duplicate"
    } else {
        "linguini.semantic"
    }
}

fn quick_fix_group_title(title: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;
    for ch in title.chars() {
        if ch == '`' {
            in_code = !in_code;
            if !in_code {
                output.push_str("`...`");
            }
            continue;
        }
        if !in_code {
            output.push(ch);
        }
    }
    output
}

fn to_range(document: &LinguiniDocument, span: linguini_syntax::Span) -> Range {
    let ((start_line, start_character), (end_line, end_character)) = document.range(span);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_non_overlapping, quick_fix_code_action, to_lsp_diagnostic,
        to_lsp_diagnostic_with_workspace,
    };
    use crate::LinguiniDocument;
    use linguini_analyzer::{Diagnostic as AnalyzerDiagnostic, QuickFix, Replacement};
    use linguini_syntax::{SourceId, Span};
    use tower_lsp_server::lsp_types::{
        CodeActionOrCommand, Diagnostic, DiagnosticSeverity, Position, Range, TextEdit, Url as Uri,
    };

    #[test]
    fn quick_fix_replacement_becomes_workspace_edit() {
        let document =
            LinguiniDocument::new("file:///shop.lgl", "linguini-locale", "delivery = TODO\n");
        let uri = "file:///shop.lgl".parse::<Uri>().expect("valid uri");
        let action = quick_fix_code_action(
            &uri,
            &document,
            Diagnostic {
                severity: Some(DiagnosticSeverity::ERROR),
                message: "missing locale message".to_owned(),
                ..Default::default()
            },
            QuickFix::replacement(
                "add locale message stub `summary`",
                Replacement {
                    span: Span::new(document.text.len(), document.text.len()),
                    text: "\nsummary = TODO\n".to_owned(),
                },
            ),
        )
        .expect("actionable replacement");

        let CodeActionOrCommand::CodeAction(action) = action else {
            panic!("expected code action");
        };
        let edit = action.edit.expect("workspace edit");
        let edits = edit
            .changes
            .expect("changes")
            .remove(&uri)
            .expect("uri edits");

        assert_eq!(action.title, "add locale message stub `summary`");
        assert_eq!(edits[0].new_text, "\nsummary = TODO\n");
        assert!(action.command.is_none());
    }

    #[test]
    fn apply_all_rejects_overlapping_edits() {
        let edit = |start, end, text: &str| TextEdit {
            range: Range {
                start: Position::new(0, start),
                end: Position::new(0, end),
            },
            new_text: text.to_owned(),
        };

        assert!(
            normalize_non_overlapping(vec![edit(1, 4, "left"), edit(3, 5, "right"),]).is_none()
        );
        assert!(
            normalize_non_overlapping(vec![edit(2, 2, "left"), edit(2, 2, "right"),]).is_none()
        );
    }

    #[test]
    fn diagnostics_keep_notes_related_ranges_and_codes() {
        let document = LinguiniDocument::new("file:///shop.lgs", "linguini-schema", "item\nitem\n");
        let diagnostic =
            AnalyzerDiagnostic::error("duplicate schema declaration `item`", Span::new(5, 9))
                .with_note("names must be unique")
                .with_related(Span::new(0, 4), "first declaration");

        let diagnostic = to_lsp_diagnostic(&document, &diagnostic);

        assert!(diagnostic.message.contains("Note: names must be unique"));
        assert!(diagnostic.code.is_some());
        assert_eq!(
            diagnostic
                .related_information
                .expect("related information")
                .len(),
            1
        );
    }

    #[test]
    fn related_diagnostic_uses_own_source_uri() {
        let locale = LinguiniDocument::new(
            "file:///locale/en.lgl",
            "linguini-locale",
            "delivery = Delivered\n",
        )
        .with_source_id(SourceId(1));
        let schema =
            LinguiniDocument::new("file:///schema/shop.lgs", "linguini-schema", "delivery()\n")
                .with_source_id(SourceId(2));
        let diagnostic =
            AnalyzerDiagnostic::error("locale mismatch", Span::in_source(SourceId(1), 0, 8))
                .with_related(Span::in_source(SourceId(2), 0, 8), "schema declaration");

        let diagnostic = to_lsp_diagnostic_with_workspace(&locale, &diagnostic, &[schema]);
        let related = diagnostic.related_information.expect("related information");

        assert_eq!(related[0].location.uri.as_str(), "file:///schema/shop.lgs");
    }
}
