use crate::FormatOptions;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum FormatItem {
    Text(String),
    Space,
    HardLine,
    Indent,
    Dedent,
    RawLineStart,
    ArmMarkerStart,
    ArmMarkerEnd,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub(crate) struct FormatIr {
    items: Vec<FormatItem>,
}

impl FormatIr {
    pub(crate) fn push(&mut self, item: FormatItem) {
        self.items.push(item);
    }

    pub(crate) fn text(&mut self, value: impl Into<String>) {
        let value = value.into();
        if !value.is_empty() {
            self.push(FormatItem::Text(value));
        }
    }

    pub(crate) fn render(&self, options: &FormatOptions) -> String {
        let mut out = String::new();
        let mut indent = 0usize;
        let mut at_line_start = true;
        let mut pending_space = false;
        let mut raw_line_start = false;

        for item in &self.items {
            match item {
                FormatItem::Text(text) => {
                    if raw_line_start && at_line_start {
                    } else {
                        push_indent(&mut out, indent, options, &mut at_line_start);
                    }
                    raw_line_start = false;
                    if pending_space && !out.ends_with([' ', '\n']) {
                        out.push(' ');
                    }
                    out.push_str(text);
                    pending_space = false;
                    at_line_start = false;
                }
                FormatItem::Space => {
                    pending_space = !at_line_start;
                }
                FormatItem::HardLine => {
                    push_newline(&mut out);
                    at_line_start = true;
                    pending_space = false;
                    raw_line_start = false;
                }
                FormatItem::Indent => indent += 1,
                FormatItem::Dedent => indent = indent.saturating_sub(1),
                FormatItem::RawLineStart => raw_line_start = true,
                FormatItem::ArmMarkerStart => out.push(ARROW_MARKER_START),
                FormatItem::ArmMarkerEnd => out.push(ARROW_MARKER_END),
            }
        }

        trim_trailing_blank_lines(&mut out);
        out = align_marked_match_arms(&out);
        out = enforce_line_width(&out, options);
        out.push('\n');
        out
    }
}

const ARROW_MARKER_START: char = '\u{E000}';
const ARROW_MARKER_END: char = '\u{E001}';

fn push_indent(out: &mut String, indent: usize, options: &FormatOptions, at_line_start: &mut bool) {
    if *at_line_start {
        out.push_str(&" ".repeat(indent * options.indent_width));
        *at_line_start = false;
    }
}

fn push_newline(out: &mut String) {
    while out.ends_with(' ') {
        out.pop();
    }
    if out.ends_with("\n\n") {
        return;
    }
    out.push('\n');
}

fn trim_trailing_blank_lines(out: &mut String) {
    while out.ends_with('\n') {
        out.pop();
    }
}

fn align_marked_match_arms(input: &str) -> String {
    let mut output = Vec::new();
    let mut group = Vec::new();

    for line in input.lines() {
        if line.contains(ARROW_MARKER_START) {
            group.push(line.to_owned());
        } else {
            flush_arm_group(&mut output, &mut group);
            output.push(remove_arrow_markers(line));
        }
    }

    flush_arm_group(&mut output, &mut group);
    output.join("\n")
}

fn flush_arm_group(output: &mut Vec<String>, group: &mut Vec<String>) {
    if group.is_empty() {
        return;
    }

    let max_before_width = group
        .iter()
        .filter_map(|line| {
            marked_arrow_bounds(line).map(|(start, _)| display_width(line[..start].trim_end()))
        })
        .max()
        .unwrap_or(0);

    output.extend(
        group
            .drain(..)
            .map(|line| align_marked_arm_line(&line, max_before_width)),
    );
}

fn align_marked_arm_line(line: &str, max_before_width: usize) -> String {
    let Some((start, end)) = marked_arrow_bounds(line) else {
        return remove_arrow_markers(line);
    };

    let before = line[..start].trim_end();
    let after = line[end..].trim_start();
    let padding = max_before_width.saturating_sub(display_width(before)) + 1;

    let mut aligned = String::new();
    aligned.push_str(before);
    aligned.push_str(&" ".repeat(padding));
    aligned.push_str("=>");
    if !after.is_empty() {
        aligned.push(' ');
        aligned.push_str(after);
    }
    aligned
}

fn marked_arrow_bounds(line: &str) -> Option<(usize, usize)> {
    let start = line.find(ARROW_MARKER_START)?;
    let arrow_start = start + ARROW_MARKER_START.len_utf8();
    let marker_end = line[arrow_start..].find(ARROW_MARKER_END)? + arrow_start;
    Some((start, marker_end + ARROW_MARKER_END.len_utf8()))
}

fn remove_arrow_markers(line: &str) -> String {
    line.replace([ARROW_MARKER_START, ARROW_MARKER_END], "")
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

fn enforce_line_width(input: &str, options: &FormatOptions) -> String {
    input
        .lines()
        .flat_map(|line| wrap_structural_arguments(line, options))
        .collect::<Vec<_>>()
        .join("\n")
}

fn wrap_structural_arguments(line: &str, options: &FormatOptions) -> Vec<String> {
    if options.max_line_width == 0 || display_width(line) <= options.max_line_width {
        return vec![line.to_owned()];
    }

    if line.contains('=') || line.contains("=>") || !line.contains(',') {
        return vec![line.to_owned()];
    }

    let Some(open) = line.find('(') else {
        return vec![line.to_owned()];
    };
    let Some(close) = line.rfind(')') else {
        return vec![line.to_owned()];
    };
    if close <= open {
        return vec![line.to_owned()];
    }

    let before = line[..open].trim_end();
    let args = line[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty())
        .collect::<Vec<_>>();
    if args.len() < 2 {
        return vec![line.to_owned()];
    }

    let suffix = line[close + 1..].trim_end();
    let base_indent = leading_spaces(line);
    let nested_indent = base_indent + options.indent_width;
    let mut wrapped = Vec::with_capacity(args.len() + 2);
    wrapped.push(format!("{before}("));
    for (index, arg) in args.iter().enumerate() {
        let comma = if index + 1 == args.len() { "" } else { "," };
        wrapped.push(format!("{}{}{}", " ".repeat(nested_indent), arg, comma));
    }
    wrapped.push(format!("{}){}", " ".repeat(base_indent), suffix));
    wrapped
}

fn leading_spaces(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}
