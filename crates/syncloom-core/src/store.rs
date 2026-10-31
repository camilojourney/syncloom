//! The file store: per-record JSON files and an append-only event log under one directory,
//! written atomically under one lock file.
//!
//! Layout: `pieces/<piece_id>.json`, `outbox/<key>.json`, `provenance/events.jsonl`, `.lock`.
//! Invariants: a record file is replaced by rename, so a reader sees the old or the new
//! record and never a torn one; temporary files end in `.tmp` and are never read; every
//! read and write happens while the caller holds the [`Lock`], which is a cross-process
//! advisory lock on `.lock` so the CLI and the server may share a directory.
use crate::outbox::DispatchIntent;
use crate::piece::Piece;
use crate::provenance::Event;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

const PIECES: &str = "pieces";
const OUTBOX: &str = "outbox";
const PROVENANCE: &str = "provenance";
const EVENTS: &str = "events.jsonl";
const LOCK: &str = ".lock";

pub struct Store {
    root: PathBuf,
}

/// The exclusive lock on a store; every read and write goes through it. Dropping it unlocks.
pub struct Lock<'a> {
    store: &'a Store,
    file: File,
}

impl Store {
    /// Open or create the store directories.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Store> {
        let root = root.into();
        for sub in [PIECES, OUTBOX, PROVENANCE, "assets", "exports"] {
            fs::create_dir_all(root.join(sub))?;
            if fs::symlink_metadata(root.join(sub))?
                .file_type()
                .is_symlink()
            {
                return Err(io::Error::other("store directory is a symlink"));
            }
        }
        Ok(Store { root })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Take the store's exclusive lock, waiting for another holder to release it.
    pub fn lock(&self) -> io::Result<Lock<'_>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.root.join(LOCK))?;
        file.lock()?;
        Ok(Lock { store: self, file })
    }
}

impl Drop for Lock<'_> {
    fn drop(&mut self) {
        // Closing the file releases the lock too; unlocking first keeps the intent explicit.
        let _ = self.file.unlock();
    }
}

impl Lock<'_> {
    fn piece_path(&self, piece_id: &str) -> PathBuf {
        self.store
            .root
            .join(PIECES)
            .join(format!("{piece_id}.json"))
    }
    fn intent_path(&self, key: &str) -> PathBuf {
        self.store.root.join(OUTBOX).join(format!("{key}.json"))
    }

    pub fn load_piece(&self, piece_id: &str) -> io::Result<Option<Piece>> {
        read_json(&self.piece_path(piece_id))
    }
    /// Keep only the smallest requested names, so discovery never accumulates all records.
    pub fn pieces_after(&self, after: Option<&str>, limit: usize) -> io::Result<Vec<Piece>> {
        let mut names = std::collections::BTreeSet::new();
        for entry in fs::read_dir(self.store.root.join(PIECES))? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".json")) else {
                continue;
            };
            if after.is_some_and(|after| id <= after) {
                continue;
            }
            if !entry.file_type()?.is_file() {
                return Err(io::Error::other("piece record is not a regular file"));
            }
            names.insert(id.to_owned());
            if names.len() > limit {
                names.pop_last();
            }
        }
        names.into_iter().map(|id| self.named_piece(&id)).collect()
    }

    /// Discovery never trusts a record whose stored identity disagrees with its file name.
    fn named_piece(&self, id: &str) -> io::Result<Piece> {
        let piece = self
            .load_piece(id)?
            .ok_or_else(|| io::Error::other("piece disappeared"))?;
        if piece.piece_id != id {
            return Err(io::Error::other("piece id disagrees with record"));
        }
        Ok(piece)
    }

    pub fn save_piece(&self, piece: &Piece) -> io::Result<()> {
        write_json(&self.piece_path(&piece.piece_id), piece)
    }
    pub fn load_intent(&self, key: &str) -> io::Result<Option<DispatchIntent>> {
        read_json(&self.intent_path(key))
    }
    pub fn save_intent(&self, key: &str, intent: &DispatchIntent) -> io::Result<()> {
        write_json(&self.intent_path(key), intent)
    }
    /// Content-addressed assets share the store lock and atomic-write policy.
    pub fn save_asset(&self, bytes: &[u8]) -> io::Result<String> {
        let hash = crate::visual::digest(bytes);
        atomic_write(&self.store.root.join("assets").join(&hash), bytes)?;
        Ok(hash)
    }

    pub fn load_asset(&self, hash: &str) -> Result<Vec<u8>, crate::StudioError> {
        use crate::StudioError;
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(StudioError::AssetIntegrity);
        }
        let path = self.store.root.join("assets").join(hash);
        let meta = fs::symlink_metadata(&path).map_err(|_| StudioError::AssetIntegrity)?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(StudioError::AssetIntegrity);
        }
        let bytes = fs::read(path).map_err(|_| StudioError::AssetIntegrity)?;
        if crate::visual::digest(&bytes) != hash {
            return Err(StudioError::AssetIntegrity);
        }
        Ok(bytes)
    }

    /// A complete manifest is the bundle commit marker; incomplete writes can be retried.
    /// The intent is recorded only after this returns. Existing complete bundles are verified,
    /// never silently repaired, and every destination component is checked before writing.
    pub(crate) fn save_export(
        &self,
        key: &str,
        files: &std::collections::BTreeMap<String, Vec<u8>>,
        recorded: bool,
    ) -> io::Result<PathBuf> {
        if key.len() != 64 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(io::Error::other("invalid export key"));
        }
        let root = self.store.root.join("exports");
        checked_directory(&root)?;
        let bundle = root.join(key);
        match fs::symlink_metadata(&bundle) {
            Ok(meta) if !meta.is_dir() || meta.file_type().is_symlink() => {
                return Err(io::Error::other("invalid export directory"));
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound && !recorded => {
                fs::create_dir(&bundle)?;
            }
            Err(e) => return Err(e),
        }
        let complete = recorded || bundle.join("manifest.json").symlink_metadata().is_ok();
        for (name, bytes) in files
            .iter()
            .filter(|(n, _)| n.as_str() != "manifest.json")
            .chain(files.iter().filter(|(n, _)| n.as_str() == "manifest.json"))
        {
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
                || name.starts_with('.')
            {
                return Err(io::Error::other("invalid export filename"));
            }
            let path = bundle.join(name);
            match fs::symlink_metadata(&path) {
                Ok(meta) if !meta.is_file() || meta.file_type().is_symlink() => {
                    return Err(io::Error::other("invalid export file"));
                }
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound && !complete => {}
                Err(e) => return Err(e),
            }
            if complete {
                if fs::read(&path)? != *bytes {
                    return Err(io::Error::other("export integrity failed"));
                }
            } else {
                atomic_write(&path, bytes)?;
            }
        }
        File::open(&root)?.sync_all()?;
        Ok(bundle)
    }

    /// Append one event line and flush it to disk. Earlier lines are never touched.
    pub fn append_event(&self, event: &Event) -> io::Result<()> {
        let mut line = serde_json::to_vec(event).map_err(io::Error::other)?;
        line.push(b'\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.store.root.join(PROVENANCE).join(EVENTS))?;
        file.write_all(&line)?;
        file.sync_all()
    }
}

fn checked_directory(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_dir() && !meta.file_type().is_symlink() {
        Ok(())
    } else {
        Err(io::Error::other("invalid store directory"))
    }
}

/// `None` when the file does not exist; a corrupt file is an error, never a silent `None`.
fn read_json<T: DeserializeOwned>(path: &Path) -> io::Result<Option<T>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    atomic_write(path, &bytes)
}

/// Write to a temporary file beside the target, sync it, rename it over the target and sync
/// the directory. A crash at any point leaves the old record or the new one.
fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let (directory, name) = split_record_path(path)?;
    let temporary = directory.join(format!(".{name}.{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = write_synced(&temporary, bytes).and_then(|()| {
        fs::rename(&temporary, path)?;
        File::open(directory)?.sync_all()
    });
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn split_record_path(path: &Path) -> io::Result<(&Path, &str)> {
    let directory = path
        .parent()
        .ok_or_else(|| io::Error::other("record path has no directory"))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::other("record path has no file name"))?;
    Ok((directory, name))
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
