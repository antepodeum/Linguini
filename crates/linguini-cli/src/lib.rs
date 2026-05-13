use linguini_analyzer::{render_diagnostics, Diagnostic};
use linguini_cldr::{
    cache_root, fetch_cldr_from_dir_for_locales, fetch_cldr_from_official_repo_for_locales,
    inspect_cache, require_offline_cache, OFFICIAL_CLDR_JSON_REPO,
};
use linguini_config::{
    discover_locale_files, discover_schema_files, parse_config, DEFAULT_CONFIG_FILE,
};
use linguini_syntax::{parse_locale_with_recovery, parse_schema_with_recovery};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Cldr(linguini_cldr::CldrCacheError),
    Config(linguini_config::ConfigError),
    Diagnostics(String),
    Io { path: PathBuf, source: io::Error },
    MissingCommand,
    UnknownCommand(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cldr(error) => Display::fmt(error, f),
            Self::Config(error) => Display::fmt(error, f),
            Self::Diagnostics(output) => f.write_str(output),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::MissingCommand => write!(f, "missing command"),
            Self::UnknownCommand(command) => write!(f, "unknown command `{command}`"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<linguini_config::ConfigError> for CliError {
    fn from(error: linguini_config::ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<linguini_cldr::CldrCacheError> for CliError {
    fn from(error: linguini_cldr::CldrCacheError) -> Self {
        Self::Cldr(error)
    }
}

pub fn run(
    args: impl IntoIterator<Item = String>,
    current_dir: io::Result<PathBuf>,
) -> CliResult<String> {
    let root = current_dir.map_err(|source| CliError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    let mut args = args.into_iter();

    match args.next().as_deref() {
        Some("init") => init_project(&root),
        Some("check") => check_project(&root),
        Some("build") => build_project(&root),
        Some("cldr") => cldr_command(&root, args.collect()),
        Some(command) => Err(CliError::UnknownCommand(command.to_owned())),
        None => Err(CliError::MissingCommand),
    }
}

pub fn cldr_command(root: &Path, args: Vec<String>) -> CliResult<String> {
    match args.first().map(String::as_str) {
        Some("status") => cldr_status(root),
        Some("fetch") => {
            let source = args.get(1).map(String::as_str);
            cldr_fetch(root, source)
        }
        Some(command) => Err(CliError::UnknownCommand(format!("cldr {command}"))),
        None => Err(CliError::MissingCommand),
    }
}

pub fn cldr_status(root: &Path) -> CliResult<String> {
    let config = read_project_config(root)?;
    let cache = cache_root(root, &config.paths.cache);
    let status = inspect_cache(&cache);
    let mut output = String::new();

    output.push_str(&format!("cldr cache: {}\n", path_for_output(root, &cache)));
    output.push_str(&format!("usable: {}\n", status.is_usable()));
    output.push_str(&format!("manifest: {}\n", status.manifest_exists));
    output.push_str(&format!("data: {}\n", status.data_dir_exists));
    output.push_str(&format!("plurals: {}\n", status.plurals_file_exists));
    for problem in status.problems {
        output.push_str(&format!("problem: {problem}\n"));
    }

    Ok(output)
}

pub fn cldr_fetch(root: &Path, source: Option<&str>) -> CliResult<String> {
    let config = read_project_config(root)?;
    let cache = cache_root(root, &config.paths.cache);
    let locales: Vec<_> = config.project.locales.iter().map(String::as_str).collect();
    let (status, source_label) = match source {
        None | Some(OFFICIAL_CLDR_JSON_REPO) => (
            fetch_cldr_from_official_repo_for_locales(&cache, locales)?,
            OFFICIAL_CLDR_JSON_REPO.to_owned(),
        ),
        Some(source_dir) => (
            fetch_cldr_from_dir_for_locales(Path::new(source_dir), &cache, locales)?,
            source_dir.to_owned(),
        ),
    };

    Ok(format!(
        "fetched CLDR data from {source_label}\ninto {}\nusable: {}\n",
        path_for_output(root, &status.root),
        status.is_usable()
    ))
}

pub fn build_project(root: &Path) -> CliResult<String> {
    let config = read_project_config(root)?;
    let cache = cache_root(root, &config.paths.cache);
    require_offline_cache(&cache)?;
    let check_output = check_project(root)?;

    Ok(format!("{check_output}build: ok\n"))
}

pub fn init_project(root: &Path) -> CliResult<String> {
    let schema_dir = root.join("linguini/schema");
    let locale_dir = root.join("linguini/locale");
    create_dir_all(&schema_dir)?;
    create_dir_all(&locale_dir)?;

    let config_path = root.join(DEFAULT_CONFIG_FILE);
    if !config_path.exists() {
        write_file(&config_path, default_config())?;
    }

    Ok(format!(
        "created {}\ncreated {}\ncreated {}\n",
        DEFAULT_CONFIG_FILE, "linguini/schema", "linguini/locale"
    ))
}

pub fn check_project(root: &Path) -> CliResult<String> {
    let config = read_project_config(root)?;
    let schema_root = root.join(&config.paths.schema);
    let locale_root = root.join(&config.paths.locale);
    let schema_files = discover_schema_files(schema_root)?;
    let locale_files = discover_locale_files(locale_root)?;
    let mut diagnostic_output = String::new();

    let mut output = String::new();
    output.push_str("schema files:\n");
    for file in &schema_files {
        output.push_str(&format!(
            "- {} [{}]\n",
            path_for_output(root, &file.path),
            file.namespace
        ));
        let source = read_file(&file.path)?;
        let parsed = parse_schema_with_recovery(&source);
        if !parsed.errors.is_empty() {
            diagnostic_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                parsed.errors,
            ));
        }
    }

    output.push_str("locale files:\n");
    for file in &locale_files {
        output.push_str(&format!(
            "- {} [{}:{}]\n",
            path_for_output(root, &file.path),
            file.locale,
            file.namespace
        ));
        let source = read_file(&file.path)?;
        let parsed = parse_locale_with_recovery(&source);
        if !parsed.errors.is_empty() {
            diagnostic_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "locale syntax error",
                parsed.errors,
            ));
        }
    }

    if !diagnostic_output.is_empty() {
        return Err(CliError::Diagnostics(diagnostic_output));
    }

    Ok(output)
}

fn read_project_config(root: &Path) -> CliResult<linguini_config::LinguiniConfig> {
    let config_path = root.join(DEFAULT_CONFIG_FILE);
    let source = read_file(&config_path)?;
    Ok(parse_config(&source)?)
}

fn default_config() -> &'static str {
    r#"[project]
name = "linguini-app"
default_locale = "en"
locales = ["en"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"
cache = ".linguini/cache"
"#
}

fn path_for_output(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn create_dir_all(path: &Path) -> CliResult<()> {
    fs::create_dir_all(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_file(path: &Path) -> CliResult<String> {
    fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, contents: &str) -> CliResult<()> {
    fs::write(path, contents).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn render_parse_errors(
    root: &Path,
    path: &Path,
    source: &str,
    note: &str,
    errors: Vec<linguini_syntax::ParseError>,
) -> String {
    let relative_path = path_for_output(root, path);
    let diagnostics: Vec<_> = errors
        .into_iter()
        .map(|error| Diagnostic::error(error.message, error.span).with_note(note))
        .collect();

    render_diagnostics(&relative_path, source, &diagnostics).unwrap_or_else(|error| {
        format!("failed to render diagnostics for {relative_path}: {error}")
    })
}

#[cfg(test)]
mod tests {
    use super::{build_project, check_project, cldr_fetch, cldr_status, init_project};
    use linguini_test_support::temp_project_dir;
    use std::fs;

    #[test]
    fn init_creates_valid_project() {
        let project = temp_project_dir("init_creates_valid_project");

        init_project(project.path()).expect("init project");

        assert!(project.path().join("linguini.toml").exists());
        assert!(project.path().join("linguini/schema").is_dir());
        assert!(project.path().join("linguini/locale").is_dir());
    }

    #[test]
    fn check_lists_discovered_files() {
        let project = temp_project_dir("check_lists_discovered_files");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("linguini/schema/shop");
        let locale_dir = project.path().join("linguini/locale/shop/delivery");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::create_dir_all(&locale_dir).expect("locale dir");
        fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
        fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

        let output = check_project(project.path()).expect("check project");

        assert!(output.contains("linguini/schema/shop/delivery.lqs [shop.delivery]"));
        assert!(output.contains("linguini/locale/shop/delivery/en.lgl [en:shop.delivery]"));
    }

    #[test]
    fn check_reports_schema_syntax_diagnostics() {
        let project = temp_project_dir("check_reports_schema_syntax_diagnostics");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("linguini/schema/shop");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::write(schema_dir.join("broken.lqs"), "delivery(fruit: Fruit\n").expect("schema file");

        let error = check_project(project.path()).expect_err("check fails");
        let rendered = error.to_string();

        assert!(rendered.contains("Error:"));
        assert!(rendered.contains("linguini/schema/shop/broken.lqs"));
        assert!(rendered.contains("schema syntax error"));
    }

    #[test]
    fn cldr_status_reports_missing_cache() {
        let project = temp_project_dir("cldr_status_reports_missing_cache");
        init_project(project.path()).expect("init project");

        let output = cldr_status(project.path()).expect("cldr status");

        assert!(output.contains("usable: false"));
        assert!(output.contains("plurals: false"));
    }

    #[test]
    fn cldr_fetch_imports_staged_cldr_json_cache() {
        let project = temp_project_dir("cldr_fetch_imports_staged_cldr_json_cache");
        init_project(project.path()).expect("init project");
        let source = project.path().join("source");
        let supplemental = source.join("cldr-json/cldr-core/supplemental");
        let numbers = source.join("cldr-json/cldr-numbers-full/main/en");
        let dates = source.join("cldr-json/cldr-dates-full/main/en");
        fs::create_dir_all(&supplemental).expect("supplemental dir");
        fs::create_dir_all(&numbers).expect("numbers dir");
        fs::create_dir_all(&dates).expect("dates dir");
        fs::write(supplemental.join("plurals.json"), "{}\n").expect("plural data");
        fs::write(numbers.join("numbers.json"), "{}\n").expect("numbers data");
        fs::write(dates.join("ca-gregorian.json"), "{}\n").expect("calendar data");

        let output =
            cldr_fetch(project.path(), Some(source.to_string_lossy().as_ref())).expect("fetch");

        assert!(output.contains("source"));
        assert!(output.contains("usable: true"));
        assert!(cldr_status(project.path())
            .expect("cldr status")
            .contains("usable: true"));
    }

    #[test]
    fn build_requires_offline_cldr_cache_without_fetching() {
        let project = temp_project_dir("build_requires_offline_cldr_cache");
        init_project(project.path()).expect("init project");

        let error = build_project(project.path()).expect_err("missing cache");

        assert!(error.to_string().contains("run `linguini cldr fetch`"));
    }
}
