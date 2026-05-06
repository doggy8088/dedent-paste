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

pub fn dedent_text(input: &str) -> String {
    let min_indent = input
        .lines()
        .filter(|line| !is_blank_line(line))
        .map(count_prefix_whitespace)
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return input.to_owned();
    }

    input
        .split_inclusive('\n')
        .map(|line| dedent_line(line, min_indent))
        .collect()
}

pub fn text_from_bytes(bytes: Vec<u8>) -> Result<String, DedentError> {
    String::from_utf8(bytes).map_err(|_| DedentError::InvalidUtf8)
}

fn dedent_line(line: &str, width: usize) -> String {
    let (body, newline) = match line.strip_suffix('\n') {
        Some(body) => (body, "\n"),
        None => (line, ""),
    };

    format!("{}{}", remove_prefix_whitespace(body, width), newline)
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
            dedent_text("  alpha\r\n    beta\r\n"),
            "alpha\r\n  beta\r\n"
        );
    }

    #[test]
    fn keeps_all_blank_input_as_blank() {
        assert_eq!(dedent_text("  \n\t\n"), "  \n\t\n");
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
