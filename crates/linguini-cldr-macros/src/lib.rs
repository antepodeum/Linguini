use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parse_macro_input, LitStr, Result, Token};

mod source;

#[proc_macro]
pub fn compiled_cldr_tables(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    match source::generate_compiled_tables_from_cache(input.cache.value(), input.locales()) {
        Ok(source) => source.parse().unwrap_or_else(|error| {
            let message = format!("generated CLDR Rust did not parse: {error}");
            quote!(compile_error!(#message);).into()
        }),
        Err(error) => {
            let message = error.to_string();
            quote!(compile_error!(#message);).into()
        }
    }
}

struct MacroInput {
    cache: LitStr,
    locales: Vec<LitStr>,
}

impl MacroInput {
    fn locales(&self) -> Vec<String> {
        self.locales.iter().map(LitStr::value).collect()
    }
}

impl Parse for MacroInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let cache = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        bracketed!(content in input);
        let mut locales = Vec::new();
        while !content.is_empty() {
            locales.push(content.parse()?);
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(Self { cache, locales })
    }
}

#[cfg(test)]
mod tests {
    use super::source::generate_compiled_tables_from_cache;
    use linguini_cldr::fetch_cldr_from_dir;
    use linguini_test_support::temp_project_dir;
    use std::fs;

    const PLURALS: &str = r#"{"supplemental":{"plurals-type-cardinal":{"en":{"pluralRule-count-one":"i = 1 and v = 0","pluralRule-count-other":""}}}}"#;
    const NUMBERS: &str = r##"{"main":{"en":{"numbers":{"symbols-numberSystem-latn":{"decimal":".","group":","},"decimalFormats-numberSystem-latn":{"standard":"#,##0.###"},"percentFormats-numberSystem-latn":{"standard":"#,##0%"},"currencyFormats-numberSystem-latn":{"standard":"CUR#,##0.00"}}}}}"##;
    const GREGORIAN: &str = r#"{"main":{"en":{"dates":{"calendars":{"gregorian":{"dateFormats":{"full":"EEEE, MMMM d, y","long":"MMMM d, y","medium":"MMM d, y","short":"M/d/yy"},"timeFormats":{"full":"h:mm:ss a zzzz","long":"h:mm:ss a z","medium":"h:mm:ss a","short":"h:mm a"},"dateTimeFormats":{"full":"{1}, {0}","long":"{1}, {0}","medium":"{1}, {0}","short":"{1}, {0}"}}}}}}}"#;

    #[test]
    fn proc_macro_generator_emits_typed_rust_from_cache() {
        let project = temp_project_dir("cldr_macro_generator");
        let supplemental = project
            .path()
            .join("source/cldr-json/cldr-core/supplemental");
        let numbers = project
            .path()
            .join("source/cldr-json/cldr-numbers-full/main/en");
        let dates = project
            .path()
            .join("source/cldr-json/cldr-dates-full/main/en");
        fs::create_dir_all(&supplemental).expect("supplemental");
        fs::create_dir_all(&numbers).expect("numbers");
        fs::create_dir_all(&dates).expect("dates");
        fs::write(supplemental.join("plurals.json"), PLURALS).expect("plurals");
        fs::write(numbers.join("numbers.json"), NUMBERS).expect("numbers json");
        fs::write(dates.join("ca-gregorian.json"), GREGORIAN).expect("calendar");
        let cache = project.path().join(".linguini/cache");
        fetch_cldr_from_dir(project.path().join("source"), &cache).expect("fetch");

        let generated =
            generate_compiled_tables_from_cache(&cache, vec!["en".to_owned()]).expect("generate");

        assert!(generated.contains("CompiledPluralRules"));
        assert!(generated.contains("PLURAL_CATEGORIES_EN"));
        assert!(!generated.contains("supplemental"));
    }
}
