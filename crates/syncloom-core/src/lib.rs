//! syncloom-core: the rules of the Syncloom studio, shared by the HTTP API and the CLI.
//!
//! Behavior is owned by the OpenSpec capabilities `thought-drafting`, `platform-rules`,
//! `draft-review`, `dispatch-outbox`, `platform-credentials`, `linkedin-integration` and
//! `provenance-log` under `openspec/specs/`; active integration deltas live under
//! `openspec/changes/` until archive. Rules are plain functions over plain values;
//! file access lives in [`store`], the Keychain in [`credentials`], and network access in
//! [`provider::claude`], [`linkedin`], [`mastodon`], [`meta`] and the [`social`] adapters.
pub mod attachment;
mod bluesky;
mod bluesky_transport;
pub mod capture;
pub mod channel;
pub mod clock;
pub mod config;
mod credential_session;
pub mod credentials;
pub mod error;
pub mod html;
pub mod linkedin;
pub mod manual_export;
pub mod mastodon;
pub mod meta;
pub mod oauth;
mod oauth_context;
pub mod outbox;
pub mod piece;
pub mod platform;
pub mod provenance;
pub mod provider;
pub mod routing;
pub mod social;
mod social_config;
pub mod source;
pub mod store;
pub mod studio;
mod threads;
mod threads_publish;
pub mod visual;
pub mod visual_quality;
mod x;
mod x_media;
pub use visual::{
    AssetHashes, AssetKind, Assets, RenderRequest, RendererConfig, VisualContent, renderer_config,
};

pub use capture::{
    CaptureConfirmRequest, CaptureConfirmResponse, Preview, Suggestion, preview, suggest,
};
pub use channel::{Bounds, Channel, Fit, Verdict, fit};
pub use clock::{Clock, FixedClock, SystemClock};
pub use config::{
    BaseUrl, StudioConfig, build_studio, data_dir, delivery_grant, keychain_service, studio_config,
    usable_key,
};
pub use credentials::{KeychainStore, Secret, StoreError, Token, TokenStore};
pub use error::StudioError;
pub use linkedin::{
    AuthFlow, Connection, ConnectionStatus, Linkedin, LinkedinConfig, RedirectUrl, SignInStart,
    linkedin_config, post_url, redirect_url,
};
pub use outbox::{DispatchIntent, IntentStatus, Payload, request_id};
pub use piece::{Piece, Source, Status, piece_id, review_decision_id, revision};
pub use platform::{Platform, PostFailure, PostFuture, PostRequest, Posted};
pub use provenance::{Event, EventKind, Usage};
pub use provider::{Draft, DraftFuture, DraftRequest, ModelProvider, ProviderError};
pub use store::Store;
pub use studio::{
    MIN_THOUGHT_CHARS, PiecePage, PublishRequest, PublishResponse, ReviewRequest, Settings, Studio,
    ThoughtRequest, ThoughtResponse, normalize_thought,
};
