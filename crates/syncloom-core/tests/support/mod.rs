//! Shared test support for the core crate.
#![allow(dead_code)]
pub mod fake_claude;
pub mod scripted;

use std::sync::Arc;
use syncloom_core::{FixedClock, ModelProvider, Settings, Store, Studio};
use tempfile::TempDir;
use time::macros::datetime;

#[allow(unused_imports)]
pub use scripted::{Reply, Scripted, text};

/// 2026-10-30T18:00:00Z, the fixed instant of every test.
pub fn fixed_clock() -> Arc<FixedClock> {
    Arc::new(FixedClock(datetime!(2026-10-30 18:00:00 UTC)))
}

pub const FIXED_AT: &str = "2026-10-30T18:00:00Z";

/// A studio over a fresh temporary directory.
pub fn studio(dir: &TempDir, provider: Arc<dyn ModelProvider>, settings: Settings) -> Studio {
    Studio::new(
        Store::open(dir.path()).unwrap(),
        provider,
        fixed_clock(),
        settings,
    )
}

pub fn fast() -> Settings {
    Settings {
        provider_deadline: std::time::Duration::from_millis(200),
    }
}

/// A thought with a canary word that must never reach logs or events.
pub const CANARY: &str = "zebrafish";
pub const THOUGHT: &str =
    "  Agents write better when the  spec names one owner per rule; zebrafish.  ";
pub const NORMALIZED: &str =
    "Agents write better when the spec names one owner per rule; zebrafish.";

pub fn events(dir: &TempDir) -> Vec<serde_json::Value> {
    let path = dir.path().join("provenance").join("events.jsonl");
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub fn files(dir: &TempDir, sub: &str) -> Vec<String> {
    let path = dir.path().join(sub);
    if !path.exists() {
        return Vec::new();
    }
    let mut names: Vec<String> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
