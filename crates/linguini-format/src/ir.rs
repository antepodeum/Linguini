use crate::FormatError;
use crate::FormatOptions;
use unicode_width::UnicodeWidthStr;

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

    pub(crate) fn render(&self, options: &FormatOptions) -> Result<String, FormatError> {
        let mut out = String::new();
        let mut arm_offsets = Vec::new();
        let mut indent = 0usize;
        let mut at_line_start = true;
        let mut pending_space = false;
        let mut raw_line_start = false;

        for item in &self.items {
            match item {
                FormatItem::Text(text) => {
                    if raw_line_start && at_line_start {
                    } else {
                        push_indent(&mut out, indent, options, &mut at_line_start)?;
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
                FormatItem::ArmMarkerStart => arm_offsets.push(out.len()),
                FormatItem::ArmMarkerEnd => {}
            }
        }

        trim_trailing_blank_lines(&mut out);
        out = align_marked_match_arms(&out, &arm_offsets);
        out = enforce_line_width(&out, options);
        out.push('\n');
        Ok(out)
    }
}

fn push_indent(
    out: &mut String,
    indent: usize,
    options: &FormatOptions,
    at_line_start: &mut bool,
) -> Result<(), FormatError> {
    if *at_line_start {
        let width = indent.checked_mul(options.indent_width).ok_or_else(|| {
            FormatError::InvalidOptions("indentation width overflowed usize".to_owned())
        })?;
        out.push_str(&" ".repeat(width));
        *at_line_start = false;
    }
    Ok(())
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

fn align_marked_match_arms(input: &str, arm_offsets: &[usize]) -> String {
    let mut output = Vec::new();
    let mut group = Vec::new();
    let mut line_start = 0usize;
    let mut offsets = arm_offsets.iter().copied().peekable();

    for line in input.lines() {
        let line_end = line_start + line.len();
        while offsets.peek().is_some_and(|offset| *offset < line_start) {
            offsets.next();
        }
        let marked = offsets
            .peek()
            .is_some_and(|offset| *offset >= line_start && *offset <= line_end);
        if marked {
            group.push(line.to_owned());
            offsets.next();
        } else {
            flush_arm_group(&mut output, &mut group);
            output.push(line.to_owned());
        }
        line_start = line_end.saturating_add(1);
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
            line.find("=>")
                .map(|start| display_width(line[..start].trim_end()))
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
    let Some(start) = line.find("=>") else {
        return line.to_owned();
    };

    let before = line[..start].trim_end();
    let after = line[start + 2..].trim_start();
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

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
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
