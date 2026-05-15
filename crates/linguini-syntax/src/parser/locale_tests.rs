use crate::{parse_locale, LocaleDeclaration, TextPart, TextPattern};

#[test]
fn trims_unquoted_raw_text_edges_and_preserves_quoted_edges() {
    let locale = parse_locale("plain =  hello  \nquoted = \"  hello  \"\njoined = {a} {b}\n")
        .expect("locale parses");

    let LocaleDeclaration::Message(plain) = &locale.declarations[0] else {
        panic!("expected plain message");
    };
    assert_eq!(raw_text(&plain.value), "hello");

    let LocaleDeclaration::Message(quoted) = &locale.declarations[1] else {
        panic!("expected quoted message");
    };
    assert_eq!(raw_text(&quoted.value), "  hello  ");

    let LocaleDeclaration::Message(joined) = &locale.declarations[2] else {
        panic!("expected joined message");
    };
    assert!(matches!(&joined.value.parts[1], TextPart::Text(text) if text.value == " "));
}

fn raw_text(text: &TextPattern) -> &str {
    match &text.parts[0] {
        TextPart::Text(raw) => &raw.value,
        TextPart::Placeholder(_) => panic!("expected text"),
    }
}
