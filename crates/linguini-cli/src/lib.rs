use clap::{Args, Parser, Subcommand};
use linguini_analyzer::{
    analyze_locale_coverage_with_options, analyze_locale_file, locale_public_messages,
    render_diagnostics_with_color, schema_public_messages, Diagnostic, DiagnosticSeverity,
    LocaleCoverageOptions, QuickFix,
};
use linguini_codegen_ts::{generate_plural_function, generate_typescript_files, TypeScriptOptions};
use linguini_config::{
    discover_locale_files, discover_schema_files, parse_config, LinguiniConfig,
    LocaleFile as DiscoveredLocaleFile, SchemaFile as DiscoveredSchemaFile,
    TypeScriptTargetConfig, DEFAULT_CONFIG_FILE,
};
use linguini_ir::{ensure_no_unresolved_references, lower_locale, lower_schema, IrModule};
use linguini_syntax::{
    parse_locale, parse_locale_with_recovery, parse_schema, parse_schema_with_recovery,
    LocaleFile as LocaleAst, SchemaFile as SchemaAst, Span,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug, Clone)]
struct ParsedSchemaSource {
    file: DiscoveredSchemaFile,
    source: String,
    ast: SchemaAst,
}

#[derive(Debug, Clone)]
struct ParsedLocaleSource {
    file: DiscoveredLocaleFile,
    source: String,
    ast: LocaleAst,
}

#[derive(Debug, Default)]
struct ProjectDiagnosticOutput {
    errors: String,
    warnings: String,
}

impl ProjectDiagnosticOutput {
    fn push(
        &mut self,
        root: &Path,
        path: &Path,
        source: &str,
        diagnostics: &[Diagnostic],
    ) {
        let errors = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .cloned()
            .collect::<Vec<_>>();
        let warnings = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error)
            .cloned()
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            self.errors
                .push_str(&render_file_diagnostics(root, path, source, &errors));
        }
        if !warnings.is_empty() {
            self.warnings
                .push_str(&render_file_diagnostics(root, path, source, &warnings));
        }
    }
}

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

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
enum CliCommand {
    /// Create a Linguini project skeleton
    Init,
    /// Parse configured schema and locale files and report diagnostics
    Check,
    /// Apply analyzer quick fixes such as missing locale files and message stubs
    Fix(FixArgs),
    /// Validate the project using built-in CLDR rules compiled into this binary
    Build,
}

#[derive(Debug, Clone, Eq, PartialEq, Args)]
struct FixArgs {
    /// Apply every available automatic fix
    #[arg(long)]
    all: bool,
    /// Fix ids printed by `linguini check`, for example `missing-messages:shop:ru`
    ids: Vec<String>,
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
        CliCommand::Fix(args) => fix_project(&root, &args),
        CliCommand::Build => build_project(&root),
    }
}

pub fn build_project(root: &Path) -> CliResult<String> {
    let check_output = check_project(root)?;
    let config = read_project_config(root)?;
    let codegen_output = generate_project(root, &config)?;

    Ok(format!("{check_output}{codegen_output}build: ok\n"))
}

#[derive(Debug, Clone)]
struct ProjectFix {
    id: String,
    title: String,
    operation: FixOperation,
}

#[derive(Debug, Clone)]
enum FixOperation {
    CreateFile {
        path: PathBuf,
        contents: String,
    },
    AppendFile {
        path: PathBuf,
        contents: String,
    },
}

fn fix_project(root: &Path, args: &FixArgs) -> CliResult<String> {
    let config = read_project_config(root)?;
    let fixes = available_project_fixes(root, &config)?;

    if !args.all && args.ids.is_empty() {
        let mut output = String::new();
        if fixes.is_empty() {
            output.push_str("no automatic fixes available\n");
            return Ok(output);
        }

        output.push_str("available fixes:\n");
        for fix in &fixes {
            output.push_str(&format!("- {}: {}\n", fix.id, fix.title));
        }
        output.push_str("run `linguini fix --all` or `linguini fix <id>...`\n");
        return Ok(output);
    }

    let selected = select_fixes(&fixes, args)?;
    let mut output = String::new();

    for fix in selected {
        apply_project_fix(root, &fix, &mut output)?;
    }

    if output.is_empty() {
        output.push_str("no automatic fixes applied\n");
    }

    Ok(output)
}

fn available_project_fixes(root: &Path, config: &LinguiniConfig) -> CliResult<Vec<ProjectFix>> {
    let schema_files = load_schema_sources(root, config)?;
    let locale_files = load_locale_sources(root, config)?;
    let locale_index = locale_index(&locale_files);
    let mut fixes = Vec::new();

    for schema_file in &schema_files {
        let schema_messages = schema_public_messages(&schema_file.ast);
        let schema_message_names = schema_messages
            .iter()
            .map(|message| message.name.clone())
            .collect::<Vec<_>>();

        for locale in &config.project.locales {
            let key = (schema_file.file.namespace.clone(), locale.clone());
            match locale_index.get(&key) {
                Some(locale_file) => {
                    let locale_messages = locale_public_messages(&locale_file.ast);
                    let missing = missing_schema_message_names(&schema_messages, &locale_messages);

                    if missing.is_empty() {
                        continue;
                    }

                    fixes.push(ProjectFix {
                        id: missing_messages_fix_id(&schema_file.file.namespace, locale),
                        title: format!(
                            "add {} missing message {} to {}",
                            missing.len(),
                            pluralize(missing.len(), "stub", "stubs"),
                            path_for_output(root, &locale_file.file.path)
                        ),
                        operation: FixOperation::AppendFile {
                            path: locale_file.file.path.clone(),
                            contents: append_stub_text(&locale_file.source, &missing),
                        },
                    });
                }
                None => {
                    let path =
                        expected_locale_path(root, config, &schema_file.file.namespace, locale);
                    fixes.push(ProjectFix {
                        id: missing_locale_fix_id(&schema_file.file.namespace, locale),
                        title: format!("create locale file {}", path_for_output(root, &path)),
                        operation: FixOperation::CreateFile {
                            path,
                            contents: locale_stub_text(&schema_message_names),
                        },
                    });
                }
            }
        }
    }

    Ok(fixes)
}

fn select_fixes(fixes: &[ProjectFix], args: &FixArgs) -> CliResult<Vec<ProjectFix>> {
    if args.all {
        return Ok(fixes.to_vec());
    }

    let by_id = fixes
        .iter()
        .map(|fix| (fix.id.as_str(), fix))
        .collect::<BTreeMap<_, _>>();
    let mut selected = Vec::new();
    let mut missing = Vec::new();

    for id in &args.ids {
        match by_id.get(id.as_str()) {
            Some(fix) => selected.push((*fix).clone()),
            None => missing.push(id.clone()),
        }
    }

    if !missing.is_empty() {
        return Err(CliError::Diagnostics(format!(
            "unknown fix {}: {}\nrun `linguini fix` to list available fixes\n",
            pluralize(missing.len(), "id", "ids"),
            missing.join(", ")
        )));
    }

    Ok(selected)
}

fn apply_project_fix(root: &Path, fix: &ProjectFix, output: &mut String) -> CliResult<()> {
    match &fix.operation {
        FixOperation::CreateFile { path, contents } => {
            if let Some(parent) = path.parent() {
                create_dir_all(parent)?;
            }
            if !path.exists() {
                write_file(path, contents)?;
            }
            output.push_str(&format!(
                "applied {}: created {}\n",
                fix.id,
                path_for_output(root, path)
            ));
        }
        FixOperation::AppendFile { path, contents } => {
            let mut existing = read_file(path)?;
            existing.push_str(contents);
            write_file(path, &existing)?;
            output.push_str(&format!(
                "applied {}: updated {}\n",
                fix.id,
                path_for_output(root, path)
            ));
        }
    }

    Ok(())
}

fn missing_schema_message_names(
    schema_messages: &[linguini_analyzer::RequiredLocaleMessage],
    locale_messages: &[linguini_analyzer::ImplementedLocaleMessage],
) -> Vec<String> {
    let locale_names = locale_messages
        .iter()
        .map(|message| message.name.as_str())
        .collect::<BTreeSet<_>>();

    schema_messages
        .iter()
        .filter(|message| !locale_names.contains(message.name.as_str()))
        .map(|message| message.name.clone())
        .collect()
}

fn append_stub_text(existing: &str, names: &[String]) -> String {
    let mut output = String::new();
    if !existing.is_empty() && !existing.ends_with('\n') {
        output.push('\n');
    }
    if !existing.is_empty() {
        output.push('\n');
    }
    output.push_str(&locale_stub_text(names));
    output
}

fn locale_stub_text(names: &[String]) -> String {
    let mut output = String::new();
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut top_level = Vec::new();

    for name in names {
        if let Some((group, message)) = name.split_once('.') {
            groups.entry(group).or_default().push(message);
        } else {
            top_level.push(name.as_str());
        }
    }

    for name in top_level {
        output.push_str(&format!("{name} = TODO\n"));
    }

    for (group, messages) in groups {
        output.push_str(&format!("{group} {{\n"));
        for message in messages {
            output.push_str(&format!("  {message} = TODO\n"));
        }
        output.push_str("}\n");
    }

    output
}

fn missing_locale_fix_id(namespace: &str, locale: &str) -> String {
    format!("missing-locale:{}:{}", fix_id_namespace(namespace), locale)
}

fn missing_messages_fix_id(namespace: &str, locale: &str) -> String {
    format!("missing-messages:{}:{}", fix_id_namespace(namespace), locale)
}

fn fix_id_namespace(namespace: &str) -> String {
    if namespace.is_empty() {
        "root".to_owned()
    } else {
        namespace.to_owned()
    }
}

pub fn init_project(root: &Path) -> CliResult<String> {
    let schema_dir = root.join("schema");
    let locale_dir = root.join("locales");
    create_dir_all(&schema_dir)?;
    create_dir_all(&locale_dir)?;

    let config_path = root.join(DEFAULT_CONFIG_FILE);
    if !config_path.exists() {
        write_file(&config_path, default_config())?;
    }

    Ok(format!(
        "created {}\ncreated {}\ncreated {}\n",
        DEFAULT_CONFIG_FILE, "schema", "locales"
    ))
}

pub fn check_project(root: &Path) -> CliResult<String> {
    let config = read_project_config(root)?;
    let schema_files = discover_schema_files(root.join(&config.paths.schema))?;
    let locale_files = discover_locale_files(root.join(&config.paths.locale))?;
    let mut parsed_schema_files = Vec::new();
    let mut parsed_locale_files = Vec::new();
    let mut error_output = String::new();
    let mut warning_output = String::new();

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
            error_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                parsed.errors,
            ));
        }
        if let Some(ast) = parsed.ast {
            parsed_schema_files.push(ParsedSchemaSource {
                file: file.clone(),
                source,
                ast,
            });
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
            error_output.push_str(&render_parse_errors(
                root,
                &file.path,
                &source,
                "locale syntax error",
                parsed.errors,
            ));
        } else if let Some(locale) = parsed.ast.as_ref() {
            let diagnostics = analyze_locale_file(locale);
            if !diagnostics.is_empty() {
                warning_output.push_str(&render_file_diagnostics(
                    root,
                    &file.path,
                    &source,
                    &diagnostics,
                ));
            }
        }
        if let Some(ast) = parsed.ast {
            parsed_locale_files.push(ParsedLocaleSource {
                file: file.clone(),
                source,
                ast,
            });
        }
    }

    if error_output.is_empty() {
        let project_diagnostics = render_project_coverage_diagnostics(
            root,
            &config,
            &parsed_schema_files,
            &parsed_locale_files,
        )?;
        error_output.push_str(&project_diagnostics.errors);
        warning_output.push_str(&project_diagnostics.warnings);
    }

    if !error_output.is_empty() {
        return Err(CliError::Diagnostics(error_output));
    }

    if !warning_output.is_empty() {
        output.push_str(&warning_output);
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
    let schema_files = load_schema_sources(root, config)?;
    let schema_namespaces: BTreeSet<_> = schema_files
        .iter()
        .map(|schema| schema.file.namespace.clone())
        .collect();
    let schema = merge_schema_ir(&schema_files);
    let locale_files = load_locale_sources(root, config)?;
    let locale_index = locale_index(&locale_files);
    let out_dir = root.join(&target.out);
    let mut output = String::from("generated files:\n");
    let mut written_shared = BTreeSet::new();
    let mut generated_locales = Vec::new();

    reject_locale_files_without_schema_namespace(root, &schema_namespaces, &locale_files)?;

    for locale in &config.project.locales {
        let mut locale_ir = IrModule::default();

        for schema_file in &schema_files {
            let locale_key = (schema_file.file.namespace.clone(), locale.clone());
            let default_key = (
                schema_file.file.namespace.clone(),
                config.project.default_locale.clone(),
            );
            let locale_file = locale_index.get(&locale_key);

            if locale == &config.project.default_locale && locale_file.is_none() {
                let path = expected_locale_path(root, config, &schema_file.file.namespace, locale);
                return Err(CliError::Diagnostics(format!(
                    "required locale file is missing for schema namespace `{}`: `{locale}`\nexpected path: {}\n",
                    schema_file.file.namespace,
                    path_for_output(root, &path)
                )));
            }

            if let Some(locale_file) = locale_file {
                let diagnostics = analyze_locale_coverage_with_options(
                    &schema_file.ast,
                    &locale_file.ast,
                    coverage_options(config, &schema_file.file.namespace, locale),
                );
                let blocking = diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
                    .cloned()
                    .collect::<Vec<_>>();
                if !blocking.is_empty() {
                    return Err(CliError::Diagnostics(render_file_diagnostics(
                        root,
                        &locale_file.file.path,
                        &locale_file.source,
                        &blocking,
                    )));
                }

                merge_module(&mut locale_ir, lower_locale(&locale_file.ast));
            }

            if locale != &config.project.default_locale {
                if let Some(default_locale_file) = locale_index.get(&default_key) {
                    merge_module_fallback(&mut locale_ir, lower_locale(&default_locale_file.ast));
                }
            }
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

fn load_schema_sources(root: &Path, config: &LinguiniConfig) -> CliResult<Vec<ParsedSchemaSource>> {
    let mut parsed = Vec::new();
    for file in discover_schema_files(root.join(&config.paths.schema))? {
        let source = read_file(&file.path)?;
        let ast = parse_schema(&source).map_err(|errors| {
            CliError::Diagnostics(render_parse_errors(
                root,
                &file.path,
                &source,
                "schema syntax error",
                errors,
            ))
        })?;
        parsed.push(ParsedSchemaSource { file, source, ast });
    }
    Ok(parsed)
}

fn load_locale_sources(root: &Path, config: &LinguiniConfig) -> CliResult<Vec<ParsedLocaleSource>> {
    let mut parsed = Vec::new();
    for file in discover_locale_files(root.join(&config.paths.locale))? {
        let source = read_file(&file.path)?;
        let ast = parse_locale(&source).map_err(|errors| {
            CliError::Diagnostics(render_parse_errors(
                root,
                &file.path,
                &source,
                "locale syntax error",
                errors,
            ))
        })?;
        parsed.push(ParsedLocaleSource { file, source, ast });
    }
    Ok(parsed)
}

fn merge_schema_ir(schema_files: &[ParsedSchemaSource]) -> IrModule {
    let mut schema = IrModule::default();
    for file in schema_files {
        merge_module(&mut schema, lower_schema(&file.ast));
    }
    schema
}

fn locale_index<'a>(
    locale_files: &'a [ParsedLocaleSource],
) -> BTreeMap<(String, String), &'a ParsedLocaleSource> {
    locale_files
        .iter()
        .map(|file| {
            (
                (file.file.namespace.clone(), file.file.locale.clone()),
                file,
            )
        })
        .collect()
}

fn render_project_coverage_diagnostics(
    root: &Path,
    config: &LinguiniConfig,
    schema_files: &[ParsedSchemaSource],
    locale_files: &[ParsedLocaleSource],
) -> CliResult<ProjectDiagnosticOutput> {
    let schema_namespaces: BTreeSet<_> = schema_files
        .iter()
        .map(|schema| schema.file.namespace.clone())
        .collect();
    let locale_index = locale_index(locale_files);
    let mut output = ProjectDiagnosticOutput::default();

    for schema_file in schema_files {
        let mut missing_default_locale = Vec::new();
        let mut missing_secondary_locales = Vec::new();

        for locale in &config.project.locales {
            match locale_index.get(&(schema_file.file.namespace.clone(), locale.clone())) {
                Some(locale_file) => {
                    let diagnostics = analyze_locale_coverage_with_options(
                        &schema_file.ast,
                        &locale_file.ast,
                        coverage_options(config, &schema_file.file.namespace, locale),
                    );
                    output.push(
                        root,
                        &locale_file.file.path,
                        &locale_file.source,
                        &diagnostics,
                    );
                }
                None if locale == &config.project.default_locale => {
                    missing_default_locale.push(locale.clone());
                }
                None => {
                    missing_secondary_locales.push(locale.clone());
                }
            }
        }

        if !missing_default_locale.is_empty() {
            let diagnostic = missing_locale_files_diagnostic(
                config,
                &schema_file.file.namespace,
                &missing_default_locale,
                DiagnosticSeverity::Error,
                Span::new(0, 0),
                root,
            );
            output.push(root, &schema_file.file.path, &schema_file.source, &[diagnostic]);
        }

        if !missing_secondary_locales.is_empty() {
            let diagnostic = missing_locale_files_diagnostic(
                config,
                &schema_file.file.namespace,
                &missing_secondary_locales,
                DiagnosticSeverity::Warning,
                Span::new(0, 0),
                root,
            );
            output.push(root, &schema_file.file.path, &schema_file.source, &[diagnostic]);
        }
    }

    for locale_files in locale_files_without_schema_namespace(locale_files, &schema_namespaces) {
        let primary = &locale_files[0];
        let affected = locale_files
            .iter()
            .map(|file| path_for_output(root, &file.file.path))
            .collect::<Vec<_>>()
            .join(", ");
        let diagnostic = Diagnostic::error(
            format!(
                "locale namespace `{}` has no matching schema namespace",
                namespace_display(&primary.file.namespace)
            ),
            Span::new(0, 0),
        )
        .without_source()
        .with_note(format!(
            "move these files under locales/<schema-namespace>/<locale>.lgl: {affected}"
        ));
        output.push(root, &primary.file.path, &primary.source, &[diagnostic]);
    }

    Ok(output)
}

fn reject_locale_files_without_schema_namespace(
    root: &Path,
    schema_namespaces: &BTreeSet<String>,
    locale_files: &[ParsedLocaleSource],
) -> CliResult<()> {
    for locale_file in locale_files {
        if !schema_namespaces.contains(&locale_file.file.namespace) {
            return Err(CliError::Diagnostics(render_file_diagnostics(
                root,
                &locale_file.file.path,
                &locale_file.source,
                &[Diagnostic::error(
                    format!(
                        "locale namespace `{}` has no matching schema namespace",
                        namespace_display(&locale_file.file.namespace)
                    ),
                    Span::new(0, 0),
                )
                .without_source()
                .with_note("move this file under locales/<schema-namespace>/<locale>.lgl")],
            )));
        }
    }
    Ok(())
}

fn coverage_options(
    config: &LinguiniConfig,
    namespace: &str,
    locale: &str,
) -> LocaleCoverageOptions {
    LocaleCoverageOptions {
        missing_message_severity: if locale == config.project.default_locale {
            DiagnosticSeverity::Error
        } else {
            DiagnosticSeverity::Warning
        },
        subject: format!(
            "locale `{locale}` for schema namespace `{}`",
            namespace_display(namespace)
        ),
        quick_fix_id: Some(missing_messages_fix_id(namespace, locale)),
    }
}

fn missing_locale_files_diagnostic(
    config: &LinguiniConfig,
    namespace: &str,
    locales: &[String],
    severity: DiagnosticSeverity,
    span: linguini_syntax::Span,
    root: &Path,
) -> Diagnostic {
    let expected_paths = locales
        .iter()
        .map(|locale| {
            path_for_output(root, &expected_locale_path(root, config, namespace, locale))
        })
        .collect::<Vec<_>>();
    let message = format!(
        "{} locale {} missing for schema namespace `{}`: {}",
        if severity == DiagnosticSeverity::Error {
            "required"
        } else {
            "secondary"
        },
        pluralize(locales.len(), "file is", "files are"),
        namespace_display(namespace),
        locales
            .iter()
            .map(|locale| format!("`{locale}`"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    let note = if expected_paths.len() == 1 {
        format!("expected path: {}", expected_paths[0])
    } else {
        format!("expected paths: {}", expected_paths.join(", "))
    };

    let mut diagnostic = match severity {
        DiagnosticSeverity::Error => Diagnostic::error(message, span),
        DiagnosticSeverity::Warning => Diagnostic::warning(message, span),
        DiagnosticSeverity::Advice => Diagnostic::advice(message, span),
    }
    .without_source()
    .with_note(note);

    for locale in locales {
        diagnostic = diagnostic.with_quick_fix(QuickFix::command(
            missing_locale_fix_id(namespace, locale),
            format!("create locale file for `{locale}`"),
        ));
    }

    diagnostic
}

fn locale_files_without_schema_namespace<'a>(
    locale_files: &'a [ParsedLocaleSource],
    schema_namespaces: &BTreeSet<String>,
) -> Vec<Vec<&'a ParsedLocaleSource>> {
    let mut grouped: BTreeMap<&str, Vec<&ParsedLocaleSource>> = BTreeMap::new();
    for locale_file in locale_files {
        if !schema_namespaces.contains(&locale_file.file.namespace) {
            grouped
                .entry(locale_file.file.namespace.as_str())
                .or_default()
                .push(locale_file);
        }
    }
    grouped.into_values().collect()
}

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn expected_locale_path(
    root: &Path,
    config: &LinguiniConfig,
    namespace: &str,
    locale: &str,
) -> PathBuf {
    let mut path = root.join(&config.paths.locale);
    for part in namespace.split('.').filter(|part| !part.is_empty()) {
        path.push(part);
    }
    path.join(format!("{locale}.lgl"))
}

fn namespace_display(namespace: &str) -> String {
    if namespace.is_empty() {
        "<root>".to_owned()
    } else {
        namespace.to_owned()
    }
}

fn merge_module(target: &mut IrModule, source: IrModule) {
    target.enums.extend(source.enums);
    target.type_aliases.extend(source.type_aliases);
    target.messages.extend(source.messages);
    target.forms.extend(source.forms);
    target.functions.extend(source.functions);
}

fn merge_module_fallback(target: &mut IrModule, source: IrModule) {
    for item in source.enums {
        if !target.enums.iter().any(|existing| existing.name == item.name) {
            target.enums.push(item);
        }
    }
    for item in source.type_aliases {
        if !target
            .type_aliases
            .iter()
            .any(|existing| existing.name == item.name)
        {
            target.type_aliases.push(item);
        }
    }
    for item in source.messages {
        if !target.messages.iter().any(|existing| existing.name == item.name) {
            target.messages.push(item);
        }
    }
    for item in source.forms {
        if !target.forms.iter().any(|existing| existing.name == item.name) {
            target.forms.push(item);
        }
    }
    for item in source.functions {
        if !target
            .functions
            .iter()
            .any(|existing| existing.name == item.name)
        {
            target.functions.push(item);
        }
    }
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
schema = "schema"
locale = "locales"

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
    let diagnostics: Vec<_> = errors
        .into_iter()
        .map(|error| Diagnostic::error(error.message, error.span).with_note(note))
        .collect();

    render_file_diagnostics(root, path, source, &diagnostics)
}

fn render_file_diagnostics(
    root: &Path,
    path: &Path,
    source: &str,
    diagnostics: &[Diagnostic],
) -> String {
    let relative_path = path_for_output(root, path);
    render_diagnostics_with_color(
        &relative_path,
        source,
        diagnostics,
        std::io::stderr().is_terminal(),
    )
    .unwrap_or_else(|error| format!("failed to render diagnostics for {relative_path}: {error}"))
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
        assert!(subcommands.contains(&"fix".to_owned()));
        assert!(subcommands.contains(&"build".to_owned()));
        assert!(!subcommands.contains(&"cldr".to_owned()));
    }

    #[test]
    fn init_creates_valid_project() {
        let project = temp_project_dir("init_creates_valid_project");

        init_project(project.path()).expect("init project");

        assert!(project.path().join("linguini.toml").exists());
        assert!(project.path().join("schema").is_dir());
        assert!(project.path().join("locales").is_dir());
        let config = fs::read_to_string(project.path().join("linguini.toml")).expect("config");
        assert!(!config.contains("cache"));
        assert!(config.contains("[targets.ts]"));
        assert!(config.contains("out = \"src/generated/linguini\""));
    }

    #[test]
    fn check_lists_discovered_files() {
        let project = temp_project_dir("check_lists_discovered_files");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("schema/shop");
        let locale_dir = project.path().join("locales/shop/delivery");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::create_dir_all(&locale_dir).expect("locale dir");
        fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
        fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\n").expect("locale file");

        let output = check_project(project.path()).expect("check project");

        assert!(output.contains("schema/shop/delivery.lqs [shop.delivery]"));
        assert!(output.contains("locales/shop/delivery/en.lgl [en:shop.delivery]"));
    }

    #[test]
    fn check_reports_schema_syntax_diagnostics() {
        let project = temp_project_dir("check_reports_schema_syntax_diagnostics");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("schema/shop");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::write(schema_dir.join("broken.lqs"), "delivery(fruit: Fruit\n").expect("schema file");

        let error = check_project(project.path()).expect_err("check fails");
        let rendered = error.to_string();

        assert!(rendered.contains("Error:"));
        assert!(rendered.contains("schema/shop/broken.lqs"));
        assert!(rendered.contains("schema syntax error"));
    }

    #[test]
    fn check_reports_missing_schema_message_for_empty_locale_file() {
        let project = temp_project_dir("check_reports_missing_schema_message");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("schema/shop");
        let locale_dir = project.path().join("locales/shop/delivery");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::create_dir_all(&locale_dir).expect("locale dir");
        fs::write(schema_dir.join("delivery.lqs"), "delivery()\n").expect("schema file");
        fs::write(locale_dir.join("en.lgl"), "").expect("locale file");

        let error = check_project(project.path()).expect_err("check fails on missing message");
        let rendered = error.to_string();

        assert!(rendered.contains(
            "locale `en` for schema namespace `shop.delivery` is missing 1 schema message: `delivery`"
        ));
        assert!(rendered.contains("locales/shop/delivery/en.lgl"));
        assert!(rendered.contains("Fix: add missing locale message stubs"));
        assert!(!rendered.contains(",- ["));
        assert!(!rendered.contains("locale file contains no declarations"));
    }

    #[test]
    fn check_rejects_root_locale_file_for_schema_namespace() {
        let project = temp_project_dir("check_rejects_root_locale_file");
        init_project(project.path()).expect("init project");

        fs::write(project.path().join("schema/shop.lqs"), "delivery()\n").expect("schema file");
        fs::write(project.path().join("locales/en.lgl"), "delivery = Delivered\n")
            .expect("locale file");

        let error = check_project(project.path()).expect_err("check fails on misplaced locale");
        let rendered = error.to_string();

        assert!(
            rendered.contains("required locale file is missing for schema namespace `shop`: `en`")
        );
        assert!(rendered.contains("expected path: locales/shop/en.lgl"));
        assert!(rendered.contains("locale namespace `<root>` has no matching schema namespace"));
        assert!(!rendered.contains(",- ["));
    }


    #[test]
    fn check_warns_for_secondary_locale_missing_messages() {
        let project = temp_project_dir("check_warns_for_secondary_locale_missing_messages");
        init_project(project.path()).expect("init project");

        fs::write(
            project.path().join("linguini.toml"),
            r#"[project]
name = "shop"
default_locale = "en"
locales = ["en", "ru"]

[paths]
schema = "schema"
locale = "locales"

[targets.ts]
out = "src/generated/linguini"
module = "esm"
declaration = true
"#,
        )
        .expect("config");
        let schema_dir = project.path().join("schema");
        let locale_dir = project.path().join("locales/shop");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::create_dir_all(&locale_dir).expect("locale dir");
        fs::write(schema_dir.join("shop.lqs"), "delivery()\ncounted()\n").expect("schema file");
        fs::write(locale_dir.join("en.lgl"), "delivery = Delivered\ncounted = Counted\n")
            .expect("default locale file");
        fs::write(locale_dir.join("ru.lgl"), "delivery = Доставлено\n")
            .expect("secondary locale file");

        let output = check_project(project.path()).expect("secondary locale gaps are warnings");

        assert!(output.contains("Warning:"));
        assert!(output.contains(
            "locale `ru` for schema namespace `shop` is missing 1 schema message: `counted`"
        ));
        assert!(output.contains("Fix: add missing locale message stubs"));
        assert!(output.contains("linguini fix missing-messages:shop:ru"));
    }

    #[test]
    fn build_generates_typescript_project_files_without_cldr_cache() {
        let project = temp_project_dir("build_generates_typescript");
        init_project(project.path()).expect("init project");

        let schema_dir = project.path().join("schema/shop");
        let locale_dir = project.path().join("locales/shop/delivery");
        fs::create_dir_all(&schema_dir).expect("schema dir");
        fs::create_dir_all(&locale_dir).expect("locale dir");
        fs::write(schema_dir.join("delivery.lqs"), "delivery(count: Number)\n")
            .expect("schema file");
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
