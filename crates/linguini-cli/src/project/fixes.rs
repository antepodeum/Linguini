use crate::{CliError, CliResult, FixArgs};
use linguini_analyzer::{locale_public_messages, schema_public_messages};
use linguini_config::LinguiniConfig;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::io::{
    atomic_write_file_if_unchanged, create_new_file, path_for_output, read_project_config,
};
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
    CreateFile {
        path: PathBuf,
        contents: String,
    },
    AppendFile {
        path: PathBuf,
        expected: String,
        contents: String,
    },
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

    let selected = select_fixes(root, &fixes, args)?;
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
    let locale_index = locale_index(&locale_files)?;
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
                            expected: locale_file.source.clone(),
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

fn select_fixes(root: &Path, fixes: &[ProjectFix], args: &FixArgs) -> CliResult<Vec<ProjectFix>> {
    let requested_file = args
        .file
        .as_ref()
        .map(|file| normalized_project_file(root, file))
        .transpose()?;
    let candidates = fixes
        .iter()
        .filter(|fix| {
            requested_file
                .as_ref()
                .map_or(true, |file| fix.path() == file)
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
    reject_conflicting_fixes(&selected)?;
    Ok(selected)
}

fn normalized_project_file(root: &Path, file: &Path) -> CliResult<PathBuf> {
    if file.is_absolute()
        || file.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(CliError::Diagnostics(format!(
            "fix file filter must be project-relative: `{}`\n",
            file.display()
        )));
    }
    Ok(root.join(file))
}

fn reject_conflicting_fixes(fixes: &[ProjectFix]) -> CliResult<()> {
    let mut paths = BTreeMap::new();
    for fix in fixes {
        if let Some(previous) = paths.insert(fix.path(), fix.id.as_str()) {
            return Err(CliError::Diagnostics(format!(
                "fixes `{previous}` and `{}` both modify `{}`; apply one and rerun analysis\n",
                fix.id,
                fix.path().display()
            )));
        }
    }
    Ok(())
}

fn apply_project_fix(root: &Path, fix: &ProjectFix, output: &mut String) -> CliResult<()> {
    match &fix.operation {
        FixOperation::CreateFile { path, contents } => {
            create_new_file(path, contents)?;
            output.push_str(&format!(
                "applied {}: created {}\n",
                fix.id,
                path_for_output(root, path)
            ));
        }
        FixOperation::AppendFile {
            path,
            expected,
            contents,
        } => {
            let mut updated = expected.clone();
            updated.push_str(contents);
            atomic_write_file_if_unchanged(path, expected, &updated)?;
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
    let mut root = StubGroup::default();
    for name in names {
        root.insert(name);
    }
    let mut output = String::new();
    root.render(0, &mut output);
    output
}

#[derive(Default)]
struct StubGroup {
    messages: BTreeSet<String>,
    groups: BTreeMap<String, StubGroup>,
}

impl StubGroup {
    fn insert(&mut self, qualified_name: &str) {
        let mut segments = qualified_name.split('.').peekable();
        let mut group = self;
        while let Some(segment) = segments.next() {
            if segments.peek().is_none() {
                group.messages.insert(segment.to_owned());
            } else {
                group = group.groups.entry(segment.to_owned()).or_default();
            }
        }
    }

    fn render(&self, indent: usize, output: &mut String) {
        let padding = " ".repeat(indent);
        for message in &self.messages {
            output.push_str(&format!("{padding}{message} = TODO\n"));
        }
        for (name, group) in &self.groups {
            output.push_str(&format!("{padding}{name} {{\n"));
            group.render(indent + 2, output);
            output.push_str(&format!("{padding}}}\n"));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{apply_project_fix, locale_stub_text, select_fixes, FixOperation, ProjectFix};
    use crate::FixArgs;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn append_fix_refuses_concurrently_changed_source() {
        let root = TempDir::new().expect("project");
        let path = root.path().join("locale.lgl");
        fs::write(&path, "message = Original\n").expect("source");
        let fix = ProjectFix {
            id: "missing-messages:root:en".to_owned(),
            title: "append stub".to_owned(),
            operation: FixOperation::AppendFile {
                path: path.clone(),
                expected: "message = Original\n".to_owned(),
                contents: "\nmissing = TODO\n".to_owned(),
            },
        };
        fs::write(&path, "message = Edited\n").expect("concurrent edit");

        let error = apply_project_fix(root.path(), &fix, &mut String::new())
            .expect_err("stale fix must fail");

        assert!(error.to_string().contains("concurrently changed"));
        assert_eq!(
            fs::read_to_string(path).expect("source"),
            "message = Edited\n"
        );
    }

    #[test]
    fn create_fix_never_reports_or_overwrites_existing_file() {
        let root = TempDir::new().expect("project");
        let path = root.path().join("locale.lgl");
        fs::write(&path, "keep\n").expect("source");
        let fix = ProjectFix {
            id: "missing-locale:root:en".to_owned(),
            title: "create locale".to_owned(),
            operation: FixOperation::CreateFile {
                path: path.clone(),
                contents: "replacement\n".to_owned(),
            },
        };
        let mut output = String::new();

        assert!(apply_project_fix(root.path(), &fix, &mut output).is_err());
        assert!(output.is_empty());
        assert_eq!(fs::read_to_string(path).expect("source"), "keep\n");
    }

    #[test]
    fn file_filter_matches_only_exact_project_relative_path() {
        let root = TempDir::new().expect("project");
        let exact = ProjectFix {
            id: "missing-locale:shop:en".to_owned(),
            title: "exact".to_owned(),
            operation: FixOperation::CreateFile {
                path: root.path().join("locales/shop/en.lgl"),
                contents: String::new(),
            },
        };
        let suffix = ProjectFix {
            id: "missing-locale:other-shop:en".to_owned(),
            title: "suffix".to_owned(),
            operation: FixOperation::CreateFile {
                path: root.path().join("locales/other/shop/en.lgl"),
                contents: String::new(),
            },
        };
        let args = FixArgs {
            all: true,
            types: Vec::new(),
            file: Some(PathBuf::from("locales/shop/en.lgl")),
            ids: Vec::new(),
        };

        let selected =
            select_fixes(root.path(), &[exact, suffix], &args).expect("select exact file");

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "missing-locale:shop:en");
    }

    #[test]
    fn nested_stub_text_preserves_every_group_segment() {
        let output = locale_stub_text(&[
            "shop.main.title".to_owned(),
            "shop.cart.items".to_owned(),
            "plain".to_owned(),
        ]);

        assert_eq!(
            output,
            "plain = TODO\nshop {\n  cart {\n    items = TODO\n  }\n  main {\n    title = TODO\n  }\n}\n"
        );
    }
}
