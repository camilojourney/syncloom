//! Manual handoff is not Platform publishing. Fixed filenames and target-specific notes only.
use crate::{Piece, StudioError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRequest {
    pub expected_revision: String,
    pub target: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportResponse {
    pub status: String,
    pub request_id: String,
    pub bundle: String,
    pub created: bool,
}
#[derive(Clone, Copy)]
pub(crate) enum Target {
    Reddit,
    Tiktok,
}
impl Target {
    pub fn parse(raw: &str) -> Result<Self, StudioError> {
        match raw {
            "reddit" => Ok(Self::Reddit),
            "tiktok" => Ok(Self::Tiktok),
            _ => Err(StudioError::InvalidRequest),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Reddit => "reddit",
            Self::Tiktok => "tiktok",
        }
    }
    fn notes(self) -> &'static str {
        match self {
            Self::Reddit => {
                "# Reddit manual handoff\n\nNot posted. Paste text.txt only after checking the target subreddit's rules, relevance, title and self-promotion policy. Select image.png manually if allowed. carousel.pdf is a local reference, not a Reddit carousel upload. Copy alt-text.txt when an image description is supported. Do not apply for API access through this tool.\n"
            }
            Self::Tiktok => {
                "# TikTok later handoff\n\nNot posted. This bundle is copy and visual source material for a later manual video workflow, not a ready video or Direct Post upload. Check current format, music rights, AI disclosure and account/privacy requirements before using TikTok's own editor. carousel.pdf is a reference, not a TikTok video.\n"
            }
        }
    }
}
pub(crate) fn files(
    piece: &Piece,
    target: Target,
    image: Option<Vec<u8>>,
    carousel: Option<Vec<u8>>,
) -> BTreeMap<String, Vec<u8>> {
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::from([
        ("text.txt".into(), piece.text.as_bytes().to_vec()),
        ("notes.md".into(), target.notes().as_bytes().to_vec()),
    ]);
    if let Some(image) = image {
        files.insert("image.png".into(), image);
        files.insert(
            "alt-text.txt".into(),
            piece
                .visual
                .as_ref()
                .map(|v| v.image_alt_text())
                .unwrap_or("")
                .as_bytes()
                .to_vec(),
        );
    }
    if let Some(carousel) = carousel {
        files.insert("carousel.pdf".into(), carousel);
    }
    let hashes: BTreeMap<_, _> = files
        .iter()
        .map(|(name, bytes)| (name.clone(), crate::visual::digest(bytes)))
        .collect();
    let manifest = serde_json::json!({"schema_version":"1.0","piece_id":piece.piece_id,"revision":piece.revision,"target":target.name(),"status":"exported","files":hashes});
    files.insert(
        "manifest.json".into(),
        serde_json::to_vec_pretty(&manifest).expect("manifest serializes"),
    );
    files
}
