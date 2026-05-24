use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Result};

mod formatting;
mod plural_rule;
mod source;
mod source_paths;

#[proc_macro]
pub fn compiled_cldr_tables(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as EmptyInput);

    match source::generate_compiled_tables() {
        Ok(tokens) => tokens.into(),
        Err(error) => {
            let message = error.to_string();
            quote!(compile_error!(#message);).into()
        }
    }
}

struct EmptyInput;

impl Parse for EmptyInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            Ok(Self)
        } else {
            Err(input.error("compiled_cldr_tables! takes no arguments"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::source::generate_compiled_tables;
    use linguini_test_support::temp_project_dir;
    use std::env;
    use std::fs;

    const PLURALS: &str = r#"{"supplemental":{"plurals-type-cardinal":{"en":{"pluralRule-count-one":"i = 1 and v = 0","pluralRule-count-other":""}}}}"#;
    const NUMBERS: &str = r##"{"main":{"en":{"numbers":{"symbols-numberSystem-latn":{"decimal":".","group":","},"decimalFormats-numberSystem-latn":{"standard":"#,##0.###"},"percentFormats-numberSystem-latn":{"standard":"#,##0%"},"currencyFormats-numberSystem-latn":{"standard":"CUR#,##0.00"}}}}}"##;
    const GREGORIAN: &str = r#"{"main":{"en":{"dates":{"calendars":{"gregorian":{"dateFormats":{"full":"EEEE, MMMM d, y","long":"MMMM d, y","medium":"MMM d, y","short":"M/d/yy"},"timeFormats":{"full":"h:mm:ss a zzzz","long":"h:mm:ss a z","medium":"h:mm:ss a","short":"h:mm a"},"dateTimeFormats":{"full":"{1}, {0}","long":"{1}, {0}","medium":"{1}, {0}","short":"{1}, {0}"},"months":{"format":{"wide":{"1":"Month 1","2":"Month 2","3":"Month 3","4":"Month 4","5":"Month 5","6":"Month 6","7":"Month 7","8":"Month 8","9":"Month 9","10":"Month 10","11":"Month 11","12":"Month 12"},"abbreviated":{"1":"M01","2":"M02","3":"M03","4":"M04","5":"M05","6":"M06","7":"M07","8":"M08","9":"M09","10":"M10","11":"M11","12":"M12"}}},"days":{"format":{"wide":{"sun":"Sunday","mon":"Monday","tue":"Tuesday","wed":"Wednesday","thu":"Thursday","fri":"Friday","sat":"Saturday"},"abbreviated":{"sun":"Sun","mon":"Mon","tue":"Tue","wed":"Wed","thu":"Thu","fri":"Fri","sat":"Sat"}}}}}}}}}"#;
    const LAYOUT: &str = r#"{"main":{"en":{"layout":{"orientation":{"characterOrder":"left-to-right","lineOrder":"top-to-bottom"}}}}}"#;

    #[test]
    fn proc_macro_generator_emits_typed_rust_from_source() {
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
        let layout = project
            .path()
            .join("source/cldr-json/cldr-misc-full/main/en");
        fs::create_dir_all(&supplemental).expect("supplemental");
        fs::create_dir_all(&numbers).expect("numbers");
        fs::create_dir_all(&dates).expect("dates");
        fs::create_dir_all(&layout).expect("layout");
        fs::write(supplemental.join("plurals.json"), PLURALS).expect("plurals");
        fs::write(numbers.join("numbers.json"), NUMBERS).expect("numbers json");
        fs::write(dates.join("ca-gregorian.json"), GREGORIAN).expect("calendar");
        fs::write(layout.join("layout.json"), LAYOUT).expect("layout json");
        let previous = env::var("LINGUINI_CLDR_SOURCE_DIR").ok();
        env::set_var("LINGUINI_CLDR_SOURCE_DIR", project.path().join("source"));

        let generated = generate_compiled_tables().expect("generate").to_string();
        if let Some(previous) = previous {
            env::set_var("LINGUINI_CLDR_SOURCE_DIR", previous);
        } else {
            env::remove_var("LINGUINI_CLDR_SOURCE_DIR");
        }

        assert!(generated.contains("CompiledPluralRules"));
        assert!(generated.contains("PLURAL_CATEGORIES_EN"));
        assert!(generated.contains("generated_text_direction"));
        assert!(generated.contains("\"ltr\""));
        assert!(generated.contains("abbreviated : [\"M01\""));
        assert!(!generated.contains("box_new"));
        assert!(!generated.contains("into_vec"));
        assert!(!generated.contains("supplemental"));
    }
}
