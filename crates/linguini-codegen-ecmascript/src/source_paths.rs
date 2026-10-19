//! Rebase source identity without reading or rewriting source content.
use crate::EcmaSource;
use linguini_codegen::lexical_relative_path;

/// Resolve original source paths relative to one emitted source map.
pub fn rebase_map_sources(
    output_root: &str,
    source_map_path: &str,
    sources: &[EcmaSource],
) -> Result<Vec<EcmaSource>, &'static str> {
    let map_directory = format!(
        "{output_root}/{}",
        source_map_path
            .rsplit_once('/')
            .map_or("", |(directory, _)| directory)
    );
    sources
        .iter()
        .map(|source| {
            lexical_relative_path(&map_directory, &source.path)
                .map(|path| EcmaSource::new(source.id, path, source.contents.clone()))
        })
        .collect()
}
