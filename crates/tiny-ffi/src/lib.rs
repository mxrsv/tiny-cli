//! UniFFI adapter that links `tiny-core` into the native macOS app.
//!
//! Swift sees only the records, objects and errors declared here; `tiny-core`
//! types never cross the boundary directly.

// UniFFI converts unwinding panics into a failed call. With `panic = "abort"`
// a Rust panic would terminate the whole app instead.
#[cfg(not(panic = "unwind"))]
compile_error!("tiny-ffi must be built with panic = \"unwind\"");

uniffi::setup_scaffolding!();

pub mod clean;
pub mod processes;
pub mod session;

pub use session::{CancellationToken, TinySession};

/// Typed adapter error. Cases are stable; `detail` is diagnostic text, while
/// user-facing copy lives in Swift.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("invalid input: {detail}")]
    InvalidInput { detail: String },
    #[error("operation failed: {detail}")]
    Operation { detail: String },
    #[error("unsupported: {detail}")]
    Unsupported { detail: String },
    #[error("i/o error: {detail}")]
    Io { detail: String },
    #[error("another operation is already running")]
    Busy,
    #[error("operation cancelled")]
    Cancelled,
    /// macOS denied Automation (Apple Events) access; never falls back to deletion.
    #[error("automation permission denied: {detail}")]
    AutomationDenied { detail: String },
    /// A cleanup preview is unknown, expired, consumed or no longer matches disk.
    #[error("preview is no longer valid: {detail}")]
    PreviewInvalid { detail: String },
}

impl From<tiny_core::error::Error> for FfiError {
    fn from(error: tiny_core::error::Error) -> Self {
        use tiny_core::error::Error;
        match error {
            Error::InvalidInput(detail) => Self::InvalidInput { detail },
            Error::Operation(detail) => Self::Operation { detail },
            Error::Unsupported(detail) => Self::Unsupported { detail },
            Error::AutomationDenied(detail) => Self::AutomationDenied { detail },
            Error::Io(error) => Self::Io {
                detail: error.to_string(),
            },
            Error::Json(error) => Self::Operation {
                detail: error.to_string(),
            },
        }
    }
}

/// Progress update for the call that started the operation.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiProgress {
    pub operation: String,
    pub completed: u64,
    pub total: Option<u64>,
    pub message: String,
}

impl From<tiny_core::progress::Progress> for FfiProgress {
    fn from(progress: tiny_core::progress::Progress) -> Self {
        Self {
            operation: progress.operation,
            completed: progress.completed,
            total: progress.total,
            message: progress.message,
        }
    }
}

/// Implemented in Swift. Called on a background thread; Swift hops to the
/// main actor before touching UI state.
#[uniffi::export(with_foreign)]
pub trait ProgressListener: Send + Sync {
    fn on_progress(&self, progress: FfiProgress);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_core::error::Error;

    #[test]
    fn core_errors_map_to_typed_cases() {
        let invalid: FfiError = Error::InvalidInput("idleDays must be > 0".into()).into();
        assert!(
            matches!(invalid, FfiError::InvalidInput { detail } if detail == "idleDays must be > 0")
        );

        let io: FfiError =
            Error::Io(std::io::Error::from(std::io::ErrorKind::PermissionDenied)).into();
        assert!(matches!(io, FfiError::Io { .. }));

        let unsupported: FfiError = Error::Unsupported("x".into()).into();
        assert!(matches!(unsupported, FfiError::Unsupported { .. }));
    }

    #[test]
    fn progress_converts_field_by_field() {
        let progress = tiny_core::progress::Progress {
            operation: "scan".into(),
            completed: 2,
            total: Some(4),
            message: "Xcode DerivedData".into(),
        };
        assert_eq!(
            FfiProgress::from(progress),
            FfiProgress {
                operation: "scan".into(),
                completed: 2,
                total: Some(4),
                message: "Xcode DerivedData".into(),
            }
        );
    }
}
