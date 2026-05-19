use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PLURALS_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/plurals.json";
const LAYOUT_MAIN_RELATIVE_PATH: &str = "cldr-json/cldr-misc-full/main";
const NUMBERS_MAIN_RELATIVE_PATH: &str = "cldr-json/cldr-numbers-full/main";
const DATES_MAIN_RELATIVE_PATH: &str = "cldr-json/cldr-dates-full/main";
const LOCAL_CLDR_SOURCE_RELATIVE_PATH: &str = "vendor/cldr-json";
const CLDR_SOURCE_CONFIG_RELATIVE_PATH: &str = "cldr-json.toml";
const CLDR_SOURCE_CHECKOUT_ENV: &str = "LINGUINI_CLDR_SOURCE_CHECKOUT_DIR";

pub(crate) fn plural_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_PLURALS_JSON") {
        return Ok(PathBuf::from(path));
    }

    let source_dir = cldr_source_dir()?;
    plural_source_path_from_source_dir(source_dir)
}

pub(crate) fn layout_main_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_LAYOUT_MAIN_DIR") {
        return Ok(PathBuf::from(path));
    }

    let source_dir = cldr_source_dir()?;
    layout_main_source_path_from_source_dir(source_dir)
}

pub(crate) fn numbers_main_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_NUMBERS_MAIN_DIR") {
        return Ok(PathBuf::from(path));
    }

    let source_dir = cldr_source_dir()?;
    main_source_path_from_source_dir(
        source_dir,
        NUMBERS_MAIN_RELATIVE_PATH,
        "cldr-numbers-full/main",
    )
}

pub(crate) fn dates_main_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_DATES_MAIN_DIR") {
        return Ok(PathBuf::from(path));
    }

    let source_dir = cldr_source_dir()?;
    main_source_path_from_source_dir(source_dir, DATES_MAIN_RELATIVE_PATH, "cldr-dates-full/main")
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

    let macro_manifest_dir = macro_manifest_dir();
    let source_dir = macro_manifest_dir.join(LOCAL_CLDR_SOURCE_RELATIVE_PATH);
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

    let config_path = macro_manifest_dir.join(CLDR_SOURCE_CONFIG_RELATIVE_PATH);
    let config = read_cldr_source_config(&config_path)?;
    let source_checkout = env::var(CLDR_SOURCE_CHECKOUT_ENV)
        .map(PathBuf::from)
        .unwrap_or(source_dir);
    fetch_cldr_json(&source_checkout, &config)?;
    Ok(source_checkout)
}

fn macro_manifest_dir() -> PathBuf {
    let compile_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if has_source_config(&compile_manifest_dir) {
        return compile_manifest_dir;
    }

    if let Ok(runtime_manifest_dir) = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from) {
        if has_source_config(&runtime_manifest_dir) {
            return runtime_manifest_dir;
        }
        if let Some(sibling) = macro_crate_sibling(&runtime_manifest_dir) {
            return sibling;
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        for ancestor in current_dir.ancestors() {
            if has_source_config(ancestor) {
                return ancestor.to_path_buf();
            }
            let candidate = ancestor.join("crates/linguini-cldr-macros");
            if has_source_config(&candidate) {
                return candidate;
            }
        }
    }

    compile_manifest_dir
}

fn has_source_config(path: &Path) -> bool {
    path.join(CLDR_SOURCE_CONFIG_RELATIVE_PATH).is_file()
}

fn macro_crate_sibling(path: &Path) -> Option<PathBuf> {
    let parent = path.parent()?;
    let sibling = parent.join("linguini-cldr-macros");
    has_source_config(&sibling).then_some(sibling)
}

fn is_usable_cldr_source_dir(path: &Path) -> bool {
    path.join("cldr-json/cldr-core/supplemental/plurals.json")
        .is_file()
        && path.join("cldr-json/cldr-misc-full/main").is_dir()
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

fn main_source_path_from_source_dir(
    source_dir: PathBuf,
    relative_path: &str,
    fallback_path: &str,
) -> Result<PathBuf, String> {
    for candidate in [
        source_dir.join(relative_path),
        source_dir.join(fallback_path),
        source_dir.join("main"),
    ] {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "LINGUINI_CLDR_SOURCE_DIR={} does not contain {relative_path}",
        source_dir.display()
    ))
}
