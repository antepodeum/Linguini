use crate::{FormatError, FormatOptions};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum FormatItem {
    Text(String),
    Verbatim(String),
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
        self.push_text(value.into(), false);
    }

    pub(crate) fn verbatim(&mut self, value: impl Into<String>) {
        self.push_text(value.into(), true);
    }

    fn push_text(&mut self, value: String, verbatim: bool) {
        if value.is_empty() {
            return;
        }
        self.push(if verbatim {
            FormatItem::Verbatim(value)
        } else {
            FormatItem::Text(value)
        });
    }

    pub(crate) fn render(
        &self,
        options: &FormatOptions,
        newline: &str,
    ) -> Result<String, FormatError> {
        let mut document = RenderedDocument::new(newline);
        let mut indent = 0usize;
        let mut pending_space = false;
        let mut raw_line_start = false;
        let mut mark_next_text_as_arm = false;

        for item in &self.items {
            match item {
                FormatItem::Text(text) | FormatItem::Verbatim(text) => {
                    let verbatim = matches!(item, FormatItem::Verbatim(_));
                    document.push_text(
                        text,
                        verbatim,
                        indent,
                        options,
                        raw_line_start,
                        pending_space,
                        mark_next_text_as_arm,
                    )?;
                    pending_space = false;
                    raw_line_start = false;
                    mark_next_text_as_arm = false;
                }
                FormatItem::Space => {
                    pending_space = !document.at_line_start();
                }
                FormatItem::HardLine => {
                    document.hard_line();
                    pending_space = false;
                    raw_line_start = false;
                    mark_next_text_as_arm = false;
                }
                FormatItem::Indent => {
                    indent = indent.checked_add(1).ok_or_else(|| {
                        FormatError::InvalidOptions("nesting depth overflowed usize".to_owned())
                    })?;
                }
                FormatItem::Dedent => indent = indent.saturating_sub(1),
                FormatItem::RawLineStart => raw_line_start = true,
                FormatItem::ArmMarkerStart => mark_next_text_as_arm = true,
                FormatItem::ArmMarkerEnd => {}
            }
        }

        document.finish();
        document.align_match_arms()?;
        document.enforce_line_width(options)?;
        Ok(document.into_string())
    }
}

#[derive(Debug, Default)]
struct RenderedLine {
    text: String,
    ending: Option<String>,
    contains_verbatim: bool,
    arm_offset: Option<usize>,
}

#[derive(Debug)]
struct RenderedDocument {
    lines: Vec<RenderedLine>,
    newline: String,
}

#[derive(Debug, Clone, Copy)]
struct PushText {
    verbatim: bool,
    raw_line_start: bool,
    pending_space: bool,
    arm_marker: bool,
}

impl RenderedDocument {
    fn new(newline: &str) -> Self {
        Self {
            lines: vec![RenderedLine::default()],
            newline: newline.to_owned(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push_text(
        &mut self,
        text: &str,
        verbatim: bool,
        indent: usize,
        options: &FormatOptions,
        raw_line_start: bool,
        pending_space: bool,
        arm_marker: bool,
    ) -> Result<(), FormatError> {
        self.prepare_text(
            indent,
            options,
            PushText {
                verbatim,
                raw_line_start,
                pending_space,
                arm_marker,
            },
        )?;
        self.append_text(text, verbatim);
        Ok(())
    }

    fn prepare_text(
        &mut self,
        indent: usize,
        options: &FormatOptions,
        push: PushText,
    ) -> Result<(), FormatError> {
        if !(push.raw_line_start && self.at_line_start()) {
            self.push_indent(indent, options)?;
        }

        let line = self.current_mut();
        if push.pending_space && !line.text.ends_with(' ') && !line.text.is_empty() {
            line.text.push(' ');
        }
        if push.arm_marker {
            line.arm_offset = Some(line.text.len());
        }
        if push.verbatim {
            line.contains_verbatim = true;
        }
        Ok(())
    }

    fn push_indent(&mut self, indent: usize, options: &FormatOptions) -> Result<(), FormatError> {
        if self.at_line_start() {
            let width = indent.checked_mul(options.indent_width).ok_or_else(|| {
                FormatError::InvalidOptions("indentation width overflowed usize".to_owned())
            })?;
            self.current_mut().text.push_str(&" ".repeat(width));
        }
        Ok(())
    }

    fn append_text(&mut self, mut text: &str, verbatim: bool) {
        while let Some((start, end)) = newline_bounds(text) {
            let line = self.current_mut();
            line.text.push_str(&text[..start]);
            line.contains_verbatim |= verbatim;
            line.ending = Some(text[start..end].to_owned());
            self.lines.push(RenderedLine::default());
            text = &text[end..];
        }

        if !text.is_empty() {
            let line = self.current_mut();
            line.text.push_str(text);
            line.contains_verbatim |= verbatim;
        }
    }

    fn hard_line(&mut self) {
        if !self.current().contains_verbatim {
            while self.current().text.ends_with(' ') {
                self.current_mut().text.pop();
            }
        }

        if self.current().text.is_empty()
            && self.lines.len() >= 2
            && self.lines[self.lines.len() - 2].text.is_empty()
        {
            return;
        }

        if self.current().ending.is_none() {
            self.current_mut().ending = Some(self.newline.clone());
            self.lines.push(RenderedLine::default());
        }
    }

    fn finish(&mut self) {
        while self.lines.len() > 1 && self.lines.last().is_some_and(|line| line.text.is_empty()) {
            self.lines.pop();
        }

        if self.lines.is_empty() {
            self.lines.push(RenderedLine::default());
        }
        self.current_mut().ending = Some(self.newline.clone());
    }

    fn align_match_arms(&mut self) -> Result<(), FormatError> {
        let mut start = 0usize;
        while start < self.lines.len() {
            if self.lines[start].arm_offset.is_none() {
                start += 1;
                continue;
            }

            let mut end = start + 1;
            while end < self.lines.len() && self.lines[end].arm_offset.is_some() {
                end += 1;
            }
            align_arm_group(&mut self.lines[start..end])?;
            start = end;
        }
        Ok(())
    }

    fn enforce_line_width(&mut self, options: &FormatOptions) -> Result<(), FormatError> {
        if options.max_line_width == 0 {
            return Ok(());
        }

        let mut output = Vec::with_capacity(self.lines.len());
        for mut line in self.lines.drain(..) {
            if line.contains_verbatim || display_width(&line.text) <= options.max_line_width {
                output.push(line);
                continue;
            }

            let Some(wrapped) = wrap_structural_arguments(&line.text, options)? else {
                output.push(line);
                continue;
            };
            let last = wrapped.len().saturating_sub(1);
            for (index, text) in wrapped.into_iter().enumerate() {
                output.push(RenderedLine {
                    text,
                    ending: if index == last {
                        line.ending.take()
                    } else {
                        Some(self.newline.clone())
                    },
                    contains_verbatim: false,
                    arm_offset: None,
                });
            }
        }
        self.lines = output;
        Ok(())
    }

    fn into_string(self) -> String {
        let capacity = self
            .lines
            .iter()
            .map(|line| line.text.len() + line.ending.as_ref().map_or(0, std::string::String::len))
            .sum();
        let mut output = String::with_capacity(capacity);
        for line in self.lines {
            output.push_str(&line.text);
            if let Some(ending) = line.ending {
                output.push_str(&ending);
            }
        }
        output
    }

    fn at_line_start(&self) -> bool {
        self.current().text.is_empty()
    }

    fn current(&self) -> &RenderedLine {
        self.lines
            .last()
            .expect("rendered document always has a current line")
    }

    fn current_mut(&mut self) -> &mut RenderedLine {
        self.lines
            .last_mut()
            .expect("rendered document always has a current line")
    }
}

fn newline_bounds(text: &str) -> Option<(usize, usize)> {
    text.char_indices()
        .find_map(|(index, character)| match character {
            '\r' if text.as_bytes().get(index + 1) == Some(&b'\n') => Some((index, index + 2)),
            '\r' | '\n' => Some((index, index + 1)),
            _ => None,
        })
}

fn align_arm_group(lines: &mut [RenderedLine]) -> Result<(), FormatError> {
    let max_before_width = lines
        .iter()
        .map(arm_prefix)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(display_width)
        .max()
        .unwrap_or(0);

    for line in lines {
        let offset = line
            .arm_offset
            .ok_or_else(|| FormatError::SyntaxMismatch("missing match-arm marker".to_owned()))?;
        let before = line
            .text
            .get(..offset)
            .ok_or_else(|| FormatError::SyntaxMismatch("invalid match-arm marker".to_owned()))?
            .trim_end();
        let after = line
            .text
            .get(offset..)
            .ok_or_else(|| FormatError::SyntaxMismatch("invalid match-arm marker".to_owned()))?;
        if !after.starts_with("=>") {
            return Err(FormatError::SyntaxMismatch(
                "match-arm marker does not point to `=>`".to_owned(),
            ));
        }

        let padding = max_before_width
            .saturating_sub(display_width(before))
            .saturating_add(1);
        let mut aligned = String::with_capacity(before.len() + padding + after.len());
        aligned.push_str(before);
        aligned.push_str(&" ".repeat(padding));
        let new_offset = aligned.len();
        aligned.push_str(after);
        line.text = aligned;
        line.arm_offset = Some(new_offset);
    }
    Ok(())
}

fn arm_prefix(line: &RenderedLine) -> Result<&str, FormatError> {
    let offset = line
        .arm_offset
        .ok_or_else(|| FormatError::SyntaxMismatch("missing match-arm marker".to_owned()))?;
    line.text
        .get(..offset)
        .map(str::trim_end)
        .ok_or_else(|| FormatError::SyntaxMismatch("invalid match-arm marker".to_owned()))
}

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn wrap_structural_arguments(
    line: &str,
    options: &FormatOptions,
) -> Result<Option<Vec<String>>, FormatError> {
    let Some((open, close)) = outer_parentheses(line) else {
        return Ok(None);
    };
    let Some(arguments) = split_arguments(&line[open + 1..close]) else {
        return Ok(None);
    };
    if arguments.len() < 2 {
        return Ok(None);
    }

    let before = line[..open].trim_end();
    let suffix = line[close + 1..].trim_end();
    let base_indent = leading_spaces(line);
    let nested_indent = base_indent
        .checked_add(options.indent_width)
        .ok_or_else(|| FormatError::InvalidOptions("nested indentation overflowed usize".into()))?;
    let mut wrapped = Vec::with_capacity(arguments.len() + 2);
    wrapped.push(format!("{before}("));
    for (index, argument) in arguments.iter().enumerate() {
        let comma = if index + 1 == arguments.len() {
            ""
        } else {
            ","
        };
        wrapped.push(format!(
            "{}{}{}",
            " ".repeat(nested_indent),
            argument,
            comma
        ));
    }
    wrapped.push(format!("{}){}", " ".repeat(base_indent), suffix));
    Ok(Some(wrapped))
}

fn outer_parentheses(line: &str) -> Option<(usize, usize)> {
    let mut open = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in line.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '(' => {
                if open.is_none() {
                    open = Some(index);
                }
                depth = depth.saturating_add(1);
            }
            ')' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    return open.map(|open| (open, index));
                }
            }
            _ => {}
        }
    }
    None
}

fn split_arguments(arguments: &str) -> Option<Vec<&str>> {
    let mut output = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in arguments.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '(' => depth = depth.checked_add(1)?,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                let argument = arguments[start..index].trim();
                if !argument.is_empty() {
                    output.push(argument);
                }
                start = index + 1;
            }
            _ => {}
        }
    }

    if in_string || depth != 0 {
        return None;
    }
    let last = arguments[start..].trim();
    if !last.is_empty() {
        output.push(last);
    }
    Some(output)
}

fn leading_spaces(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}
