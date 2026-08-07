use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DedentError {
    InvalidUtf8,
}

impl fmt::Display for DedentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DedentError::InvalidUtf8 => write!(f, "clipboard text is not valid UTF-8"),
        }
    }
}

impl Error for DedentError {}

enum PromptMode {
    Unwrap,
    PreserveLines,
}

pub fn dedent_text(input: &str) -> String {
    let (stripped_prompt, mode) = strip_prompt_prefix(input);
    let input = stripped_prompt.as_deref().unwrap_or(input);

    let min_indent = input
        .lines()
        .filter(|line| !is_blank_line(line))
        .map(count_prefix_whitespace)
        .min()
        .unwrap_or(0);

    let dedented: String = input
        .split_inclusive('\n')
        .map(|line| dedent_line(line, min_indent))
        .collect();

    if matches!(mode, Some(PromptMode::Unwrap)) {
        unwrap_prompt_lines(&dedented)
    } else {
        dedented
    }
}

pub fn text_from_bytes(bytes: Vec<u8>) -> Result<String, DedentError> {
    String::from_utf8(bytes).map_err(|_| DedentError::InvalidUtf8)
}

fn dedent_line(line: &str, width: usize) -> String {
    let (body, newline) = split_trailing_newline(line);
    let body = remove_prefix_whitespace(body, width).trim_end_matches([' ', '\t']);

    format!("{body}{newline}")
}

fn strip_prompt_prefix(input: &str) -> (Option<String>, Option<PromptMode>) {
    let mut lines = input.split_inclusive('\n');
    let first_line = match lines.next() {
        Some(line) => line,
        None => return (None, None),
    };
    let (first_body, first_newline) = split_trailing_newline(first_line);
    let (indent, _) = match prompt_indent(first_body) {
        Some(value) => value,
        None => return (None, None),
    };

    let mut stripped = String::new();
    let (first_body, mode) = match strip_prompt_marker(first_body, indent) {
        Some(value) => value,
        None => return (None, None),
    };
    stripped.push_str(first_body);
    stripped.push_str(first_newline);

    for line in lines {
        let (body, newline) = split_trailing_newline(line);
        if is_blank_line(body) {
            stripped.push_str(body.trim_matches([' ', '\t']));
        } else {
            let Some(body) = body.strip_prefix(indent) else {
                return (None, None);
            };
            let Some(body) = body.strip_prefix("  ") else {
                return (None, None);
            };
            stripped.push_str(body);
        }
        stripped.push_str(newline);
    }

    (Some(stripped), Some(mode))
}

fn prompt_indent(line: &str) -> Option<(&str, PromptMode)> {
    for (idx, ch) in line.char_indices() {
        if ch == ' ' || ch == '\t' {
            continue;
        }

        if line[idx..].starts_with("❯ ") || line[idx..].starts_with("› ") {
            return Some((&line[..idx], PromptMode::Unwrap));
        }

        if line[idx..].starts_with("> ") {
            return Some((&line[..idx], PromptMode::PreserveLines));
        }
    }

    None
}

fn strip_prompt_marker<'a>(line: &'a str, indent: &str) -> Option<(&'a str, PromptMode)> {
    let line = line.strip_prefix(indent)?;

    if let Some(rest) = line.strip_prefix("❯ ") {
        Some((rest, PromptMode::Unwrap))
    } else if let Some(rest) = line.strip_prefix("› ") {
        Some((rest, PromptMode::Unwrap))
    } else {
        line.strip_prefix("> ")
            .map(|rest| (rest, PromptMode::PreserveLines))
    }
}

fn unwrap_prompt_lines(input: &str) -> String {
    let lines: Vec<_> = input
        .split_inclusive('\n')
        .map(split_trailing_newline)
        .collect();
    let mut output = String::with_capacity(input.len());
    let mut previous_line_was_joined = false;

    for (index, (body, newline)) in lines.iter().enumerate() {
        let body = if previous_line_was_joined {
            body.trim_start_matches([' ', '\t'])
        } else {
            body
        };

        output.push_str(body);

        let next_body = lines.get(index + 1).map(|(body, _)| *body);
        let should_join = !newline.is_empty()
            && !is_blank_line(body)
            && next_body.is_some_and(|next| !is_blank_line(next));

        if should_join {
            let next_body = next_body.expect("next body exists when joining prompt lines");
            output.push_str(join_separator(body, next_body));
            previous_line_was_joined = true;
        } else {
            output.push_str(newline);
            previous_line_was_joined = false;
        }
    }

    output
}

fn join_separator(left: &str, right: &str) -> &'static str {
    let left = left.chars().next_back();
    let right = right.chars().find(|ch| !matches!(ch, ' ' | '\t'));

    if matches!((left, right), (Some(left), Some(right)) if is_cjk(left) && is_cjk(right)) {
        ""
    } else {
        " "
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x2E80..=0x2FFF
            | 0x3000..=0x303F
            | 0x3040..=0x30FF
            | 0x31F0..=0x31FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFF00..=0xFFEF
    )
}

fn split_trailing_newline(line: &str) -> (&str, &str) {
    if let Some(body) = line.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = line.strip_suffix('\n') {
        (body, "\n")
    } else {
        (line, "")
    }
}

fn remove_prefix_whitespace(line: &str, width: usize) -> &str {
    let mut removed = 0;

    for (idx, ch) in line.char_indices() {
        if removed == width {
            return &line[idx..];
        }

        if ch == ' ' || ch == '\t' {
            removed += 1;
        } else {
            return &line[idx..];
        }
    }

    ""
}

fn is_blank_line(line: &str) -> bool {
    line.trim_matches([' ', '\t', '\r']).is_empty()
}

fn count_prefix_whitespace(line: &str) -> usize {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_common_leading_spaces() {
        assert_eq!(dedent_text("    alpha\n      beta\n"), "alpha\n  beta\n");
    }

    #[test]
    fn ignores_blank_lines_when_calculating_indent() {
        assert_eq!(
            dedent_text("    alpha\n\n      beta\n"),
            "alpha\n\n  beta\n"
        );
    }

    #[test]
    fn ignores_whitespace_only_lines_when_calculating_indent() {
        assert_eq!(
            dedent_text("    alpha\n  \n      beta\n"),
            "alpha\n\n  beta\n"
        );
    }

    #[test]
    fn leaves_text_unchanged_when_any_content_line_has_no_indent() {
        assert_eq!(dedent_text("alpha\n  beta\n"), "alpha\n  beta\n");
    }

    #[test]
    fn trims_trailing_spaces_and_tabs_from_each_line() {
        assert_eq!(dedent_text("  alpha  \n    beta\t \n"), "alpha\n  beta\n");
    }

    #[test]
    fn trims_trailing_whitespace_even_without_common_indent() {
        assert_eq!(dedent_text("alpha  \n  beta\t\n"), "alpha\n  beta\n");
    }

    #[test]
    fn treats_tabs_and_spaces_as_one_prefix_character_each() {
        assert_eq!(dedent_text("\t alpha\n\t\tbeta\n"), "alpha\nbeta\n");
    }

    #[test]
    fn preserves_missing_trailing_newline() {
        assert_eq!(dedent_text("  alpha\n    beta"), "alpha\n  beta");
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(
            dedent_text("  alpha  \r\n    beta\t\r\n"),
            "alpha\r\n  beta\r\n"
        );
    }

    #[test]
    fn strips_terminal_prompt_and_continuation_prefixes() {
        assert_eq!(
            dedent_text(
                "❯ Rewrite everything in Rust. I need cross-platform of the repo\n  scanner. Use cargo-dist to package all available platforms."
            ),
            "Rewrite everything in Rust. I need cross-platform of the repo scanner. Use cargo-dist to package all available platforms."
        );
    }

    #[test]
    fn strips_terminal_prompt_after_shared_indent() {
        assert_eq!(dedent_text("    ❯ alpha\n      beta\n"), "alpha beta\n");
    }

    #[test]
    fn strips_codex_prompt_and_unwraps_the_user_example() {
        assert_eq!(
            dedent_text(
                "› 我想要分析整個專案 src/ 的原始碼結構與系統架構、資料庫結構(if any)、專案詞彙表(統一名詞解釋)、技術棧、共用模組、主要流程與維護注意事項，請幫我制定一套詳盡的分\n  析計畫，幫助我快速理解本專案。請設計給一位資深工程師可以輕鬆理解的分析報告。請建立多份 Markdown 文件，方便我快速理解架構。相關圖表要參考 $design-doc-mermaid\n  技能進行繪製。"
            ),
            "我想要分析整個專案 src/ 的原始碼結構與系統架構、資料庫結構(if any)、專案詞彙表(統一名詞解釋)、技術棧、共用模組、主要流程與維護注意事項，請幫我制定一套詳盡的分析計畫，幫助我快速理解本專案。請設計給一位資深工程師可以輕鬆理解的分析報告。請建立多份 Markdown 文件，方便我快速理解架構。相關圖表要參考 $design-doc-mermaid 技能進行繪製。"
        );
    }

    #[test]
    fn strips_markdown_style_blockquote_prefix_and_keeps_lines() {
        assert_eq!(
            dedent_text("> Hello 123\n  I'm Will.\n"),
            "Hello 123\nI'm Will.\n"
        );
    }

    #[test]
    fn joins_cjk_lines_without_a_space() {
        assert_eq!(dedent_text("› 中文，\n  測試\n"), "中文，測試\n");
    }

    #[test]
    fn joins_latin_and_mixed_lines_with_one_space() {
        assert_eq!(dedent_text("› alpha\n  beta\n"), "alpha beta\n");
        assert_eq!(dedent_text("› Mermaid\n  技能\n"), "Mermaid 技能\n");
    }

    #[test]
    fn preserves_blank_lines_as_paragraph_breaks() {
        assert_eq!(
            dedent_text("› 第一段\n  仍在同段\n\n  第二段\n  仍在第二段\n"),
            "第一段仍在同段\n\n第二段仍在第二段\n"
        );
    }

    #[test]
    fn preserves_whitespace_only_blank_lines_as_paragraph_breaks() {
        assert_eq!(
            dedent_text("› 第一段\n  仍在同段\n  \n  第二段\n"),
            "第一段仍在同段\n\n第二段\n"
        );
    }

    #[test]
    fn preserves_crlf_paragraph_breaks_when_unwrapping_prompt() {
        assert_eq!(
            dedent_text("› 第一段\r\n  仍在同段\r\n\r\n  第二段\r\n"),
            "第一段仍在同段\r\n\r\n第二段\r\n"
        );
    }

    #[test]
    fn leaves_malformed_prompt_blocks_unchanged() {
        assert_eq!(
            dedent_text("› 第一行\n不是續行格式\n"),
            "› 第一行\n不是續行格式\n"
        );
    }

    #[test]
    fn keeps_all_blank_input_as_blank() {
        assert_eq!(dedent_text("  \n\t\n"), "\n\n");
    }

    #[test]
    fn keeps_empty_input_empty() {
        assert_eq!(dedent_text(""), "");
    }

    #[test]
    fn rejects_invalid_utf8() {
        assert_eq!(text_from_bytes(vec![0xff]), Err(DedentError::InvalidUtf8));
    }
}
