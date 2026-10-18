//! JS and TS share canonical templates; paired runtime implementations remain forbidden.

use std::path::{Path, PathBuf};

fn source_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("read codegen source directory") {
        let path = entry.expect("read codegen source entry").path();
        if path.is_dir() {
            source_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

#[test]
fn production_runtime_ownership_has_no_language_templates_or_retired_emitters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    source_files(&root, &mut files);
    files.sort();
    for path in files {
        assert!(
            !matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("ts" | "js")
            ),
            "runtime syntax belongs in shared target templates outside Rust source: {}",
            path.display()
        );
        let source = std::fs::read_to_string(&path).expect("read codegen source");
        for retired in [
            "fn javascript_shared_runtime(",
            "fn typescript_index_runtime(",
            "fn javascript_index_runtime(",
            "fn typescript_index(",
            "fn javascript_index(",
            "fn typescript_locale(",
            "fn javascript_locale(",
            "SHARED_RUNTIME",
            "SINGLE_INDEX_RUNTIME",
            "include_str!(\"templates/",
        ] {
            assert!(
                !source.contains(retired),
                "retired assembly `{retired}` reintroduced in {}",
                path.display()
            );
        }
    }
}
