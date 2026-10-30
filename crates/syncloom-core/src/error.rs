//! The one table of error codes and fixed messages the API and the CLI share. Messages never
//! carry request content. HTTP statuses are the server's; `contracts/studio-api/errors/`
//! holds the same codes and messages and a test proves they agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn code(self) -> &'static str {
        ""
    }

    pub fn message(self) -> &'static str {
        ""
    }
}

impl std::fmt::Display for StudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}
impl std::error::Error for StudioError {}

impl From<std::io::Error> for StudioError {
    fn from(error: std::io::Error) -> Self {
        tracing::error!(kind = ?error.kind(), "storage_failed");
        StudioError::Internal
    }
}
