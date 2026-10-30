//! syncloom-core: the rules of the Syncloom studio, shared by the HTTP API and the CLI.
//!
//! Behavior is owned by the OpenSpec capabilities `thought-drafting`, `platform-rules`,
//! `draft-review`, `dispatch-outbox` and `provenance-log` under `openspec/specs/`.
//! Rules are plain functions over plain values; file access lives in [`store`] and network
//! access in [`provider::claude`].
pub mod channel;
pub mod clock;
pub mod config;
pub mod error;
pub mod outbox;
pub mod piece;
pub mod provenance;
pub mod provider;
pub mod store;
pub mod studio;

pub use channel::{Bounds, Channel, Fit, Verdict, fit};
pub use clock::{Clock, FixedClock, SystemClock};
pub use config::{StudioConfig, build_studio, data_dir, studio_config, usable_key};
pub use error::StudioError;
pub use outbox::{DispatchIntent, Payload, request_id};
pub use piece::{Piece, Source, Status, piece_id, review_decision_id, revision};
pub use provenance::{Event, EventKind, Usage};
pub use provider::{Draft, DraftFuture, DraftRequest, ModelProvider, ProviderError};
pub use store::Store;
pub use studio::{
    MIN_THOUGHT_CHARS, PublishRequest, PublishResponse, ReviewRequest, Settings, Studio,
    ThoughtRequest, ThoughtResponse, normalize_thought,
};
