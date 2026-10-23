use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::{CliError, CliResult};

pub(super) fn highlight_file(root: &Path, path: &Path) -> CliResult<String> {
    let path = root.join(path);
    let language = match path.extension().and_then(|extension| extension.to_str()) {
        Some("lgs") => "linguini-schema",
        Some("lgl") => "linguini-locale",
        _ => {
            return Err(CliError::Diagnostics(
                "highlight requires a .lgs or .lgl file".to_owned(),
            ))
        }
    };
    let mut source = String::new();
    File::open(&path)
        .and_then(|file| file.take(4 * 1024 * 1024 + 1).read_to_string(&mut source))
        .map_err(|source| CliError::Io {
            path: path.clone(),
            source,
        })?;
    let output = linguini_lsp::highlight_source(language, &source)
        .ok_or_else(|| CliError::Diagnostics("highlight safety limit exceeded".to_owned()))?;
    Ok(format!("{output}\n"))
}
