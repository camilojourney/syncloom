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
const ROWS: [Row; 11] = [
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
    row("internal_error", "The studio hit an unexpected error."),
];

impl StudioError {
    pub const ALL: [StudioError; 11] = [
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
