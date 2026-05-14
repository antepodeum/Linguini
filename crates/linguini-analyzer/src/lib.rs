mod branch_coverage;
mod diagnostic;
mod expression;
mod locale;
mod message_coverage;
mod reference;

pub use branch_coverage::{
    analyze_branch_coverage, require_other_branch, BranchCoverage, NamedSpan,
};
pub use diagnostic::{
    render_diagnostics, render_diagnostics_with_color, Diagnostic, DiagnosticSeverity, QuickFix,
    RelatedSpan, RenderError, Replacement,
};
pub use expression::{
    analyze_expressions, analyze_function_patterns, ExpressionAnalysis, FormProperty,
    FormSignature, FunctionSignature, MessageToAnalyze, Variable,
};
pub use locale::{
    analyze_locale_coverage, analyze_locale_coverage_with_options, analyze_locale_file,
    analyze_locale_message_coverage, analyze_locale_message_coverage_with_options,
    locale_public_messages, schema_public_messages, ImplementedLocaleMessage,
    LocaleCoverageOptions, RequiredLocaleMessage,
};
pub use message_coverage::{analyze_message_coverage, PublicMessage};
pub use reference::{detect_reference_cycles, ReferenceNode};

pub const CRATE_PURPOSE: &str = "semantic analysis";

#[cfg(test)]
mod tests {
    use super::{
        analyze_branch_coverage, analyze_expressions, analyze_function_patterns,
        analyze_locale_coverage, analyze_locale_coverage_with_options, analyze_locale_file,
        analyze_message_coverage, detect_reference_cycles, render_diagnostics,
        require_other_branch, BranchCoverage, Diagnostic, DiagnosticSeverity, ExpressionAnalysis,
        FormProperty, FormSignature, FunctionSignature, LocaleCoverageOptions, MessageToAnalyze,
        NamedSpan, PublicMessage, QuickFix, ReferenceNode, Variable,
    };
    use linguini_syntax::{parse_locale, parse_schema, Span};

    #[test]
    fn renders_primary_span_related_span_note_and_quick_fix() {
        let diagnostic = Diagnostic::error("unknown type `Color`", Span::new(13, 18))
            .with_related(Span::new(0, 5), "while checking this message")
            .with_note("schema types must be declared before use")
            .with_quick_fix(QuickFix::hint("declare enum Color"));

        let rendered = render_diagnostics("shop.lgs", "paint(color: Color)\n", &[diagnostic])
            .expect("render diagnostics");

        assert_eq!(
            rendered,
            include_str!("../../../tests/fixtures/golden/snapshots/diagnostic-schema-syntax.txt")
        );
    }

    #[test]
    fn locale_analysis_does_not_warn_about_empty_files_without_schema_context() {
        let locale = parse_locale("").expect("empty locale is syntactically valid");
        let diagnostics = analyze_locale_file(&locale);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn locale_coverage_groups_missing_schema_messages() {
        let schema = parse_schema("delivery()\ncounted()\n").expect("schema parses");
        let locale = parse_locale("delivery = Delivered\n").expect("locale parses");
        let diagnostics = analyze_locale_coverage(&schema, &locale);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "locale is missing 1 schema message: `counted`"
        );
        assert_eq!(diagnostics[0].quick_fixes.len(), 1);
    }

    #[test]
    fn locale_coverage_groups_missing_grouped_schema_messages() {
        let schema = parse_schema("email_input {\n  label\n  placeholder\n  error\n}\n")
            .expect("schema parses");
        let locale = parse_locale("email_input {\n  label = Email\n}\n").expect("locale parses");
        let diagnostics = analyze_locale_coverage(&schema, &locale);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "locale is missing 2 schema messages: `email_input.placeholder`, `email_input.error`"
        );
        let replacement = diagnostics[0].quick_fixes[0]
            .replacement
            .as_ref()
            .expect("quick fix replacement");
        assert!(replacement.text.contains("email_input {"));
        assert!(replacement.text.contains("  placeholder = TODO"));
        assert!(replacement.text.contains("  error = TODO"));
    }

    #[test]
    fn locale_message_coverage_uses_requested_warning_severity_for_missing_messages() {
        let diagnostics = analyze_locale_coverage_with_options(
            &parse_schema("delivery()\n").expect("schema parses"),
            &parse_locale("").expect("empty locale parses"),
            LocaleCoverageOptions {
                missing_message_severity: DiagnosticSeverity::Warning,
                subject: "locale `ru`".to_owned(),
                quick_fix_id: None,
            },
        );

        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostics[0].span, Span::new(0, 0));
        assert_eq!(
            diagnostics[0].message,
            "locale `ru` is missing 1 schema message: `delivery`"
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

    #[test]
    fn branch_coverage_reports_missing_enum_variant() {
        let diagnostics = analyze_branch_coverage(BranchCoverage {
            subject: "form `Fruit.nom`",
            enum_name: "Fruit",
            variants: vec![
                NamedSpan::new("apple", Span::new(5, 10)),
                NamedSpan::new("pear", Span::new(13, 17)),
            ],
            branches: vec![NamedSpan::new("apple", Span::new(30, 35))],
            span: Span::new(20, 40),
        });

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "form `Fruit.nom` for enum `Fruit` is missing branch `pear`"
        );
        assert_eq!(diagnostics[0].related.len(), 1);
    }

    #[test]
    fn required_other_branch_reports_missing_fallback() {
        let diagnostics = require_other_branch(
            "plural map `Fruit.nom`",
            &[NamedSpan::new("one", Span::new(0, 3))],
            Span::new(0, 12),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "plural map `Fruit.nom` is missing required `other` branch"
        );
    }

    #[test]
    fn required_other_branch_accepts_fallback() {
        let diagnostics = require_other_branch(
            "plural map `Fruit.nom`",
            &[NamedSpan::new("other", Span::new(0, 5))],
            Span::new(0, 12),
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn expression_analysis_accepts_valid_delivery_message() {
        let locale =
            parse_locale("delivery = {Delivered(count, fruit.Gender)} {fruit.nom(count)}\n")
                .expect("locale parses");
        let value = message_value(&locale, "delivery");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "delivery",
                value,
                vec![
                    Variable::new("count", "Number", Span::new(0, 0)),
                    Variable::new("fruit", "Fruit", Span::new(0, 0)),
                ],
            )],
            functions: vec![FunctionSignature::new("Delivered", 2, Span::new(12, 21))],
            forms: vec![FormSignature::new(
                "Fruit",
                vec![
                    FormProperty::new("Gender", Span::new(0, 0)),
                    FormProperty::new("nom", Span::new(0, 0)),
                ],
                Span::new(0, 0),
            )],
        });

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn expression_analysis_reports_unknown_variable() {
        let locale = parse_locale("delivery = {fruit.nom}\n").expect("locale parses");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "delivery",
                message_value(&locale, "delivery"),
                vec![],
            )],
            functions: vec![],
            forms: vec![],
        });

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "unknown variable `fruit`");
    }

    #[test]
    fn expression_analysis_reports_unknown_form_property() {
        let locale = parse_locale("delivery = {fruit.acc}\n").expect("locale parses");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "delivery",
                message_value(&locale, "delivery"),
                vec![Variable::new("fruit", "Fruit", Span::new(0, 0))],
            )],
            functions: vec![],
            forms: vec![FormSignature::new(
                "Fruit",
                vec![FormProperty::new("nom", Span::new(0, 0))],
                Span::new(0, 0),
            )],
        });

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "unknown form property `acc` on type `Fruit`"
        );
    }

    #[test]
    fn expression_analysis_reports_function_arity() {
        let locale = parse_locale("delivery = {size(fruit)}\n").expect("locale parses");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "delivery",
                message_value(&locale, "delivery"),
                vec![Variable::new("fruit", "Fruit", Span::new(0, 0))],
            )],
            functions: vec![FunctionSignature::new("size", 2, Span::new(0, 0))],
            forms: vec![],
        });

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "function `size` expects 2 argument(s), got 1"
        );
    }

    #[test]
    fn expression_analysis_reports_unknown_function_call() {
        let locale = parse_locale("delivery = {missing(fruit)}\n").expect("locale parses");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "delivery",
                message_value(&locale, "delivery"),
                vec![Variable::new("fruit", "Fruit", Span::new(0, 0))],
            )],
            functions: vec![],
            forms: vec![],
        });

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "unknown function `missing`");
    }

    #[test]
    fn expression_analysis_accepts_int_plural_argument() {
        let locale = parse_locale("delivery = {SizeAdj(size, count, fruit.Gender)}\n")
            .expect("locale parses");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "delivery",
                message_value(&locale, "delivery"),
                vec![
                    Variable::new("size", "Size", Span::new(0, 0)),
                    Variable::new("count", "Number", Span::new(0, 0)),
                    Variable::new("fruit", "Fruit", Span::new(0, 0)),
                ],
            )],
            functions: vec![FunctionSignature::new("SizeAdj", 3, Span::new(0, 0))],
            forms: vec![FormSignature::new(
                "Fruit",
                vec![FormProperty::new("Gender", Span::new(0, 0))],
                Span::new(0, 0),
            )],
        });

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn function_pattern_analysis_reports_dispatch_depth() {
        let locale = parse_locale("form Choose(Gender, Size) {\n  male => ok\n  _ => ok\n}\n")
            .expect("locale parses");
        let diagnostics = analyze_function_patterns(&locale);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0].message,
            "function `Choose` branch pattern expects 2 value(s), got 1"
        );
    }

    #[test]
    fn expression_analysis_reports_ambiguous_implicit_plural_argument() {
        let locale = parse_locale("summary = {fruit.nom}\n").expect("locale parses");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "summary",
                message_value(&locale, "summary"),
                vec![
                    Variable::new("apples", "Number", Span::new(0, 0)),
                    Variable::new("pears", "Number", Span::new(0, 0)),
                    Variable::new("fruit", "Fruit", Span::new(0, 0)),
                ],
            )],
            functions: vec![],
            forms: vec![FormSignature::new(
                "Fruit",
                vec![FormProperty::plural("nom", Span::new(0, 0))],
                Span::new(0, 0),
            )],
        });

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "ambiguous implicit plural argument for `fruit.nom`; pass a numeric argument explicitly"
        );
    }

    #[test]
    fn expression_analysis_accepts_single_implicit_plural_argument() {
        let locale = parse_locale("summary = {fruit.nom}\n").expect("locale parses");
        let diagnostics = analyze_expressions(ExpressionAnalysis {
            messages: vec![MessageToAnalyze::new(
                "summary",
                message_value(&locale, "summary"),
                vec![
                    Variable::new("count", "Number", Span::new(0, 0)),
                    Variable::new("fruit", "Fruit", Span::new(0, 0)),
                ],
            )],
            functions: vec![],
            forms: vec![FormSignature::new(
                "Fruit",
                vec![FormProperty::plural("nom", Span::new(0, 0))],
                Span::new(0, 0),
            )],
        });

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn reference_cycle_analysis_reports_cycle() {
        let diagnostics = detect_reference_cycles(&[ReferenceNode::new(
            "delivery",
            vec![NamedSpan::new("delivery", Span::new(4, 12))],
            Span::new(0, 12),
        )]);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "cyclic reference `delivery -> delivery`"
        );
    }

    fn message_value(
        locale: &linguini_syntax::LocaleFile,
        name: &str,
    ) -> linguini_syntax::TextPattern {
        locale
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                linguini_syntax::LocaleDeclaration::Message(message)
                    if message.name.value == name =>
                {
                    Some(message.value.clone())
                }
                _ => None,
            })
            .expect("message exists")
    }
}
