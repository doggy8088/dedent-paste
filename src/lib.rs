use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use base64::Engine as _;

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
            stripped.push_str(body.trim_matches(is_inline_space));
        } else {
            let Some(body) = body.strip_prefix(indent) else {
                return (None, None);
            };
            let Some(body) = strip_inline_spaces(body, 2) else {
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
        if is_inline_space(ch) {
            continue;
        }

        return strip_prompt_symbol(&line[idx..]).map(|(_, mode)| (&line[..idx], mode));
    }

    None
}

fn strip_prompt_marker<'a>(line: &'a str, indent: &str) -> Option<(&'a str, PromptMode)> {
    strip_prompt_symbol(line.strip_prefix(indent)?)
}

fn strip_prompt_symbol(line: &str) -> Option<(&str, PromptMode)> {
    let (rest, mode) = if let Some(rest) = line.strip_prefix(['❯', '›']) {
        (rest, PromptMode::Unwrap)
    } else if let Some(rest) = line.strip_prefix('>') {
        (rest, PromptMode::PreserveLines)
    } else {
        return None;
    };

    strip_inline_spaces(rest, 1).map(|rest| (rest, mode))
}

fn strip_inline_spaces(line: &str, count: usize) -> Option<&str> {
    let mut chars = line.chars();
    for _ in 0..count {
        if !is_inline_space(chars.next()?) {
            return None;
        }
    }
    Some(chars.as_str())
}

// Terminal UIs (e.g. Codex CLI) may render the prompt and continuation
// indentation with non-breaking or other Unicode spaces that survive
// copy-and-paste, so prompt detection must not assume ASCII spaces.
fn is_inline_space(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '\t' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
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
            body.trim_start_matches(is_inline_space)
        } else {
            body
        };

        output.push_str(body);

        let next_body = lines.get(index + 1).map(|(body, _)| *body);
        let should_join = !newline.is_empty()
            && !is_blank_line(body)
            && !ends_with_sentence_terminator(body)
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

// Terminal wrapping breaks lines mid-sentence at the window width, so a line
// that ends exactly at sentence-ending punctuation is treated as a real line
// break the user typed, not a visual wrap.
fn ends_with_sentence_terminator(line: &str) -> bool {
    let last = line.chars().rev().find(|ch| !is_closing_wrapper(*ch));

    matches!(
        last,
        Some('。' | '．' | '！' | '？' | '…' | '.' | '!' | '?')
    )
}

fn is_closing_wrapper(ch: char) -> bool {
    matches!(
        ch,
        '」' | '』' | '）' | '】' | '〉' | '》' | ')' | ']' | '}' | '"' | '\'' | '”' | '’'
    )
}

fn join_separator(left: &str, right: &str) -> &'static str {
    let left = left.chars().next_back();
    let right = right.chars().find(|ch| !is_inline_space(*ch));

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
    line.trim_matches(|ch| is_inline_space(ch) || ch == '\r')
        .is_empty()
}

fn count_prefix_whitespace(line: &str) -> usize {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .count()
}

pub const ENV_GEMINI_API_KEY: &str = "DEDENT_PASTE_GEMINI_API_KEY";
pub const ENV_GEMINI_API_KEY_FALLBACK: &str = "GEMINI_API_KEY";
pub const ENV_GEMINI_MODEL: &str = "DEDENT_PASTE_GEMINI_MODEL";
pub const ENV_OUTPUT_LANGUAGE: &str = "DEDENT_PASTE_LANG";
pub const ENV_POSIX_LANG: &str = "LANG";
pub const ENV_SYSTEM_PROMPT: &str = "DEDENT_PASTE_GEMINI_SYSTEM_PROMPT";
pub const ENV_SYSTEM_PROMPT_FILE: &str = "DEDENT_PASTE_GEMINI_SYSTEM_PROMPT_FILE";
pub const ENV_TIMEOUT_SECS: &str = "DEDENT_PASTE_GEMINI_TIMEOUT_SECS";
pub const ENV_LOG_FILE: &str = "DEDENT_PASTE_LOG_FILE";

pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.7-flash";
pub const DEFAULT_OUTPUT_LANGUAGE: &str = "zh-TW";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
pub const GEMINI_MAX_REQUEST_BYTES: usize = 20 * 1024 * 1024;

pub const DEFAULT_SYSTEM_PROMPT: &str = "\
You convert one image from the user's clipboard into text.

If the image contains readable text: transcribe it faithfully and completely in its \
original language. Preserve the structure using Markdown: headings, bullet and numbered \
lists, tables for tabular content, and fenced code blocks for code or terminal output. \
Join line breaks that exist only because of visual wrapping. Never add, translate, or \
invent content.

If the image contains no readable text: describe the image concisely in {language}.

Any explanation or description you write yourself must be in {language}.

Output only the final text: no preamble, no commentary, and do not wrap the entire \
answer in a code fence.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeminiError {
    MissingApiKey,
    NoImage,
    ImageTooLarge { request_bytes: usize },
    ImageConversion { reason: String },
    Api { status: u16, message: String },
    EmptyResponse { block_reason: Option<String> },
    InvalidResponse,
    PromptFileUnreadable { path: String, reason: String },
}

impl fmt::Display for GeminiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeminiError::MissingApiKey => write!(
                f,
                "Gemini API key not set; set {ENV_GEMINI_API_KEY} or {ENV_GEMINI_API_KEY_FALLBACK} to enable image-to-text"
            ),
            GeminiError::NoImage => write!(f, "clipboard has no text or image"),
            GeminiError::ImageTooLarge { request_bytes } => write!(
                f,
                "clipboard image is too large for the Gemini API (request would be {request_bytes} bytes; the limit is {GEMINI_MAX_REQUEST_BYTES})"
            ),
            GeminiError::ImageConversion { reason } => {
                write!(f, "failed to convert clipboard image to PNG: {reason}")
            }
            GeminiError::Api { status, message } => {
                write!(f, "Gemini API error (HTTP {status}): {message}")
            }
            GeminiError::EmptyResponse { block_reason } => match block_reason {
                Some(reason) => write!(f, "Gemini returned no text (block reason: {reason})"),
                None => write!(f, "Gemini returned no text"),
            },
            GeminiError::InvalidResponse => {
                write!(f, "Gemini returned an unexpected response format")
            }
            GeminiError::PromptFileUnreadable { path, reason } => {
                write!(f, "cannot read system prompt file {path}: {reason}")
            }
        }
    }
}

impl Error for GeminiError {}

pub struct ClipboardImage {
    pub mime_type: &'static str,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct GeminiSettings {
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    pub user_instruction: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemPromptSource {
    Inline(String),
    File(String),
    BuiltIn,
}

fn env_non_empty(get: &impl Fn(&str) -> Option<String>, name: &str) -> Option<String> {
    get(name).filter(|value| !value.trim().is_empty())
}

pub fn resolve_api_key(get: impl Fn(&str) -> Option<String>) -> Result<String, GeminiError> {
    env_non_empty(&get, ENV_GEMINI_API_KEY)
        .or_else(|| env_non_empty(&get, ENV_GEMINI_API_KEY_FALLBACK))
        .ok_or(GeminiError::MissingApiKey)
}

pub fn resolve_model(get: impl Fn(&str) -> Option<String>) -> String {
    env_non_empty(&get, ENV_GEMINI_MODEL).unwrap_or_else(|| DEFAULT_GEMINI_MODEL.to_string())
}

pub fn resolve_output_language(get: impl Fn(&str) -> Option<String>) -> String {
    env_non_empty(&get, ENV_OUTPUT_LANGUAGE)
        .or_else(|| {
            env_non_empty(&get, ENV_POSIX_LANG)
                .and_then(|locale| language_tag_from_posix_locale(&locale))
        })
        .unwrap_or_else(|| DEFAULT_OUTPUT_LANGUAGE.to_string())
}

pub fn language_tag_from_posix_locale(locale: &str) -> Option<String> {
    let base = locale.split(['.', '@']).next().unwrap_or("");
    if base.is_empty() || base == "C" || base == "POSIX" {
        return None;
    }

    let mut parts = base.split('_');
    let language = parts.next()?;
    if language.is_empty() {
        return None;
    }

    match parts.next() {
        Some(region) if !region.is_empty() => Some(format!("{language}-{region}")),
        _ => Some(language.to_string()),
    }
}

pub fn resolve_system_prompt_source(get: impl Fn(&str) -> Option<String>) -> SystemPromptSource {
    if let Some(prompt) = env_non_empty(&get, ENV_SYSTEM_PROMPT) {
        SystemPromptSource::Inline(prompt)
    } else if let Some(path) = env_non_empty(&get, ENV_SYSTEM_PROMPT_FILE) {
        SystemPromptSource::File(path)
    } else {
        SystemPromptSource::BuiltIn
    }
}

pub fn render_prompt(template: &str, language: &str) -> String {
    template.replace("{language}", language)
}

pub fn resolve_timeout(get: impl Fn(&str) -> Option<String>) -> Duration {
    env_non_empty(&get, ENV_TIMEOUT_SECS)
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_TIMEOUT)
}

pub fn resolve_gemini_settings(
    get: impl Fn(&str) -> Option<String>,
    read_file: impl Fn(&str) -> Result<String, String>,
) -> Result<GeminiSettings, GeminiError> {
    let api_key = resolve_api_key(&get)?;
    let model = resolve_model(&get);
    let language = resolve_output_language(&get);

    let template = match resolve_system_prompt_source(&get) {
        SystemPromptSource::Inline(prompt) => prompt,
        SystemPromptSource::File(path) => {
            read_file(&path).map_err(|reason| GeminiError::PromptFileUnreadable { path, reason })?
        }
        SystemPromptSource::BuiltIn => DEFAULT_SYSTEM_PROMPT.to_string(),
    };

    Ok(GeminiSettings {
        api_key,
        model,
        system_prompt: render_prompt(&template, &language),
        user_instruction: format!(
            "Convert this image to text per the system instructions. Preferred output language: {language}."
        ),
        timeout: resolve_timeout(&get),
    })
}

pub fn build_generate_content_request(
    system_prompt: &str,
    user_instruction: &str,
    image: &ClipboardImage,
) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(&image.data);

    serde_json::json!({
        "system_instruction": {
            "parts": [{ "text": system_prompt }]
        },
        "contents": [{
            "parts": [
                { "inline_data": { "mime_type": image.mime_type, "data": encoded } },
                { "text": user_instruction }
            ]
        }]
    })
    .to_string()
}

pub fn extract_text_from_response(body: &str) -> Result<String, GeminiError> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| GeminiError::InvalidResponse)?;

    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(|message| message.as_str())
            .unwrap_or("unknown error")
            .to_string();
        let status = error
            .get("code")
            .and_then(|code| code.as_u64())
            .unwrap_or(0) as u16;
        return Err(GeminiError::Api { status, message });
    }

    let text: String = value
        .get("candidates")
        .and_then(|candidates| candidates.get(0))
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.as_array())
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(|text| text.as_str()))
                .collect()
        })
        .unwrap_or_default();

    if text.trim().is_empty() {
        let block_reason = value
            .get("promptFeedback")
            .and_then(|feedback| feedback.get("blockReason"))
            .and_then(|reason| reason.as_str())
            .map(str::to_string);
        return Err(GeminiError::EmptyResponse { block_reason });
    }

    Ok(text)
}

pub fn check_image_size(image_bytes: usize, prompt_bytes: usize) -> Result<(), GeminiError> {
    const ENVELOPE_BYTES: usize = 1024;

    let request_bytes = image_bytes.div_ceil(3) * 4 + prompt_bytes + ENVELOPE_BYTES;
    if request_bytes > GEMINI_MAX_REQUEST_BYTES {
        Err(GeminiError::ImageTooLarge { request_bytes })
    } else {
        Ok(())
    }
}

pub fn parse_osascript_image_data(stdout: &str) -> Option<Vec<u8>> {
    let hex = stdout
        .trim()
        .strip_prefix("«data PNGf")?
        .strip_suffix('»')?;
    if hex.is_empty() || hex.len() % 2 != 0 {
        return None;
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks(2) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        bytes.push((high * 16 + low) as u8);
    }

    Some(bytes)
}

pub fn applescript_string_literal(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for ch in text.chars() {
        match ch {
            '"' | '\\' => {
                literal.push('\\');
                literal.push(ch);
            }
            '\n' | '\r' => literal.push(' '),
            _ => literal.push(ch),
        }
    }
    literal.push('"');
    literal
}

#[cfg(windows)]
pub fn bmp_to_png(bmp: &[u8]) -> Result<Vec<u8>, GeminiError> {
    let decoded =
        image::load_from_memory_with_format(bmp, image::ImageFormat::Bmp).map_err(|error| {
            GeminiError::ImageConversion {
                reason: error.to_string(),
            }
        })?;

    let mut png = Vec::new();
    decoded
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|error| GeminiError::ImageConversion {
            reason: error.to_string(),
        })?;

    Ok(png)
}

pub fn is_silent_error(error: &GeminiError) -> bool {
    matches!(error, GeminiError::MissingApiKey | GeminiError::NoImage)
}

pub fn resolve_log_path(
    get: impl Fn(&str) -> Option<String>,
    default: Option<PathBuf>,
) -> Option<PathBuf> {
    env_non_empty(&get, ENV_LOG_FILE)
        .map(PathBuf::from)
        .or(default)
}

pub fn format_log_line(timestamp: &str, level: &str, message: &str) -> String {
    let message = message.replace(['\n', '\r'], " ");
    format!("{timestamp} [{level}] {message}\n")
}

pub fn format_timestamp(unix_seconds: u64) -> String {
    let (year, month, day) = civil_from_days((unix_seconds / 86_400) as i64);
    let seconds_of_day = unix_seconds % 86_400;

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds_of_day / 3600,
        (seconds_of_day % 3600) / 60,
        seconds_of_day % 60
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = (days - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    } as u32;

    (if month <= 2 { year + 1 } else { year }, month, day)
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
    fn strips_prompt_when_marker_and_indent_use_non_breaking_spaces() {
        assert_eq!(
            dedent_text(
                "❯\u{00A0}我需要在「系統管理」頁面加入 Tab 分類，方便切換與操作。\n\u{00A0}\u{00A0}系統管理打開後，網址列應該要有 deep linking 能力。\n"
            ),
            "我需要在「系統管理」頁面加入 Tab 分類，方便切換與操作。\n系統管理打開後，網址列應該要有 deep linking 能力。\n"
        );
    }

    #[test]
    fn joins_wrapped_lines_when_prompt_space_is_non_breaking() {
        assert_eq!(dedent_text("›\u{00A0}alpha\n  beta\n"), "alpha beta\n");
        assert_eq!(
            dedent_text("›\u{3000}中文，\n\u{00A0}\u{00A0}測試\n"),
            "中文，測試\n"
        );
    }

    #[test]
    fn strips_blockquote_prefix_with_non_breaking_space() {
        assert_eq!(
            dedent_text(">\u{00A0}Hello 123\n\u{00A0}\u{00A0}I'm Will.\n"),
            "Hello 123\nI'm Will.\n"
        );
    }

    #[test]
    fn preserves_line_breaks_after_sentence_ending_punctuation() {
        assert_eq!(
            dedent_text(
                "❯ 我需要在「系統管理」頁面加入 Tab 分類，將不同的管理功能，區分成不同的群組，方便切換與操作。\n  系統管理打開後，網址列應該要有 deep linking 能力。切換到不同 Tab 應該也要有 deep linking 能力。\n"
            ),
            "我需要在「系統管理」頁面加入 Tab 分類，將不同的管理功能，區分成不同的群組，方便切換與操作。\n系統管理打開後，網址列應該要有 deep linking 能力。切換到不同 Tab 應該也要有 deep linking 能力。\n"
        );
    }

    #[test]
    fn preserves_line_breaks_after_latin_sentence_ending_punctuation() {
        assert_eq!(
            dedent_text("❯ First sentence.\n  Second line\n"),
            "First sentence.\nSecond line\n"
        );
    }

    #[test]
    fn preserves_line_breaks_when_terminator_is_wrapped_in_closing_punctuation() {
        assert_eq!(
            dedent_text("› 第一行（完成。）\n  第二行\n"),
            "第一行（完成。）\n第二行\n"
        );
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

    fn env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        }
    }

    #[test]
    fn derives_language_tag_from_posix_locale() {
        assert_eq!(
            language_tag_from_posix_locale("en_US.UTF-8"),
            Some("en-US".to_string())
        );
        assert_eq!(
            language_tag_from_posix_locale("zh_TW.UTF-8"),
            Some("zh-TW".to_string())
        );
        assert_eq!(
            language_tag_from_posix_locale("ja_JP"),
            Some("ja-JP".to_string())
        );
        assert_eq!(language_tag_from_posix_locale("en"), Some("en".to_string()));
        assert_eq!(
            language_tag_from_posix_locale("de_DE@euro"),
            Some("de-DE".to_string())
        );
        assert_eq!(language_tag_from_posix_locale("C"), None);
        assert_eq!(language_tag_from_posix_locale("C.UTF-8"), None);
        assert_eq!(language_tag_from_posix_locale("POSIX"), None);
        assert_eq!(language_tag_from_posix_locale(""), None);
    }

    #[test]
    fn output_language_prefers_tool_env_over_posix_lang() {
        assert_eq!(
            resolve_output_language(env(&[
                (ENV_OUTPUT_LANGUAGE, "ja"),
                (ENV_POSIX_LANG, "en_US.UTF-8"),
            ])),
            "ja"
        );
        assert_eq!(
            resolve_output_language(env(&[(ENV_POSIX_LANG, "en_US.UTF-8")])),
            "en-US"
        );
        assert_eq!(resolve_output_language(env(&[])), "zh-TW");
        assert_eq!(
            resolve_output_language(env(&[(ENV_OUTPUT_LANGUAGE, ""), (ENV_POSIX_LANG, "C")])),
            "zh-TW"
        );
    }

    #[test]
    fn api_key_prefers_tool_env_and_reports_both_names_when_missing() {
        assert_eq!(
            resolve_api_key(env(&[
                (ENV_GEMINI_API_KEY, "tool-key"),
                (ENV_GEMINI_API_KEY_FALLBACK, "shared-key"),
            ])),
            Ok("tool-key".to_string())
        );
        assert_eq!(
            resolve_api_key(env(&[(ENV_GEMINI_API_KEY_FALLBACK, "shared-key")])),
            Ok("shared-key".to_string())
        );

        let error = resolve_api_key(env(&[])).unwrap_err();
        assert_eq!(error, GeminiError::MissingApiKey);
        let message = error.to_string();
        assert!(message.contains(ENV_GEMINI_API_KEY));
        assert!(message.contains(ENV_GEMINI_API_KEY_FALLBACK));
    }

    #[test]
    fn model_defaults_and_overrides() {
        assert_eq!(resolve_model(env(&[])), "gemini-3.7-flash");
        assert_eq!(
            resolve_model(env(&[(ENV_GEMINI_MODEL, "gemini-4-pro")])),
            "gemini-4-pro"
        );
    }

    #[test]
    fn system_prompt_source_prefers_inline_over_file() {
        assert_eq!(
            resolve_system_prompt_source(env(&[
                (ENV_SYSTEM_PROMPT, "inline"),
                (ENV_SYSTEM_PROMPT_FILE, "/tmp/prompt.txt"),
            ])),
            SystemPromptSource::Inline("inline".to_string())
        );
        assert_eq!(
            resolve_system_prompt_source(env(&[(ENV_SYSTEM_PROMPT_FILE, "/tmp/prompt.txt")])),
            SystemPromptSource::File("/tmp/prompt.txt".to_string())
        );
        assert_eq!(
            resolve_system_prompt_source(env(&[])),
            SystemPromptSource::BuiltIn
        );
    }

    #[test]
    fn renders_language_placeholder() {
        assert_eq!(
            render_prompt("Describe in {language}. Reply in {language}.", "zh-TW"),
            "Describe in zh-TW. Reply in zh-TW."
        );
        assert_eq!(render_prompt("no placeholder", "zh-TW"), "no placeholder");
    }

    #[test]
    fn timeout_defaults_and_overrides() {
        assert_eq!(resolve_timeout(env(&[])), Duration::from_secs(60));
        assert_eq!(
            resolve_timeout(env(&[(ENV_TIMEOUT_SECS, "120")])),
            Duration::from_secs(120)
        );
        assert_eq!(
            resolve_timeout(env(&[(ENV_TIMEOUT_SECS, "not-a-number")])),
            Duration::from_secs(60)
        );
        assert_eq!(
            resolve_timeout(env(&[(ENV_TIMEOUT_SECS, "0")])),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn settings_read_prompt_file_and_surface_read_errors() {
        let settings = resolve_gemini_settings(
            env(&[
                (ENV_GEMINI_API_KEY, "key"),
                (ENV_SYSTEM_PROMPT_FILE, "/tmp/prompt.txt"),
            ]),
            |path| {
                assert_eq!(path, "/tmp/prompt.txt");
                Ok("from file in {language}".to_string())
            },
        )
        .unwrap();
        assert_eq!(settings.system_prompt, "from file in zh-TW");
        assert!(settings.user_instruction.contains("zh-TW"));

        let error = resolve_gemini_settings(
            env(&[
                (ENV_GEMINI_API_KEY, "key"),
                (ENV_SYSTEM_PROMPT_FILE, "/tmp/prompt.txt"),
            ]),
            |_| Err("permission denied".to_string()),
        )
        .unwrap_err();
        assert_eq!(
            error,
            GeminiError::PromptFileUnreadable {
                path: "/tmp/prompt.txt".to_string(),
                reason: "permission denied".to_string(),
            }
        );
    }

    #[test]
    fn builds_generate_content_request_with_escaped_prompts() {
        let image = ClipboardImage {
            mime_type: "image/png",
            data: vec![0x89, b'P', b'N', b'G'],
        };
        let body = build_generate_content_request(
            "系統 \"prompt\"\nwith newline",
            "instruction 中文",
            &image,
        );

        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            value["system_instruction"]["parts"][0]["text"],
            "系統 \"prompt\"\nwith newline"
        );
        let parts = &value["contents"][0]["parts"];
        assert_eq!(parts[0]["inline_data"]["mime_type"], "image/png");
        let encoded = parts[0]["inline_data"]["data"].as_str().unwrap();
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap(),
            image.data
        );
        assert_eq!(parts[1]["text"], "instruction 中文");
    }

    #[test]
    fn extracts_text_from_response_variants() {
        assert_eq!(
            extract_text_from_response(
                r#"{"candidates":[{"content":{"parts":[{"text":"hello"}]}}]}"#
            ),
            Ok("hello".to_string())
        );
        assert_eq!(
            extract_text_from_response(
                r#"{"candidates":[{"content":{"parts":[{"text":"a"},{"text":"b"}]}}]}"#
            ),
            Ok("ab".to_string())
        );
        assert_eq!(
            extract_text_from_response(r#"{"error":{"code":429,"message":"quota exceeded"}}"#),
            Err(GeminiError::Api {
                status: 429,
                message: "quota exceeded".to_string(),
            })
        );
        assert_eq!(
            extract_text_from_response(r#"{"candidates":[]}"#),
            Err(GeminiError::EmptyResponse { block_reason: None })
        );
        assert_eq!(
            extract_text_from_response(r#"{"promptFeedback":{"blockReason":"SAFETY"}}"#),
            Err(GeminiError::EmptyResponse {
                block_reason: Some("SAFETY".to_string()),
            })
        );
        assert_eq!(
            extract_text_from_response("not json"),
            Err(GeminiError::InvalidResponse)
        );
    }

    #[test]
    fn checks_image_size_against_request_limit() {
        assert_eq!(check_image_size(1024, 1024), Ok(()));
        let error = check_image_size(GEMINI_MAX_REQUEST_BYTES, 0).unwrap_err();
        assert!(matches!(error, GeminiError::ImageTooLarge { .. }));
        assert!(error.to_string().contains("too large"));
    }

    #[test]
    fn parses_osascript_image_data() {
        assert_eq!(
            parse_osascript_image_data("«data PNGf89504E47»\n"),
            Some(vec![0x89, 0x50, 0x4E, 0x47])
        );
        assert_eq!(
            parse_osascript_image_data("«data PNGf89504e47»"),
            Some(vec![0x89, 0x50, 0x4E, 0x47])
        );
        assert_eq!(parse_osascript_image_data("«data TIFF89504E47»"), None);
        assert_eq!(parse_osascript_image_data("«data PNGf895»"), None);
        assert_eq!(parse_osascript_image_data("«data PNGfZZ»"), None);
        assert_eq!(parse_osascript_image_data("«data PNGf»"), None);
        assert_eq!(parse_osascript_image_data("plain text"), None);
    }

    #[test]
    fn escapes_applescript_string_literals() {
        assert_eq!(
            applescript_string_literal(r#"say "hi" \ bye"#),
            r#""say \"hi\" \\ bye""#
        );
        assert_eq!(
            applescript_string_literal("line1\nline2"),
            "\"line1 line2\""
        );
        assert_eq!(applescript_string_literal("中文訊息"), "\"中文訊息\"");
    }

    #[test]
    fn formats_log_lines_on_a_single_line() {
        assert_eq!(
            format_log_line("2026-08-25T01:02:03Z", "error", "boom\nsecond"),
            "2026-08-25T01:02:03Z [error] boom second\n"
        );
    }

    #[test]
    fn formats_unix_timestamps_as_utc() {
        assert_eq!(format_timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_timestamp(1_756_080_000), "2025-08-25T00:00:00Z");
        assert_eq!(format_timestamp(951_827_696), "2000-02-29T12:34:56Z");
    }

    #[test]
    fn log_path_env_override_wins() {
        assert_eq!(
            resolve_log_path(
                env(&[(ENV_LOG_FILE, "/tmp/custom.log")]),
                Some(PathBuf::from("/default.log")),
            ),
            Some(PathBuf::from("/tmp/custom.log"))
        );
        assert_eq!(
            resolve_log_path(env(&[]), Some(PathBuf::from("/default.log"))),
            Some(PathBuf::from("/default.log"))
        );
        assert_eq!(resolve_log_path(env(&[]), None), None);
    }

    #[test]
    fn only_missing_key_and_missing_image_are_silent() {
        assert!(is_silent_error(&GeminiError::MissingApiKey));
        assert!(is_silent_error(&GeminiError::NoImage));
        assert!(!is_silent_error(&GeminiError::InvalidResponse));
        assert!(!is_silent_error(&GeminiError::Api {
            status: 500,
            message: "boom".to_string(),
        }));
    }

    #[cfg(windows)]
    #[test]
    fn converts_bmp_to_png() {
        let mut bmp = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(2, 2, |x, y| {
            image::Rgb([(x * 255) as u8, (y * 255) as u8, 128])
        }))
        .write_to(&mut std::io::Cursor::new(&mut bmp), image::ImageFormat::Bmp)
        .unwrap();

        let png = bmp_to_png(&bmp).unwrap();
        assert_eq!(&png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png).unwrap();
        assert_eq!(
            decoded.to_rgb8().get_pixel(1, 1),
            &image::Rgb([255, 255, 128])
        );

        assert!(bmp_to_png(b"not a bmp").is_err());
    }
}
