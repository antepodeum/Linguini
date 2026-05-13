pub(crate) fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

pub(crate) fn namespace_display(namespace: &str) -> String {
    if namespace.is_empty() {
        "<root>".to_owned()
    } else {
        namespace.to_owned()
    }
}
