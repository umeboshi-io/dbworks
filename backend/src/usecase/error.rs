use std::fmt;

/// Domain-level errors returned by usecases.
/// Handlers map these to HTTP status codes.
#[derive(Debug)]
pub enum UsecaseError {
    Unauthorized,
    Forbidden(String),
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl fmt::Display for UsecaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::Forbidden(msg) => write!(f, "{}", msg),
            Self::NotFound(msg) => write!(f, "{}", msg),
            Self::BadRequest(msg) => write!(f, "{}", msg),
            Self::Internal(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for UsecaseError {}

/// Helper: check that the caller has `super_admin` role.
pub(crate) fn require_super_admin(
    caller: &crate::domain::user::AppUser,
) -> Result<(), UsecaseError> {
    if caller.role != "super_admin" {
        return Err(UsecaseError::Forbidden(
            "SuperAdmin role required".to_string(),
        ));
    }
    Ok(())
}
