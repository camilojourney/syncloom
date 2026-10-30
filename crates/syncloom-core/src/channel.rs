//! The one owner of channel identities, their platforms and text bounds (`platform-rules`).
use serde::{Deserialize, Serialize};

/// A channel a piece is drafted for. The table of channels is this enum and its methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    LinkedinText,
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
        let _ = name;
        None
    }
    pub fn name(self) -> &'static str {
        match self {
            Channel::LinkedinText => "linkedin_text",
        }
    }
    /// The platform the channel posts to.
    pub fn platform(self) -> &'static str {
        match self {
            Channel::LinkedinText => "linkedin",
        }
    }
    pub fn bounds(self) -> Bounds {
        match self {
            Channel::LinkedinText => Bounds {
                min_chars: 0,
                max_chars: 0,
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

pub fn fit(channel: Channel, text: &str) -> Fit {
    let _ = (channel, text);
    Fit {
        verdict: Verdict::Review,
        char_count: 0,
        min_chars: 0,
        max_chars: 0,
    }
}
