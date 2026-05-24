use crate::{CliError, CliResult, FixArgs};
use linguini_analyzer::{locale_public_messages, schema_public_messages};
use linguini_config::LinguiniConfig;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::io::{create_dir_all, path_for_output, read_file, read_project_config, write_file};
use super::sources::{
    expected_locale_path, load_locale_sources, load_schema_sources, locale_index,
};
use super::util::pluralize;

#[derive(Debug, Clone)]
struct ProjectFix {
    id: String,
    title: String,
    operation: FixOperation,
}

#[derive(Debug, Clone)]
enum FixOperation {
    CreateFile { path: PathBuf, contents: String },
    AppendFile { path: PathBuf, contents: String },
}

impl ProjectFix {
    fn kind(&self) -> &str {
        self.id
            .split_once(':')
            .map_or(self.id.as_str(), |(kind, _)| kind)
    }

    fn path(&self) -> &Path {
        match &self.operation {
            FixOperation::CreateFile { path, .. } | FixOperation::AppendFile { path, .. } => path,
        }
    }
}

pub(crate) fn fix_project(root: &Path, args: &FixArgs) -> CliResult<String> {
    let config = read_project_config(root)?;
    let fixes = available_project_fixes(root, &config)?;

    if !args.all && args.ids.is_empty() && args.types.is_empty() && args.file.is_none() {
        return Ok(render_fix_list(&fixes));
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

fn render_fix_list(fixes: &[ProjectFix]) -> String {
    let mut output = String::new();
    if fixes.is_empty() {
        output.push_str("no automatic fixes available\n");
        return output;
    }

    output.push_str("available fixes:\n");
    for fix in fixes {
        output.push_str(&format!("- {}: {}\n", fix.id, fix.title));
    }
    output.push_str("run `linguini fix --all`, `linguini fix --type <type>`, `linguini fix --file <path> --all`, or `linguini fix <id>...`\n");
    output
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
    let candidates = fixes
        .iter()
        .filter(|fix| {
            args.file
                .as_ref()
                .is_none_or(|file| fix_matches_file(fix, file, fix.path()))
        })
        .collect::<Vec<_>>();

    if args.all && args.types.is_empty() && args.ids.is_empty() {
        return Ok(candidates.into_iter().cloned().collect());
    }

    let by_id = candidates
        .iter()
        .map(|fix| (fix.id.as_str(), *fix))
        .collect::<BTreeMap<_, _>>();
    let mut selected = Vec::new();
    let mut missing = Vec::new();

    for id in &args.ids {
        match by_id.get(id.as_str()) {
            Some(fix) => selected.push((*fix).clone()),
            None => missing.push(id.clone()),
        }
    }

    for kind in &args.types {
        let matches = candidates
            .iter()
            .filter(|fix| fix.kind() == kind)
            .copied()
            .collect::<Vec<_>>();
        if matches.is_empty() {
            missing.push(format!("--type {kind}"));
            continue;
        }
        selected.extend(matches.into_iter().cloned());
    }

    if !missing.is_empty() {
        return Err(CliError::Diagnostics(format!(
            "unknown fix {}: {}\nrun `linguini fix` to list available fixes\n",
            pluralize(missing.len(), "id", "ids"),
            missing.join(", ")
        )));
    }

    if selected.is_empty() && args.file.is_some() {
        selected.extend(candidates.into_iter().cloned());
    }

    selected.sort_by(|left, right| left.id.cmp(&right.id));
    selected.dedup_by(|left, right| left.id == right.id);
    Ok(selected)
}

fn fix_matches_file(fix: &ProjectFix, file: &Path, path: &Path) -> bool {
    let root = path.ancestors().last().unwrap_or_else(|| Path::new(""));
    path == file
        || path.ends_with(file)
        || file
            .strip_prefix(root)
            .is_ok_and(|relative| path.ends_with(relative))
        || fix.title.contains(&file.display().to_string())
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

pub(crate) fn missing_locale_fix_id(namespace: &str, locale: &str) -> String {
    format!("missing-locale:{}:{}", fix_id_namespace(namespace), locale)
}

pub(crate) fn missing_messages_fix_id(namespace: &str, locale: &str) -> String {
    format!(
        "missing-messages:{}:{}",
        fix_id_namespace(namespace),
        locale
    )
}

fn fix_id_namespace(namespace: &str) -> String {
    if namespace.is_empty() {
        "root".to_owned()
    } else {
        namespace.to_owned()
    }
}
