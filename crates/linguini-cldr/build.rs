use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const OFFICIAL_CLDR_JSON_REPO: &str = "https://github.com/unicode-org/cldr-json";
const PLURALS_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/plurals.json";
const GENERATED_FILE: &str = "linguini_generated_plural_rules.rs";

include!("build/plural_rule.rs");

fn main() {
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_PLURALS_JSON");
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_SOURCE_DIR");

    if let Err(error) = run() {
        panic!("failed to generate built-in CLDR plural rules: {error}");
    }
}

fn run() -> Result<(), String> {
    let plurals = plural_source_path()?;
    println!("cargo:rerun-if-changed={}", plurals.display());

    let source = fs::read_to_string(&plurals)
        .map_err(|error| format!("{}: {error}", plurals.display()))?;
    let generated = generate_plural_tables(&source)?;
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|error| error.to_string())?);
    let output = out_dir.join(GENERATED_FILE);
    fs::write(&output, generated).map_err(|error| format!("{}: {error}", output.display()))?;
    Ok(())
}

fn plural_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_PLURALS_JSON") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(source_dir) = env::var("LINGUINI_CLDR_SOURCE_DIR") {
        let source_dir = PathBuf::from(source_dir);
        for candidate in [
            source_dir.join(PLURALS_RELATIVE_PATH),
            source_dir.join("cldr-core/supplemental/plurals.json"),
            source_dir.join("supplemental/plurals.json"),
        ] {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        return Err(format!(
            "LINGUINI_CLDR_SOURCE_DIR={} does not contain {PLURALS_RELATIVE_PATH}",
            source_dir.display()
        ));
    }

    download_official_plural_rules()
}

fn download_official_plural_rules() -> Result<PathBuf, String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|error| error.to_string())?);
    let source_dir = out_dir.join("cldr-json-source");
    if source_dir.exists() {
        fs::remove_dir_all(&source_dir)
            .map_err(|error| format!("{}: {error}", source_dir.display()))?;
    }

    let source_dir_arg = source_dir.to_string_lossy().into_owned();
    run_git(&[
        "clone",
        "--filter=blob:none",
        "--no-checkout",
        "--depth=1",
        OFFICIAL_CLDR_JSON_REPO,
        source_dir_arg.as_str(),
    ])?;
    run_git(&[
        "-C",
        source_dir_arg.as_str(),
        "sparse-checkout",
        "set",
        "--no-cone",
        PLURALS_RELATIVE_PATH,
    ])?;
    run_git(&["-C", source_dir_arg.as_str(), "checkout"])?;

    Ok(source_dir.join(PLURALS_RELATIVE_PATH))
}

fn run_git(args: &[&str]) -> Result<(), String> {
    let status = Command::new("git")
        .args(args)
        .status()
        .map_err(|error| format!("failed to execute git: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {} failed with status {status}", args.join(" ")))
    }
}

fn generate_plural_tables(source: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(source).map_err(|error| error.to_string())?;
    let cardinal = value
        .get("supplemental")
        .and_then(|value| value.get("plurals-type-cardinal"))
        .and_then(Value::as_object)
        .ok_or_else(|| "missing supplemental.plurals-type-cardinal".to_owned())?;

    let mut locales: Vec<_> = cardinal.iter().collect();
    locales.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut compiled_match_arms = String::new();
    let mut source_match_arms = String::new();
    let mut category_tables = String::new();
    let mut predicate_functions = String::new();
    let mut source_functions = String::new();

    for (locale, value) in locales {
        let object = value
            .as_object()
            .ok_or_else(|| format!("plural rules for locale `{locale}` are not an object"))?;
        let mut categories = Vec::new();
        for (key, value) in object {
            let Some(category) = key.strip_prefix("pluralRule-count-") else {
                continue;
            };
            let rule_source = value.as_str().ok_or_else(|| {
                format!("plural rule `{key}` for locale `{locale}` is not a string")
            })?;
            let rule = parse_plural_rule(rule_source)
                .map_err(|error| format!("{locale}/{category}: {error}"))?;
            categories.push((category.to_owned(), rule_source.to_owned(), rule));
        }
        if categories.is_empty() {
            return Err(format!("locale `{locale}` has no plural categories"));
        }
        categories.sort_by(|left, right| {
            category_rank(&left.0)
                .cmp(&category_rank(&right.0))
                .then_with(|| left.0.cmp(&right.0))
        });

        let const_name = const_name(locale);
        let source_function = format!("plural_rule_source_{}", rust_identifier(locale));
        compiled_match_arms.push_str(&format!(
            "        {} => Some(CompiledPluralRules {{ locale: {}, categories: PLURAL_CATEGORIES_{const_name} }}),\n",
            rust_string(locale),
            rust_string(locale)
        ));
        source_match_arms.push_str(&format!(
            "        {} => Some({source_function}()),\n",
            rust_string(locale)
        ));
        category_tables.push_str(&format!(
            "const PLURAL_CATEGORIES_{const_name}: &[CompiledPluralCategory] = &[\n"
        ));
        source_functions.push_str(&format!(
            "fn {source_function}() -> PluralRules {{\n    PluralRules {{\n        locale: {}.to_owned(),\n        categories: vec![\n",
            rust_string(locale)
        ));

        for (category, rule_source, rule) in &categories {
            let function = format!(
                "plural_{}_{}",
                rust_identifier(locale),
                rust_identifier(category)
            );
            category_tables.push_str(&format!(
                "    CompiledPluralCategory {{ category: {}, matches: {function} }},\n",
                rust_string(category)
            ));
            source_functions.push_str(&format!(
                "            PluralCategoryRule {{ category: {}.to_owned(), source: {}.to_owned(), rule: {} }},\n",
                rust_string(category),
                rust_string(rule_source),
                plural_rule_literal(rule)
            ));
            let parameter = if plural_rule_uses_operands(rule) {
                "operands"
            } else {
                "_operands"
            };
            let body = plural_rule_to_rust(rule);
            predicate_functions.push_str(&format!(
                "fn {function}({parameter}: &PluralOperands) -> bool {{\n    {body}\n}}\n\n",
            ));
        }
        category_tables.push_str("];\n\n");
        source_functions.push_str("        ],\n    }\n}\n\n");
    }

    Ok(format!(
        "// generated by crates/linguini-cldr/build.rs from Unicode CLDR plural rules\n\
         fn generated_plural_rules(locale: &str) -> Option<CompiledPluralRules> {{\n\
             match locale {{\n{compiled_match_arms}        _ => None,\n    }}\n}}\n\n\
         fn generated_plural_rule_sources(locale: &str) -> Option<PluralRules> {{\n\
             match locale {{\n{source_match_arms}        _ => None,\n    }}\n}}\n\n\
         fn integer_value(value: (f64, bool)) -> Option<u64> {{\n\
             if value.1 && value.0 >= 0.0 {{ Some(value.0 as u64) }} else {{ None }}\n\
         }}\n\n\
         {category_tables}{source_functions}{predicate_functions}"
    ))
}

fn category_rank(category: &str) -> usize {
    match category {
        "zero" => 0,
        "one" => 1,
        "two" => 2,
        "few" => 3,
        "many" => 4,
        "other" => 5,
        _ => 6,
    }
}

fn const_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn rust_identifier(value: &str) -> String {
    let mut identifier = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            identifier.push(character.to_ascii_lowercase());
        } else {
            identifier.push('_');
        }
    }
    if identifier
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        identifier.insert(0, '_');
    }
    identifier
}

fn rust_string(value: &str) -> String {
    format!("{value:?}")
}
