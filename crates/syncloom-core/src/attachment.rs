//! Bounded text-file acquisition. Names and MIME are hints, never paths or byte validation.
use crate::{Source, StudioError};
use serde::{Deserialize, Serialize};

pub const MAX_BYTES: usize = 262_144;
pub const MAX_TEXT_CHARS: usize = 8_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub filename: String,
    pub format: String,
    pub byte_count: usize,
    pub content_sha256: String,
    pub captured_at: String,
    pub extracted_text: String,
}

fn directional(c: char) -> bool {
    matches!(c, '\u{061c}' | '\u{200e}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// File extraction and native browser form serialization use the same canonical line endings.
pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn valid_filename(filename: &str) -> bool {
    !filename.is_empty()
        && filename.len() <= 128
        && !filename
            .chars()
            .any(|c| c.is_control() || directional(c) || matches!(c, '/' | '\\'))
}

fn format(filename: &str, mime: Option<&str>) -> Result<&'static str, StudioError> {
    if !valid_filename(filename) {
        return Err(StudioError::AttachmentInvalid);
    }
    let lower = filename.to_ascii_lowercase();
    let format = if lower.ends_with(".txt") {
        "plain_text"
    } else if lower.ends_with(".md") {
        "markdown"
    } else {
        return Err(StudioError::AttachmentUnsupported);
    };
    let mime = mime
        .unwrap_or("application/octet-stream")
        .to_ascii_lowercase();
    if !matches!(mime.as_str(), "text/plain" | "application/octet-stream")
        && !(format == "markdown" && mime == "text/markdown")
    {
        return Err(StudioError::AttachmentUnsupported);
    }
    Ok(format)
}

fn text(bytes: &[u8]) -> Result<String, StudioError> {
    if bytes.len() > MAX_BYTES {
        return Err(StudioError::AttachmentTooLarge);
    }
    let utf8 = std::str::from_utf8(bytes).map_err(|_| StudioError::AttachmentInvalid)?;
    // Only the optional first BOM and line endings change. No lossy decode or truncation.
    let text = normalize_line_endings(utf8.strip_prefix('\u{feff}').unwrap_or(utf8));
    if text.trim_start().starts_with("%PDF-") {
        return Err(StudioError::AttachmentUnsupported);
    }
    if text
        .chars()
        .any(|c| (c.is_control() && !matches!(c, '\n' | '\t')) || directional(c))
    {
        return Err(StudioError::AttachmentInvalid);
    }
    if text.chars().count() > MAX_TEXT_CHARS {
        return Err(StudioError::AttachmentTooLarge);
    }
    crate::studio::checked_thought(&text)?;
    Ok(text)
}

pub(crate) fn extract(
    filename: &str,
    mime: Option<&str>,
    bytes: &[u8],
    captured_at: String,
) -> Result<Source, StudioError> {
    let text = text(bytes)?;
    let format = format(filename, mime)?;
    Ok(Source {
        kind: "attachment".into(),
        raw_input: filename.into(),
        retrieval: None,
        attachment: Some(Attachment {
            filename: filename.into(),
            format: format.into(),
            byte_count: bytes.len(),
            content_sha256: crate::visual::digest(bytes),
            captured_at,
            extracted_text: text,
        }),
    })
}
