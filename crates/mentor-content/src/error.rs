use std::path::Path;

/// Content error: the message is prefixed with the offending file.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{path} : {message}")]
pub struct ContentError {
    pub path: String,
    pub message: String,
}

impl ContentError {
    pub fn new(path: &Path, message: impl Into<String>) -> Self {
        Self { path: path.display().to_string(), message: message.into() }
    }
}

pub type Result<T> = std::result::Result<T, ContentError>;
