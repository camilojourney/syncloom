//! The one table of error codes and fixed messages the API and the CLI share (`ROWS`, indexed
//! by variant). Messages never carry request content. HTTP statuses are the server's;
//! `contracts/studio-api/errors/` holds the same codes and messages and a test proves they agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum StudioError {
    InvalidRequest,
    ThoughtTooShort,
    UnsupportedChannel,
    PieceNotFound,
    InvalidStatusTransition,
    RevisionConflict,
    ApprovalRequired,
    ProviderNotConfigured,
    ProviderError,
    ProviderTimeout,
    LinkedinNotConfigured,
    SignInInProgress,
    SignInNotStarted,
    SignInDenied,
    SignInStateMismatch,
    SignInTimedOut,
    SignInRejected,
    RendererMissing,
    RendererVersionMismatch,
    RendererFontMismatch,
    RendererTimeout,
    RendererFailed,
    AssetsRequired,
    AssetIntegrity,
    AssetConflict,
    MastodonNotConfigured,
    MetaNotConfigured,
    PlatformNotConfigured,
    CostConfirmationRequired,
    Internal,
}

/// One row per error, in the order of the `StudioError` variants.
struct Row {
    code: &'static str,
    message: &'static str,
}
const fn row(code: &'static str, message: &'static str) -> Row {
    Row { code, message }
}
const ROWS: [Row; 30] = [
    row(
        "invalid_request",
        "The request body is not valid for this operation.",
    ),
    row(
        "thought_too_short",
        "The thought needs at least 8 characters that are not spaces.",
    ),
    row(
        "unsupported_channel",
        "One of the requested channels is not supported.",
    ),
    row("piece_not_found", "No piece exists with this id."),
    row(
        "invalid_status_transition",
        "The piece cannot move from its current status to the requested one.",
    ),
    row(
        "revision_conflict",
        "The expected revision does not match the piece's current revision.",
    ),
    row(
        "approval_required",
        "The piece must be approved at this revision before it can be published.",
    ),
    row(
        "provider_not_configured",
        "The model provider is not configured. Set ANTHROPIC_API_KEY for the studio.",
    ),
    row(
        "provider_error",
        "The model provider could not return a draft. Try again.",
    ),
    row(
        "provider_timeout",
        "The model provider took too long. Try again.",
    ),
    row(
        "linkedin_not_configured",
        "LinkedIn sign-in needs SYNCLOOM_LINKEDIN_CLIENT_ID, SYNCLOOM_LINKEDIN_CLIENT_SECRET and a free port at SYNCLOOM_LINKEDIN_REDIRECT_URL.",
    ),
    row(
        "sign_in_in_progress",
        "A platform sign-in is already waiting for the browser. Finish it or wait for it to expire.",
    ),
    row(
        "sign_in_not_started",
        "No platform sign-in is waiting. Start one first.",
    ),
    row(
        "sign_in_denied",
        "The platform did not grant consent for this sign-in.",
    ),
    row(
        "sign_in_state_mismatch",
        "The sign-in callback did not match the sign-in that was started. Nothing was exchanged.",
    ),
    row(
        "sign_in_timed_out",
        "The browser did not return in time. Start the sign-in again.",
    ),
    row(
        "sign_in_rejected",
        "The platform refused to issue a token for this sign-in. Check the app's client id, secret and redirect URL.",
    ),
    row(
        "renderer_missing",
        "The configured renderer binary is missing.",
    ),
    row(
        "renderer_version_mismatch",
        "The renderer does not match the configured version.",
    ),
    row(
        "renderer_font_mismatch",
        "The renderer font does not match the configured checksum.",
    ),
    row("renderer_timeout", "The renderer took too long."),
    row(
        "renderer_failed",
        "The renderer failed to produce visual assets.",
    ),
    row(
        "assets_required",
        "Current rendered assets and their exact hashes are required for this operation.",
    ),
    row(
        "asset_integrity",
        "The visual asset bytes do not match their recorded hashes.",
    ),
    row(
        "asset_conflict",
        "The expected asset hashes do not match the piece\'s current rendered assets.",
    ),
    row(
        "mastodon_not_configured",
        "Mastodon needs an explicit SYNCLOOM_MASTODON_INSTANCE and a free loopback callback port.",
    ),
    row(
        "meta_not_configured",
        "Meta needs a client id, client secret, business login configuration, Page id and a free loopback callback port.",
    ),
    row(
        "platform_not_configured",
        "This platform needs its documented app settings before sign-in.",
    ),
    row(
        "cost_confirmation_required",
        "Preview the X cost estimate and confirm its exact micro-USD amount before publishing.",
    ),
    row("internal_error", "The studio hit an unexpected error."),
];

impl StudioError {
    pub const ALL: [StudioError; 30] = [
        StudioError::InvalidRequest,
        StudioError::ThoughtTooShort,
        StudioError::UnsupportedChannel,
        StudioError::PieceNotFound,
        StudioError::InvalidStatusTransition,
        StudioError::RevisionConflict,
        StudioError::ApprovalRequired,
        StudioError::ProviderNotConfigured,
        StudioError::ProviderError,
        StudioError::ProviderTimeout,
        StudioError::LinkedinNotConfigured,
        StudioError::SignInInProgress,
        StudioError::SignInNotStarted,
        StudioError::SignInDenied,
        StudioError::SignInStateMismatch,
        StudioError::SignInTimedOut,
        StudioError::SignInRejected,
        StudioError::RendererMissing,
        StudioError::RendererVersionMismatch,
        StudioError::RendererFontMismatch,
        StudioError::RendererTimeout,
        StudioError::RendererFailed,
        StudioError::AssetsRequired,
        StudioError::AssetIntegrity,
        StudioError::AssetConflict,
        StudioError::MastodonNotConfigured,
        StudioError::MetaNotConfigured,
        StudioError::PlatformNotConfigured,
        StudioError::CostConfirmationRequired,
        StudioError::Internal,
    ];

    fn row(self) -> &'static Row {
        &ROWS[self as usize]
    }
    pub fn code(self) -> &'static str {
        self.row().code
    }
    pub fn message(self) -> &'static str {
        self.row().message
    }
}

impl std::fmt::Display for StudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}
impl std::error::Error for StudioError {}

/// A storage failure is internal; its kind is logged, never its path or content.
impl From<std::io::Error> for StudioError {
    fn from(error: std::io::Error) -> Self {
        tracing::error!(kind = ?error.kind(), "storage_failed");
        StudioError::Internal
    }
}

/// A credential store failure is internal; its kind is logged, never a secret.
impl From<crate::credentials::StoreError> for StudioError {
    fn from(error: crate::credentials::StoreError) -> Self {
        tracing::error!(kind = %error.0, "credential_store_failed");
        StudioError::Internal
    }
}
