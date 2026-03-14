use thiserror::Error;

#[derive(Error, Debug)]
pub enum MnemeBrainError {
    /// Server returned 4xx/5xx. Contains status code and response body text.
    #[error("HTTP error (status {status}): {message}")]
    Http { status: u16, message: String },

    /// Network/transport error (DNS, connection, timeout). Wraps reqwest::Error.
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Response body could not be deserialized. Wraps serde_json::Error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Catch-all for errors that don't fit other variants.
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MnemeBrainError>;
