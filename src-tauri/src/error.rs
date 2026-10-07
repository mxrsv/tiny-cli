use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}
impl ErrorPayload {
    pub fn new(code: &str, message: impl ToString) -> Self {
        Self {
            code: code.into(),
            message: message.to_string(),
        }
    }
}
impl std::fmt::Display for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for ErrorPayload {}
impl From<tiny_core::error::Error> for ErrorPayload {
    fn from(error: tiny_core::error::Error) -> Self {
        let code = match error {
            tiny_core::error::Error::InvalidInput(_) => "invalid_input",
            tiny_core::error::Error::Unsupported(_) => "unsupported",
            _ => "engine_error",
        };
        Self::new(code, error)
    }
}
impl From<std::io::Error> for ErrorPayload {
    fn from(e: std::io::Error) -> Self {
        Self::new("filesystem_error", e)
    }
}
impl From<rusqlite::Error> for ErrorPayload {
    fn from(e: rusqlite::Error) -> Self {
        Self::new("database_error", e)
    }
}
impl From<serde_json::Error> for ErrorPayload {
    fn from(e: serde_json::Error) -> Self {
        Self::new("data_error", e)
    }
}
pub type Result<T> = std::result::Result<T, ErrorPayload>;
