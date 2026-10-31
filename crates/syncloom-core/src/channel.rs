//! The one owner of channel identities, their platforms and text bounds (`platform-rules`).
use serde::{Deserialize, Serialize};

/// A channel a piece is drafted for. The table of channels is this enum and its methods;
/// no other module carries a channel name or a limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    LinkedinText,
    Mastodon,
    ManualExport,
}

/// Inclusive character bounds a draft is checked against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub min_chars: usize,
    pub max_chars: usize,
}

impl Channel {
    /// The exact channel name, or `None` for anything not in the table.
    pub fn parse(name: &str) -> Option<Channel> {
        match name {
            "linkedin_text" => Some(Channel::LinkedinText),
            "mastodon" => Some(Channel::Mastodon),
            "manual_export" => Some(Channel::ManualExport),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Channel::LinkedinText => "linkedin_text",
            Channel::Mastodon => "mastodon",
            Channel::ManualExport => "manual_export",
        }
    }
    /// The platform the channel posts to.
    pub fn platform(self) -> &'static str {
        match self {
            Channel::LinkedinText => "linkedin",
            Channel::Mastodon => "mastodon",
            Channel::ManualExport => "manual",
        }
    }
    /// Retained from the old pipeline for LinkedIn text: 20 to 3000 characters.
    pub fn bounds(self) -> Bounds {
        match self {
            Channel::LinkedinText => Bounds {
                min_chars: 20,
                max_chars: 3000,
            },
            Channel::Mastodon => Bounds {
                min_chars: 1,
                max_chars: 500,
            },
            Channel::ManualExport => Bounds {
                min_chars: 1,
                max_chars: 40000,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "REVIEW")]
    Review,
}

/// How a draft fits its channel. Characters are Unicode scalar values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fit {
    pub verdict: Verdict,
    pub char_count: usize,
    pub min_chars: usize,
    pub max_chars: usize,
}

/// `PASS` inside the channel's bounds (inclusive), `REVIEW` outside. Never truncates.
pub fn fit(channel: Channel, text: &str) -> Fit {
    let Bounds {
        min_chars,
        max_chars,
    } = channel.bounds();
    let char_count = text.chars().count();
    let verdict = if (min_chars..=max_chars).contains(&char_count) {
        Verdict::Pass
    } else {
        Verdict::Review
    };
    Fit {
        verdict,
        char_count,
        min_chars,
        max_chars,
    }
}
