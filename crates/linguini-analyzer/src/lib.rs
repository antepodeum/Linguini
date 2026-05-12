mod diagnostic;
mod message_coverage;

pub use diagnostic::{
    render_diagnostics, Diagnostic, DiagnosticSeverity, QuickFix, RelatedSpan, RenderError,
    Replacement,
};
pub use message_coverage::{analyze_message_coverage, PublicMessage};

pub const CRATE_PURPOSE: &str = "semantic analysis";

#[cfg(test)]
mod tests {
    use super::{
        analyze_message_coverage, render_diagnostics, Diagnostic, PublicMessage, QuickFix,
    };
    use linguini_syntax::Span;

    #[test]
    fn renders_primary_span_related_span_note_and_quick_fix() {
        let diagnostic = Diagnostic::error("unknown type `Color`", Span::new(13, 18))
            .with_related(Span::new(0, 5), "while checking this message")
            .with_note("schema types must be declared before use")
            .with_quick_fix(QuickFix::hint("declare enum Color"));

        let rendered = render_diagnostics("shop.lqs", "paint(color: Color)\n", &[diagnostic])
            .expect("render diagnostics");

        assert_eq!(
            rendered,
            include_str!("../../../tests/fixtures/golden/snapshots/diagnostic-schema-syntax.txt")
        );
    }

    #[test]
    fn message_coverage_accepts_matching_public_messages() {
        let diagnostics = analyze_message_coverage(
            &[PublicMessage::new("delivery", Span::new(0, 8))],
            &[PublicMessage::new("delivery", Span::new(10, 18))],
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn message_coverage_reports_missing_public_message() {
        let diagnostics =
            analyze_message_coverage(&[PublicMessage::new("delivery", Span::new(0, 8))], &[]);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "missing locale implementation for public message `delivery`"
        );
    }

    #[test]
    fn message_coverage_reports_unknown_public_message() {
        let diagnostics =
            analyze_message_coverage(&[], &[PublicMessage::new("delivery", Span::new(10, 18))]);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "unknown public message implementation `delivery`"
        );
    }
}
