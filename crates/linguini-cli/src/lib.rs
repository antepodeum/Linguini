use linguini_config::{
    discover_locale_files, discover_schema_files, parse_config, DEFAULT_CONFIG_FILE,
};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Config(linguini_config::ConfigError),
    Io { path: PathBuf, source: io::Error },
    MissingCommand,
    UnknownCommand(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => Display::fmt(error, f),
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
        Some(command) => Err(CliError::UnknownCommand(command.to_owned())),
        None => Err(CliError::MissingCommand),
    }
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
    let config_path = root.join(DEFAULT_CONFIG_FILE);
    let source = read_file(&config_path)?;
    let config = parse_config(&source)?;
    let schema_root = root.join(&config.paths.schema);
    let locale_root = root.join(&config.paths.locale);
    let schema_files = discover_schema_files(schema_root)?;
    let locale_files = discover_locale_files(locale_root)?;

    let mut output = String::new();
    output.push_str("schema files:\n");
    for file in schema_files {
        output.push_str(&format!(
            "- {} [{}]\n",
            path_for_output(root, &file.path),
            file.namespace
        ));
    }

    output.push_str("locale files:\n");
    for file in locale_files {
        output.push_str(&format!(
            "- {} [{}:{}]\n",
            path_for_output(root, &file.path),
            file.locale,
            file.namespace
        ));
    }

    Ok(output)
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

#[cfg(test)]
mod tests {
    use super::{check_project, init_project};
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
}
