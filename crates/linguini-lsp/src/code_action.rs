use crate::LinguiniDocument;
use linguini_analyzer::{Diagnostic as AnalyzerDiagnostic, QuickFix};
use std::collections::HashMap;
use tower_lsp_server::ls_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, DiagnosticSeverity,
    Position, Range, TextEdit, Uri, WorkspaceEdit,
};

pub(crate) fn diagnostic_display_actions(diagnostics: Vec<Diagnostic>) -> Vec<CodeActionOrCommand> {
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Show Linguini diagnostic: {}", diagnostic.message),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic]),
                ..Default::default()
            })
        })
        .collect()
}

pub(crate) fn analyzer_quick_fix_actions(
    uri: &Uri,
    document: &LinguiniDocument,
    range: Range,
    diagnostics: impl IntoIterator<Item = AnalyzerDiagnostic>,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    for diagnostic in diagnostics {
        if !diagnostic_has_action_for_range(document, &diagnostic, range) {
            continue;
        }
        let lsp_diagnostic = to_lsp_diagnostic(document, &diagnostic);
        for quick_fix in diagnostic.quick_fixes {
            actions.push(quick_fix_code_action(
                uri,
                document,
                lsp_diagnostic.clone(),
                quick_fix,
            ));
        }
    }
    actions
}

pub(crate) fn to_lsp_diagnostic(
    document: &LinguiniDocument,
    diagnostic: &linguini_analyzer::Diagnostic,
) -> Diagnostic {
    Diagnostic {
        range: to_range(document, diagnostic.span),
        severity: Some(match diagnostic.severity {
            linguini_analyzer::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
            linguini_analyzer::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
            linguini_analyzer::DiagnosticSeverity::Advice => DiagnosticSeverity::HINT,
        }),
        source: Some("linguini".to_owned()),
        message: diagnostic.message.clone(),
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
        return true;
    }
    ranges_overlap(to_range(document, diagnostic.span), range)
}

fn ranges_overlap(left: Range, right: Range) -> bool {
    left.start <= right.end && right.start <= left.end
}

fn quick_fix_code_action(
    uri: &Uri,
    document: &LinguiniDocument,
    diagnostic: Diagnostic,
    quick_fix: QuickFix,
) -> CodeActionOrCommand {
    let edit = quick_fix.replacement.map(|replacement| {
        let mut changes = HashMap::new();
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: to_range(document, replacement.span),
                new_text: replacement.text,
            }],
        );
        WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }
    });
    let command = quick_fix.id.map(|id| Command {
        title: quick_fix.title.clone(),
        command: id,
        arguments: None,
    });

    CodeActionOrCommand::CodeAction(CodeAction {
        title: quick_fix.title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit,
        command,
        ..Default::default()
    })
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
    use super::quick_fix_code_action;
    use crate::LinguiniDocument;
    use linguini_analyzer::{QuickFix, Replacement};
    use linguini_syntax::Span;
    use tower_lsp_server::ls_types::{CodeActionOrCommand, Diagnostic, DiagnosticSeverity, Uri};

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
        );

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
    }
}
