use linguini_analyzer::{analyze_locale_coverage, analyze_locale_file};
use linguini_syntax::{parse_locale, parse_schema};

#[test]
fn locale_coverage_reports_incomplete_schema_enum_impl() {
    let schema = parse_schema("enum Item { pasta, sauce, cookbook }\n").expect("schema parses");
    let locale = parse_locale("impl Item {\n  pasta {}\n  sauce {}\n}\n").expect("locale parses");

    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "impl `Item` for enum `Item` is missing variant `cookbook`"
    );
    assert_eq!(diagnostics[0].quick_fixes.len(), 1);
    assert_eq!(diagnostics[0].quick_fixes[0].title, "add variant `cookbook`");
}

#[test]
fn locale_file_analysis_reports_incomplete_local_enum_impl() {
    let locale = parse_locale(
        "enum Item { pasta, sauce, cookbook }\n\
         impl Item {\n\
           pasta {}\n\
           sauce {}\n\
         }\n",
    )
    .expect("locale parses");

    let diagnostics = analyze_locale_file(&locale);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "impl `Item` for enum `Item` is missing variant `cookbook`"
    );
}
