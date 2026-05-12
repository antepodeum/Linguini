mod diagnostic;

pub use diagnostic::{
    render_diagnostics, Diagnostic, DiagnosticSeverity, QuickFix, RelatedSpan, RenderError,
    Replacement,
};

pub const CRATE_PURPOSE: &str = "semantic analysis";

#[cfg(test)]
mod tests {
    use super::{render_diagnostics, Diagnostic, QuickFix};
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
}
