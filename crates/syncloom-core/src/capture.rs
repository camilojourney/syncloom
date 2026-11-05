//! Capture selection and non-approval previews. Retrieval and drafting have separate owners.
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<crate::Source>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureConfirmRequest {
    pub text: String,
    pub channels: Vec<String>,
    pub expected_revision: String,
    #[serde(default)]
    pub source_revision: Option<String>,
}

#[derive(Serialize)]
pub struct UrlCapture {
    pub source_revision: String,
    pub source: crate::Source,
    pub suggestions: Vec<Suggestion>,
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
    /// A retrieved snapshot uses this existing capture record with no selected channels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<crate::Source>,
}

impl CaptureRecord {
    pub(crate) fn into_source(self, revision: &str) -> Result<crate::Source, StudioError> {
        let source = self.source.ok_or(StudioError::RevisionConflict)?;
        if self.revision != revision
            || !self.channels.is_empty()
            || source_revision(&source)? != revision
            || !matches!(
                (source.kind.as_str(), &source.retrieval, &source.attachment),
                ("url", Some(_), None) | ("attachment", None, Some(_))
            )
        {
            return Err(StudioError::RevisionConflict);
        }
        Ok(source)
    }
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
        source_revision: None,
        source: None,
    })
}

pub(crate) fn source_revision(source: &crate::Source) -> Result<String, StudioError> {
    let bytes = serde_json::to_vec(source).map_err(|_| StudioError::Internal)?;
    Ok(crate::visual::digest(&bytes))
}

pub(crate) fn preview_source(
    source: &crate::Source,
    channels: &[String],
) -> Result<Preview, StudioError> {
    if source.retrieval.is_none() && source.attachment.is_none() {
        return preview(&source.raw_input, channels);
    }
    let mut preview = preview(source.text(), channels)?;
    let revision = source_revision(source)?;
    preview.revision = crate::visual::digest(
        format!("{}-capture-v1:{revision}:{}", source.kind, preview.revision).as_bytes(),
    );
    preview.source_revision = Some(revision);
    preview.source = Some(source.clone());
    Ok(preview)
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
