use linguini_config::parse_config;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct TomlFence {
    path: PathBuf,
    line: usize,
    fragment: Option<String>,
    source: String,
}

#[test]
fn documented_toml_is_registered_and_valid() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fences = documentation_files(&root)
        .into_iter()
        .flat_map(|path| toml_fences(&root, &path))
        .collect::<Vec<_>>();
    let standalone = fences
        .iter()
        .filter(|fence| fence.fragment.is_none())
        .collect::<Vec<_>>();
    let fragments = fences
        .iter()
        .filter(|fence| fence.fragment.is_some())
        .collect::<Vec<_>>();

    assert!(!standalone.is_empty(), "no standalone TOML documentation");
    for fence in standalone {
        parse_config(&fence.source).unwrap_or_else(|errors| {
            panic!(
                "{}:{} is not a valid Linguini config: {errors:#?}",
                fence.path.display(),
                fence.line
            )
        });
    }
    for fence in fragments {
        fence.source.parse::<toml::Value>().unwrap_or_else(|error| {
            panic!(
                "{}:{} is not valid TOML fragment `{}`: {error}",
                fence.path.display(),
                fence.line,
                fence.fragment.as_deref().expect("fragment identity")
            )
        });
    }

    assert_eq!(fences.len(), 11, "new TOML fences must be registered");
    assert_eq!(
        fences
            .iter()
            .filter(|fence| fence.fragment.is_none())
            .count(),
        5
    );
    assert_eq!(
        fences
            .iter()
            .filter(|fence| fence.fragment.is_some())
            .count(),
        6
    );
}

fn documentation_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![root.join("README.md")];
    collect_markdown(&root.join("docs"), &mut files);
    files.sort();
    files
}

fn collect_markdown(directory: &Path, files: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
        .map(|entry| entry.expect("documentation directory entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_markdown(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
}

fn toml_fences(root: &Path, path: &Path) -> Vec<TomlFence> {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let mut fences = Vec::new();
    let mut open: Option<(usize, Option<String>, Vec<&str>)> = None;

    for (index, line) in source.lines().enumerate() {
        if let Some((start, fragment, body)) = &mut open {
            if line == "```" {
                assert!(
                    !body.is_empty(),
                    "{}:{start}: empty TOML fence",
                    path.display()
                );
                fences.push(TomlFence {
                    path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
                    line: *start,
                    fragment: fragment.clone(),
                    source: format!("{}\n", body.join("\n")),
                });
                open = None;
            } else {
                body.push(line);
            }
            continue;
        }

        if let Some(metadata) = line.strip_prefix("```toml") {
            let metadata = metadata.trim();
            let fragment = if metadata.is_empty() {
                None
            } else {
                Some(
                    metadata
                        .strip_prefix("fragment=")
                        .filter(|value| valid_fragment_name(value))
                        .unwrap_or_else(|| {
                            panic!(
                                "{}:{}: unsupported TOML fence metadata `{metadata}`",
                                path.display(),
                                index + 1
                            )
                        })
                        .to_owned(),
                )
            };
            open = Some((index + 1, fragment, Vec::new()));
        }
    }

    assert!(
        open.is_none(),
        "{} contains an unclosed TOML fence",
        path.display()
    );
    fences
}

fn valid_fragment_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_lowercase()))
}
