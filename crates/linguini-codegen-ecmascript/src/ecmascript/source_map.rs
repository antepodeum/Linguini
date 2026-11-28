//! Source-map encoding and streaming UTF-16 position resolution.

use super::literals::string_literal as json_string;
use super::EcmaSource;
use linguini_syntax::Span;
use std::collections::BTreeMap;

pub(super) fn render(
    file_name: &str,
    sources: &[EcmaSource],
    mappings: &[(usize, Span)],
) -> String {
    let positions = source_positions(sources, mappings);
    let last_generated_line = mappings
        .iter()
        .map(|(line, _)| *line)
        .max()
        .unwrap_or_default();
    let mut encoded = String::new();
    let mut mapping_index = 0;
    let mut previous_source = 0_i64;
    let mut previous_original_line = 0_i64;
    let mut previous_original_column = 0_i64;

    for generated_line in 0..=last_generated_line {
        if generated_line > 0 {
            encoded.push(';');
        }
        while let Some((line, _)) = mappings.get(mapping_index) {
            if *line != generated_line {
                break;
            }
            let Some(SourcePosition {
                source_index,
                line: original_line,
                column: original_column,
            }) = positions[mapping_index]
            else {
                mapping_index += 1;
                continue;
            };
            encode_vlq(0, &mut encoded);
            encode_vlq(source_index as i64 - previous_source, &mut encoded);
            encode_vlq(original_line as i64 - previous_original_line, &mut encoded);
            encode_vlq(
                original_column as i64 - previous_original_column,
                &mut encoded,
            );
            previous_source = source_index as i64;
            previous_original_line = original_line as i64;
            previous_original_column = original_column as i64;
            mapping_index += 1;
        }
    }

    let source_paths = sources
        .iter()
        .map(|source| json_string(&source.path))
        .collect::<Vec<_>>()
        .join(",");
    let source_contents = sources
        .iter()
        .map(|source| json_string(&source.contents))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"version\":3,\"file\":{},\"sources\":[{}],\"sourcesContent\":[{}],\"names\":[],\"mappings\":{}}}\n",
        json_string(file_name),
        source_paths,
        source_contents,
        json_string(&encoded)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourcePosition {
    source_index: usize,
    line: usize,
    column: usize,
}

/// Resolve offsets in source order, retaining the original order of generated mappings.
/// Source intervals never overlap, including mappings within very long single lines.
fn source_positions(
    sources: &[EcmaSource],
    mappings: &[(usize, Span)],
) -> Vec<Option<SourcePosition>> {
    let source_indexes = sources
        .iter()
        .enumerate()
        .map(|(index, source)| (source.id, index))
        .collect::<BTreeMap<_, _>>();
    let mut order = mappings
        .iter()
        .enumerate()
        .filter_map(|(mapping_index, (_, span))| {
            source_indexes
                .get(&span.source)
                .map(|&source_index| (source_index, span.start, mapping_index))
        })
        .collect::<Vec<_>>();
    order.sort_unstable_by_key(|&(source_index, offset, _)| (source_index, offset));
    let mut positions = vec![None; mappings.len()];
    for group in order.chunk_by(|left, right| left.0 == right.0) {
        let source_index = group[0].0;
        let text = &sources[source_index].contents;
        let mut consumed = 0;
        let mut line = 0;
        let mut column = 0;
        for &(_, offset, mapping_index) in group {
            if !text.is_char_boundary(offset) {
                continue;
            }
            let interval = &text[consumed..offset];
            if let Some(last_newline) = interval.rfind('\n') {
                line += interval.bytes().filter(|byte| *byte == b'\n').count();
                column = interval[last_newline + 1..].encode_utf16().count();
            } else {
                column += interval.encode_utf16().count();
            }
            consumed = offset;
            positions[mapping_index] = Some(SourcePosition {
                source_index,
                line,
                column,
            });
        }
    }
    positions
}

fn encode_vlq(value: i64, output: &mut String) {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = if value < 0 {
        ((-value) as u64) << 1 | 1
    } else {
        (value as u64) << 1
    };
    loop {
        let mut digit = (value & 31) as usize;
        value >>= 5;
        if value != 0 {
            digit |= 32;
        }
        output.push(BASE64[digit] as char);
        if value == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests;
