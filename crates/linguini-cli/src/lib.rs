use clap::{Parser, Subcommand};
use linguini_analyzer::{render_diagnostics, Diagnostic};
use linguini_codegen_ts::{generate_plural_function, generate_typescript_files, TypeScriptOptions};
use linguini_config::{
    discover_locale_files, discover_schema_files, parse_config, LinguiniConfig,
    TypeScriptTargetConfig, DEFAULT_CONFIG_FILE,
};
use linguini_ir::{ensure_no_unresolved_references, lower_locale, lower_schema, IrModule};
use linguini_syntax::{
    parse_locale, parse_locale_with_recovery, parse_schema, parse_schema_with_recovery,
};
use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Args(clap::Error),
    Config(linguini_config::ConfigError),
    Diagnostics(String),
    Io { path: PathBuf, source: io::Error },
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Args(error) => Display::fmt(error, f),
            Self::Config(error) => Display::fmt(error, f),
            Self::Diagnostics(output) => f.write_str(output),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for CliError {}

impl From<clap::Error> for CliError {
    fn from(error: clap::Error) -> Self {
        Self::Args(error)
    }
}

impl From<linguini_config::ConfigError> for CliError {
    fn from(error: linguini_config::ConfigError) -> Self {
        Self::Config(error)
    }
}

#[derive(Debug, Parser)]
#[command(name = "linguini", about = "Experimental localization toolkit CLI")]
pub struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Subcommand)]
enum CliCommand {
    /// Create a Linguini project skeleton
    Init,
    /// Parse configured schema and locale files and report diagnostics
    Check,
    /// Validate the project using built-in CLDR rules compiled into this binary
    Build,
}

pub fn run(
    args: impl IntoIterator<Item = String>,
    current_dir: io::Result<PathBuf>,
) -> CliResult<String> {
    let root = current_dir.map_err(|source| CliError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    let cli = Cli::try_parse_from(std::iter::once("linguini".to_owned()).chain(args))?;

    match cli.command {
        CliCommand::Init => init_project(&root),
        CliCommand::Check => check_project(&root),
        CliCommand::Build => build_project(&root),
    }
}

pub fn build_project(root: &Path) -> CliResult<String> {
    let check_output = check_project(root)?;
    let config = read_project_config(root)?;
    let codegen_output = generate_project(root, &config)?;

    Ok(format!("{check_output}{codegen_output}build: ok\n"))
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

fn generate_project(root: &Path, config: &LinguiniConfig) -> CliResult<String> {
    let Some(target) = &config.targets.ts else {
        return Ok("codegen targets: none\n".to_owned());
    };

    generate_typescript_target(root, config, target)
}

fn generate_typescript_target(
    root: &Path,
    config: &LinguiniConfig,
    target: &TypeScriptTargetConfig,
) -> CliResult<String> {
    let schema = load_schema_ir(root, config)?;
    let locale_files = discover_locale_files(root.join(&config.paths.locale))?;
    let out_dir = root.join(&target.out);
    let mut output = String::from("generated files:\n");
    let mut written_shared = BTreeSet::new();
    let mut generated_locales = Vec::new();

    for locale in &config.project.locales {
        let mut locale_ir = IrModule::default();
        let mut matched_locale_files = Vec::new();
        for file in locale_files.iter().filter(|file| &file.locale == locale) {
            matched_locale_files.push(path_for_output(root, &file.path));
            let source = read_file(&file.path)?;
            let parsed = parse_locale(&source).map_err(|errors| {
                CliError::Diagnostics(render_parse_errors(
                    root,
                    &file.path,
                    &source,
                    "locale syntax error",
                    errors,
                ))
            })?;
            merge_module(&mut locale_ir, lower_locale(&parsed));
        }

        if matched_locale_files.is_empty() {
            return Err(CliError::Diagnostics(format!(
                "missing locale files for configured locale `{locale}`\n"
            )));
        }

        if let Err(errors) = ensure_no_unresolved_references(&schema, &locale_ir) {
            let rendered = errors
                .into_iter()
                .map(|error| format!("{locale}: {}", error.message))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(CliError::Diagnostics(format!("{rendered}\n")));
        }

        let plural_rules = linguini_cldr::built_in_plural_rules(locale).ok_or_else(|| {
            CliError::Diagnostics(format!(
                "missing built-in CLDR plural rules for configured locale `{locale}`\n"
            ))
        })?;
        let plural_function = format!("plural{}", pascal_identifier(locale));
        let mut files = generate_typescript_files(
            &schema,
            &locale_ir,
            &TypeScriptOptions {
                locale: locale.clone(),
                plural_function: plural_function.clone(),
                plural_import: None,
                plural_source: Some(generate_plural_function(&plural_function, &plural_rules)),
            },
        );

        if !target.declaration {
            files.retain(|file| !file.path.ends_with(".d.ts"));
        }

        for file in files {
            if file.path == "index.ts" || file.path == "index.d.ts" {
                continue;
            }
            if (file.path == "shared.ts" || file.path == "shared.d.ts")
                && !written_shared.insert(file.path.clone())
            {
                continue;
            }
            let destination = out_dir.join(&file.path);
            if let Some(parent) = destination.parent() {
                create_dir_all(parent)?;
            }
            write_file(&destination, &file.contents)?;
            output.push_str(&format!("- {}\n", path_for_output(root, &destination)));
        }
        generated_locales.push(locale.clone());
    }

    let index = generate_typescript_index(&generated_locales);
    let index_path = out_dir.join("index.ts");
    if let Some(parent) = index_path.parent() {
        create_dir_all(parent)?;
    }
    write_file(&index_path, &index)?;
    output.push_str(&format!("- {}\n", path_for_output(root, &index_path)));

    if target.declaration {
        let declaration = generate_typescript_index_declaration(&generated_locales);
        let declaration_path = out_dir.join("index.d.ts");
        if let Some(parent) = declaration_path.parent() {
            create_dir_all(parent)?;
        }
        write_file(&declaration_path, &declaration)?;
        output.push_str(&format!("- {}\n", path_for_output(root, &declaration_path)));
    }

    Ok(output)
}

fn load_schema_ir(root: &Path, config: &LinguiniConfig) -> CliResult<IrModule> {
    let mut schema = IrModule::default();
    for file in discover_schema_files(root.join(&config.paths.schema))? {
        let source = read_file(&file.path)?;
        let parsed = parse_schema(&source).map_err(|errors| {
            CliError::Diagnostics(render_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                errors,
            ))
        })?;
        merge_module(&mut schema, lower_schema(&parsed));
    }
    Ok(schema)
}

fn merge_module(target: &mut IrModule, source: IrModule) {
    target.enums.extend(source.enums);
    target.type_aliases.extend(source.type_aliases);
    target.messages.extend(source.messages);
    target.forms.extend(source.forms);
    target.functions.extend(source.functions);
}

fn generate_typescript_index(locales: &[String]) -> String {
    let mut output = String::new();
    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(locale),
            escape_typescript_string(locale)
        ));
    }
    output.push('\n');
    output.push_str("const localeModules = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: {},\n",
            typescript_property_key(locale),
            locale_identifier(locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("export function createLinguini(language: LinguiniLanguage): Linguini {\n");
    output.push_str("  return localeModules[language];\n");
    output.push_str("}\n\n");
    output.push_str("export function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguage | (() => LinguiniLanguage);\n");
    output.push_str("}): { readonly lgl: Linguini } {\n");
    output.push_str("  return {\n");
    output.push_str("    get lgl() {\n");
    output.push_str("      const language = typeof options.language === \"function\" ? options.language() : options.language;\n");
    output.push_str("      return createLinguini(language);\n");
    output.push_str("    },\n");
    output.push_str("  };\n");
    output.push_str("}\n");
    output
}

fn generate_typescript_index_declaration(locales: &[String]) -> String {
    let mut output = String::new();
    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(locale),
            escape_typescript_string(locale)
        ));
    }
    output.push('\n');
    output.push_str("declare const localeModules: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: typeof {};\n",
            typescript_property_key(locale),
            locale_identifier(locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("export declare function createLinguini(language: LinguiniLanguage): Linguini;\n\n");
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguage | (() => LinguiniLanguage);\n");
    output.push_str("}): { readonly lgl: Linguini };\n");
    output
}

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}

fn pascal_identifier(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut output = String::new();
            output.push(first.to_ascii_uppercase());
            output.extend(chars.map(|character| character.to_ascii_lowercase()));
            output
        })
        .collect::<String>()
}

fn safe_identifier(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character);
        } else {
            output.push('_');
        }
    }
    if output
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        output.insert(0, '_');
    }
    output
}

fn typescript_property_key(value: &str) -> String {
    if value
        .bytes()
        .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    {
        value.to_owned()
    } else {
        format!("\"{}\"", escape_typescript_string(value))
    }
}

fn escape_typescript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
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

[targets.ts]
out = "src/generated/linguini"
module = "esm"
declaration = true
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
    use super::{build_project, check_project, init_project, Cli};
    use clap::CommandFactory;
    use linguini_test_support::temp_project_dir;
    use std::fs;

    #[test]
    fn cli_argument_parser_is_clap_backed() {
        let command = Cli::command();
        let subcommands: Vec<_> = command
            .get_subcommands()
            .map(|command| command.get_name().to_owned())
            .collect();

        assert!(subcommands.contains(&"init".to_owned()));
        assert!(subcommands.contains(&"check".to_owned()));
        assert!(subcommands.contains(&"build".to_owned()));
        assert!(!subcommands.contains(&"cldr".to_owned()));
    }

    #[test]
    fn init_creates_valid_project() {
        let project = temp_project_dir("init_creates_valid_project");

        init_project(project.path()).expect("init project");

        assert!(project.path().join("linguini.toml").exists());
        assert!(project.path().join("linguini/schema").is_dir());
        assert!(project.path().join("linguini/locale").is_dir());
        let config = fs::read_to_string(project.path().join("linguini.toml")).expect("config");
        assert!(!config.contains("cache"));
        assert!(config.contains("[targets.ts]"));
        assert!(config.contains("out = \"src/generated/linguini\""));
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
    fn build_generates_typescript_project_files_without_cldr_cache() {
        let project = temp_project_dir("build_generates_typescript");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("linguini/schema/shop");
        let locale_dir = project.path().join("linguini/locale/shop/delivery");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::create_dir_all(&locale_dir).expect("locale dir");
        fs::write(schema_dir.join("delivery.lqs"), "delivery(count: Number)\n").expect("schema file");
        fs::write(
            locale_dir.join("en.lgl"),
            "delivery = {count} deliveries\n",
        )
        .expect("locale file");

        let output = build_project(project.path()).expect("build project");

        assert!(output.contains("schema files:"));
        assert!(output.contains("locale files:"));
        assert!(output.contains("generated files:"));
        assert!(output.contains("src/generated/linguini/locales/en.ts"));
        assert!(output.contains("build: ok"));
        assert!(project
            .path()
            .join("src/generated/linguini/locales/en.ts")
            .exists());
        assert!(project.path().join("src/generated/linguini/index.ts").exists());
        assert!(!project.path().join(".linguini/cache").exists());
    }
}
