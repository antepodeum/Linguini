pub const SCHEMA_EXTENSION: &str = "lqs";
pub const LOCALE_EXTENSION: &str = "lgl";

#[cfg(test)]
mod tests {
    use super::{LOCALE_EXTENSION, SCHEMA_EXTENSION};

    #[test]
    fn dsl_extensions_match_spec() {
        assert_eq!(SCHEMA_EXTENSION, "lqs");
        assert_eq!(LOCALE_EXTENSION, "lgl");
    }
}
