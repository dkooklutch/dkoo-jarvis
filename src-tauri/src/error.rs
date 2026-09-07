use thiserror::Error;

#[derive(Debug, Error)]
pub enum JarvisError {
    #[error("security boundary rejected the operation: {0}")]
    Security(String),
    #[error("JARVIS is locked")]
    Locked,
    #[error("project not authorized")]
    UnauthorizedProject,
    #[error("credential is not configured")]
    CredentialMissing,
    #[error("provider error: {0}")]
    Provider(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("operation failed: {0}")]
    Operation(String),
}

impl serde::Serialize for JarvisError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for JarvisError {
    fn from(value: std::io::Error) -> Self {
        Self::Operation(value.to_string())
    }
}
impl From<rusqlite::Error> for JarvisError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Storage(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, JarvisError>;
