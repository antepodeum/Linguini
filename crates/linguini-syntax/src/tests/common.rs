use std::{fs, path::Path};

use crate::Token;

pub(super) fn render_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|token| {
            format!(
                "{:?} @ {}..{}\n",
                token.kind, token.span.start, token.span.end
            )
        })
        .collect()
}

pub(super) fn assert_snapshot(path: &str, snapshot: &str) {
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        fs::write(repo_root().join(path), snapshot).expect("write snapshot");
    }

    let expected = fs::read_to_string(repo_root().join(path)).expect("read snapshot");
    assert_eq!(snapshot, expected);
}

#[rustfmt::skip]
fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("repo root")
}
