//! Central error type. Serialised to the frontend as `{ "code": "...", "message": "..." }`.
//! Internal details (SQL text, file paths) are logged, never sent to the UI.

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Permission(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "VALIDATION",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Permission(_) => "PERMISSION_DENIED",
            AppError::Database(_) => "DATABASE",
            AppError::Io(_) => "IO",
            AppError::Internal(_) => "INTERNAL",
        }
    }

    /// Message that is safe to show to a librarian.
    pub fn user_message(&self) -> String {
        match self {
            AppError::Validation(m)
            | AppError::NotFound(m)
            | AppError::Conflict(m)
            | AppError::Permission(m) => m.clone(),
            AppError::Database(_) => "A database error occurred. Your data was not changed.".into(),
            AppError::Io(_) => "A file operation failed. Check disk space and permissions.".into(),
            AppError::Internal(_) => "An unexpected error occurred.".into(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if matches!(
            self,
            AppError::Database(_) | AppError::Io(_) | AppError::Internal(_)
        ) {
            tracing::error!(code = self.code(), detail = %self, "command failed");
        }
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.user_message())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_code_and_message() {
        let e = AppError::Conflict("Accession number already exists.".into());
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "CONFLICT");
        assert_eq!(v["message"], "Accession number already exists.");
    }

    #[test]
    fn internal_details_are_not_exposed() {
        let e = AppError::Database(rusqlite::Error::InvalidQuery);
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "DATABASE");
        let msg = v["message"].as_str().unwrap();
        assert!(!msg.to_lowercase().contains("query"));
    }
}
