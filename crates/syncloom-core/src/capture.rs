//! Founder-text suggestions and non-approval previews. No URL fetching or draft writing.
use crate::{Channel, Fit, StudioError, ThoughtResponse, fit, normalize_thought};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Suggestion {
    pub channel: Channel,
    pub platform: &'static str,
    pub format_hint: &'static str,
    pub edit_notes: &'static str,
    /// Migrated heuristic weight, not a measured probability of success.
    pub confidence: &'static str,
    pub fit: Fit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Preview {
    pub revision: String,
    pub text: String,
    pub normalized_thought: String,
    pub channels: Vec<Channel>,
    pub suggestions: Vec<Suggestion>,
    pub approval: Option<String>,
    pub publish_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureConfirmRequest {
    pub text: String,
    pub channels: Vec<String>,
    pub expected_revision: String,
}

impl CaptureConfirmRequest {
    pub(crate) fn checked_preview(&self) -> Result<Preview, StudioError> {
        let preview = preview(&self.text, &self.channels)?;
        if !valid_revision(&self.expected_revision) {
            return Err(StudioError::InvalidRequest);
        }
        if self.expected_revision != preview.revision {
            return Err(StudioError::RevisionConflict);
        }
        Ok(preview)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureConfirmResponse {
    pub created: bool,
    pub thought: ThoughtResponse,
}

/// Empty piece_ids is an incomplete claim, never permission to draft again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureRecord {
    pub revision: String,
    pub channels: Vec<String>,
    pub group_id: String,
    pub piece_ids: Vec<String>,
}

pub fn suggest(text: &str) -> Result<Vec<Suggestion>, StudioError> {
    let normalized = crate::studio::checked_thought(text)?;
    let lower = text.to_ascii_lowercase();
    let hints = if lower.contains("http://") || lower.contains("https://") {
        vec![
            (
                Channel::LinkedinText,
                "text post with link",
                "Put the opinion first; drop the URL at the end.",
                "0.88",
            ),
            (
                Channel::X,
                "short post or thread seed",
                "Compress to one sharp claim; link at the end if needed.",
                "0.80",
            ),
            (
                Channel::Threads,
                "conversation starter",
                "Ask one follow-up question after the link takeaway.",
                "0.72",
            ),
        ]
    } else {
        text_hints(text.trim().chars().count())
    };
    let mut suggestions: Vec<_> = hints
        .into_iter()
        .map(
            |(channel, format_hint, edit_notes, confidence)| Suggestion {
                channel,
                platform: channel.platform(),
                format_hint,
                edit_notes,
                confidence,
                fit: fit(channel, &normalized),
            },
        )
        .collect();
    suggestions.sort_by(|a, b| {
        b.confidence
            .cmp(a.confidence)
            .then_with(|| a.channel.name().cmp(b.channel.name()))
    });
    Ok(suggestions)
}

fn text_hints(length: usize) -> Vec<(Channel, &'static str, &'static str, &'static str)> {
    // These migrated plain-length hints are not platform limits. fit() owns those.
    let mut hints = vec![
        (
            Channel::LinkedinText,
            "authority text post",
            "Open with the lesson, then one concrete example.",
            if length > 200 { "0.90" } else { "0.78" },
        ),
        (
            Channel::FacebookPage,
            "community post",
            "Soften the tone; sound like a note to friends.",
            "0.60",
        ),
    ];
    if length <= 280 {
        hints.extend([
            (
                Channel::X,
                "single post",
                "Keep as one punchy line; no thread needed.",
                "0.86",
            ),
            (
                Channel::Threads,
                "short conversation post",
                "Optional: add a question to invite replies.",
                "0.80",
            ),
        ]);
    }
    hints
}

pub fn preview(text: &str, channels: &[String]) -> Result<Preview, StudioError> {
    let suggestions = suggest(text)?;
    let selected = crate::studio::parse_channels(channels)?;
    if selected
        .iter()
        .any(|channel| !suggestions.iter().any(|s| s.channel == *channel))
    {
        return Err(StudioError::InvalidRequest);
    }
    let shown = suggestions
        .into_iter()
        .filter(|s| selected.contains(&s.channel))
        .collect();
    Ok(Preview {
        revision: revision_for(text, &selected),
        text: text.to_owned(),
        normalized_thought: normalize_thought(text),
        channels: selected,
        suggestions: shown,
        approval: None,
        publish_status: None,
    })
}

pub fn valid_revision(revision: &str) -> bool {
    revision.len() == 64
        && revision
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn revision_for(text: &str, channels: &[Channel]) -> String {
    let mut names: Vec<_> = channels.iter().map(|channel| channel.name()).collect();
    names.sort_unstable();
    // Length framing prevents source text containing channel names from aliasing a set.
    crate::visual::digest(
        format!("capture-v1:{}:{text}:{}", text.len(), names.join(",")).as_bytes(),
    )
}
