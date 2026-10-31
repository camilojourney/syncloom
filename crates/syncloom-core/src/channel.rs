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
    FacebookPage,
    Instagram,
    Bluesky,
    X,
    Threads,
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
            "facebook_page" => Some(Channel::FacebookPage),
            "instagram" => Some(Channel::Instagram),
            "bluesky" => Some(Channel::Bluesky),
            "x" => Some(Channel::X),
            "threads" => Some(Channel::Threads),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Channel::LinkedinText => "linkedin_text",
            Channel::Mastodon => "mastodon",
            Channel::ManualExport => "manual_export",
            Channel::FacebookPage => "facebook_page",
            Channel::Instagram => "instagram",
            Channel::Bluesky => "bluesky",
            Channel::X => "x",
            Channel::Threads => "threads",
        }
    }
    /// User-facing platform name; machine identities remain stable in `name`.
    pub fn display_name(self) -> &'static str {
        match self {
            Channel::LinkedinText => "LinkedIn",
            Channel::Mastodon => "Mastodon",
            Channel::ManualExport => "Manual export",
            Channel::FacebookPage => "Facebook",
            Channel::Instagram => "Instagram",
            Channel::Bluesky => "Bluesky",
            Channel::X => "X",
            Channel::Threads => "Threads",
        }
    }
    /// The platform the channel posts to.
    pub fn platform(self) -> &'static str {
        match self {
            Channel::LinkedinText => "linkedin",
            Channel::ManualExport => "manual",
            Channel::FacebookPage => "facebook",
            _ => self.name(),
        }
    }
    pub fn count_unit(self) -> &'static str {
        match self {
            Channel::Bluesky => "graphemes",
            Channel::X => "weighted units",
            Channel::Threads => "UTF-8 bytes",
            _ => "characters",
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
            Channel::FacebookPage => Bounds {
                min_chars: 1,
                max_chars: 63206,
            },
            Channel::Instagram => Bounds {
                min_chars: 1,
                max_chars: 2200,
            },
            Channel::Bluesky => Bounds {
                min_chars: 1,
                max_chars: 300,
            },
            Channel::X => Bounds {
                min_chars: 1,
                max_chars: 280,
            },
            Channel::Threads => Bounds {
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

/// How a draft fits its channel. Counts use the channel-specific conservative unit.
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
    use unicode_segmentation::UnicodeSegmentation;
    let char_count = match channel {
        Channel::Bluesky => text.graphemes(true).count(),
        // Conservative bounds, not a promise that the remote service will accept a post.
        Channel::X => text
            .chars()
            .map(|c| if (c as u32) <= 0x10ff { 1 } else { 2 })
            .sum(),
        Channel::Threads => text.len(),
        _ => text.chars().count(),
    };
    let bytes_fit = channel != Channel::Bluesky || text.len() <= 3000;
    let verdict = if bytes_fit && (min_chars..=max_chars).contains(&char_count) {
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
