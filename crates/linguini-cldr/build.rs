use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PLURALS_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/plurals.json";
const LAYOUT_MAIN_RELATIVE_PATH: &str = "cldr-json/cldr-misc-full/main";
const LOCAL_CLDR_SOURCE_RELATIVE_PATH: &str = "vendor/cldr-json";
const CLDR_SOURCE_CONFIG_RELATIVE_PATH: &str = "cldr-json.toml";
const GENERATED_FILE: &str = "linguini_generated_plural_rules.rs";

include!("build/plural_rule.rs");

fn main() {
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_PLURALS_JSON");
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_LAYOUT_MAIN_DIR");
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_AUTO_FETCH");

    if let Err(error) = run() {
        panic!("failed to generate built-in CLDR data: {error}");
    }
}

fn run() -> Result<(), String> {
    let plurals = plural_source_path()?;
    let layout_main = layout_main_source_path()?;
    println!("cargo:rerun-if-changed={}", plurals.display());
    println!("cargo:rerun-if-changed={}", layout_main.display());

    let source =
        fs::read_to_string(&plurals).map_err(|error| format!("{}: {error}", plurals.display()))?;
    let mut generated = generate_plural_tables(&source)?;
    generated.push_str(&generate_text_direction_table(&layout_main)?);
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|error| error.to_string())?);
    let output = out_dir.join(GENERATED_FILE);
    fs::write(&output, generated).map_err(|error| format!("{}: {error}", output.display()))?;
    Ok(())
}

fn plural_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_PLURALS_JSON") {
        return Ok(PathBuf::from(path));
    }

    let source_dir = cldr_source_dir()?;
    plural_source_path_from_source_dir(source_dir)
}

#[derive(Debug)]
struct CldrSourceConfig {
    repo: String,
    git_ref: String,
    commit_prefix: String,
}

fn cldr_source_dir() -> Result<PathBuf, String> {
    if let Ok(source_dir) = env::var("LINGUINI_CLDR_SOURCE_DIR") {
        return Ok(PathBuf::from(source_dir));
    }

    let manifest_dir = manifest_dir()?;
    let source_dir = manifest_dir.join(LOCAL_CLDR_SOURCE_RELATIVE_PATH);
    if is_usable_cldr_source_dir(&source_dir) {
        return Ok(source_dir);
    }

    if matches!(
        env::var("LINGUINI_CLDR_AUTO_FETCH").as_deref(),
        Ok("0") | Ok("false")
    ) {
        return Err(format!(
            "missing local CLDR JSON checkout at {}. Run `./scripts/fetch-cldr-json.sh` or unset LINGUINI_CLDR_AUTO_FETCH=0.",
            source_dir.display()
        ));
    }

    let config_path = manifest_dir.join(CLDR_SOURCE_CONFIG_RELATIVE_PATH);
    println!("cargo:rerun-if-changed={}", config_path.display());
    let config = read_cldr_source_config(&config_path)?;
    fetch_cldr_json(&source_dir, &config)?;
    Ok(source_dir)
}

fn is_usable_cldr_source_dir(path: &Path) -> bool {
    path.join("cldr-json/cldr-core/supplemental/plurals.json")
        .is_file()
        && path.join("cldr-json/cldr-misc-full/main").is_dir()
}

fn manifest_dir() -> Result<PathBuf, String> {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|error| error.to_string())?)
        .canonicalize()
        .map_err(|error| error.to_string())
}

fn read_cldr_source_config(path: &Path) -> Result<CldrSourceConfig, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let repo = toml_string_value(&source, "repo")
        .ok_or_else(|| format!("{}: missing `repo`", path.display()))?;
    let git_ref = toml_string_value(&source, "ref")
        .ok_or_else(|| format!("{}: missing `ref`", path.display()))?;
    let commit_prefix = toml_string_value(&source, "commit_prefix")
        .ok_or_else(|| format!("{}: missing `commit_prefix`", path.display()))?;
    Ok(CldrSourceConfig {
        repo,
        git_ref,
        commit_prefix,
    })
}

fn toml_string_value(source: &str, key: &str) -> Option<String> {
    for line in source.lines() {
        let line = line
            .split_once('#')
            .map_or(line, |(before, _)| before)
            .trim();
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() != key {
            continue;
        }
        let value = right.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].to_owned());
        }
    }
    None
}

fn fetch_cldr_json(source_dir: &Path, config: &CldrSourceConfig) -> Result<(), String> {
    if source_dir.exists() {
        fs::remove_dir_all(source_dir)
            .map_err(|error| format!("{}: {error}", source_dir.display()))?;
    }
    if let Some(parent) = source_dir.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }

    let source_dir_arg = source_dir.to_string_lossy().into_owned();
    run_git(["init", source_dir_arg.as_str()])?;
    run_git([
        "-C",
        source_dir_arg.as_str(),
        "remote",
        "add",
        "origin",
        config.repo.as_str(),
    ])?;
    run_git([
        "-C",
        source_dir_arg.as_str(),
        "fetch",
        "--depth=1",
        "origin",
        config.git_ref.as_str(),
    ])?;
    run_git([
        "-C",
        source_dir_arg.as_str(),
        "checkout",
        "--detach",
        "FETCH_HEAD",
    ])?;

    let head = git_output(["-C", source_dir_arg.as_str(), "rev-parse", "HEAD"])?;
    if !head.trim().starts_with(&config.commit_prefix) {
        return Err(format!(
            "CLDR JSON ref `{}` resolved to {}, expected commit prefix {}",
            config.git_ref,
            head.trim(),
            config.commit_prefix
        ));
    }

    if !is_usable_cldr_source_dir(source_dir) {
        return Err(format!(
            "CLDR JSON checkout at {} does not contain expected cldr-json data",
            source_dir.display()
        ));
    }

    Ok(())
}

fn run_git<'a>(args: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    let args: Vec<&str> = args.into_iter().collect();
    let status = Command::new("git")
        .args(&args)
        .status()
        .map_err(|error| format!("git {}: {error}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {} failed with status {}",
            args.join(" "),
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_owned())
        ))
    }
}

fn git_output<'a>(args: impl IntoIterator<Item = &'a str>) -> Result<String, String> {
    let args: Vec<&str> = args.into_iter().collect();
    let output = Command::new("git")
        .args(&args)
        .output()
        .map_err(|error| format!("git {}: {error}", args.join(" ")))?;
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|error| error.to_string())
    } else {
        Err(format!(
            "git {} failed with status {}",
            args.join(" "),
            output
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_owned())
        ))
    }
}

fn plural_source_path_from_source_dir(source_dir: PathBuf) -> Result<PathBuf, String> {
    for candidate in [
        source_dir.join(PLURALS_RELATIVE_PATH),
        source_dir.join("cldr-core/supplemental/plurals.json"),
        source_dir.join("supplemental/plurals.json"),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "LINGUINI_CLDR_SOURCE_DIR={} does not contain {PLURALS_RELATIVE_PATH}",
        source_dir.display()
    ))
}

fn layout_main_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_LAYOUT_MAIN_DIR") {
        return Ok(PathBuf::from(path));
    }

    let source_dir = cldr_source_dir()?;
    layout_main_source_path_from_source_dir(source_dir)
}

fn layout_main_source_path_from_source_dir(source_dir: PathBuf) -> Result<PathBuf, String> {
    for candidate in [
        source_dir.join(LAYOUT_MAIN_RELATIVE_PATH),
        source_dir.join("cldr-misc-full/main"),
        source_dir.join("main"),
    ] {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "LINGUINI_CLDR_SOURCE_DIR={} does not contain {LAYOUT_MAIN_RELATIVE_PATH}",
        source_dir.display()
    ))
}

fn generate_text_direction_table(layout_main: &std::path::Path) -> Result<String, String> {
    let mut directions = Vec::new();
    let entries =
        fs::read_dir(layout_main).map_err(|error| format!("{}: {error}", layout_main.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", layout_main.display()))?;
        if !entry.path().is_dir() {
            continue;
        }
        let locale = entry.file_name().to_string_lossy().into_owned();
        let layout = entry.path().join("layout.json");
        if !layout.is_file() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", layout.display());
        let source = fs::read_to_string(&layout)
            .map_err(|error| format!("{}: {error}", layout.display()))?;
        if let Some(direction) = extract_text_direction(&source) {
            directions.push((locale, direction));
        }
    }
    directions.sort_by(|left, right| left.0.cmp(&right.0));

    let mut arms = String::new();
    for (locale, direction) in directions {
        arms.push_str(&format!(
            "        {} => Some({}),\n",
            rust_string(&locale),
            rust_string(direction)
        ));
    }

    Ok(format!(
        "\nfn generated_text_direction(locale: &str) -> Option<&'static str> {{\n    match locale {{\n{arms}        _ => None,\n    }}\n}}\n"
    ))
}

fn extract_text_direction(source: &str) -> Option<&'static str> {
    let key = "\"characterOrder\"";
    let key_start = source.find(key)?;
    let after_key = &source[key_start + key.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    if after_colon.starts_with("\"right-to-left\"") {
        Some("rtl")
    } else if after_colon.starts_with("\"left-to-right\"") {
        Some("ltr")
    } else {
        None
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
    locales.sort_by_key(|(left, _)| *left);

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
