//! The file store: per-record JSON files and an append-only event log under one directory,
//! written atomically under one lock file.
//!
//! Layout: `pieces/<piece_id>.json`, `outbox/<key>.json`, `provenance/events.jsonl`, `.lock`.
use crate::outbox::DispatchIntent;
use crate::piece::Piece;
use crate::provenance::Event;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

pub struct Store {
    root: PathBuf,
}

/// The exclusive lock on a store; every read and write goes through it.
pub struct Lock<'a> {
    store: &'a Store,
    _file: File,
}

impl Store {
    /// Open or create the store directories.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Store> {
        Ok(Store { root: root.into() })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Take the store's exclusive lock (cross-process, via the lock file).
    pub fn lock(&self) -> io::Result<Lock<'_>> {
        Err(io::Error::other("not implemented"))
    }
}

impl Lock<'_> {
    pub fn load_piece(&self, piece_id: &str) -> io::Result<Option<Piece>> {
        let _ = (piece_id, self.store);
        Ok(None)
    }
    pub fn save_piece(&self, piece: &Piece) -> io::Result<()> {
        let _ = piece;
        Ok(())
    }
    pub fn load_intent(&self, key: &str) -> io::Result<Option<DispatchIntent>> {
        let _ = key;
        Ok(None)
    }
    pub fn save_intent(&self, key: &str, intent: &DispatchIntent) -> io::Result<()> {
        let _ = (key, intent);
        Ok(())
    }
    pub fn append_event(&self, event: &Event) -> io::Result<()> {
        let _ = event;
        Ok(())
    }
}
