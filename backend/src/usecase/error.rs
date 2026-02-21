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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::AppUser;
    use uuid::Uuid;

    fn make_user(role: &str) -> AppUser {
        AppUser {
            id: Uuid::new_v4(),
            organization_id: None,
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
            role: role.to_string(),
            auth_provider: None,
            provider_id: None,
            avatar_url: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn require_super_admin_passes() {
        let user = make_user("super_admin");
        assert!(require_super_admin(&user).is_ok());
    }

    #[test]
    fn require_super_admin_rejects_member() {
        let user = make_user("member");
        let err = require_super_admin(&user).unwrap_err();
        assert!(matches!(err, UsecaseError::Forbidden(_)));
    }

    #[test]
    fn require_super_admin_rejects_admin() {
        let user = make_user("admin");
        assert!(matches!(
            require_super_admin(&user).unwrap_err(),
            UsecaseError::Forbidden(_)
        ));
    }

    #[test]
    fn require_super_admin_rejects_empty() {
        let user = make_user("");
        assert!(matches!(
            require_super_admin(&user).unwrap_err(),
            UsecaseError::Forbidden(_)
        ));
    }

    #[test]
    fn display_variants() {
        assert_eq!(UsecaseError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(UsecaseError::Forbidden("nope".into()).to_string(), "nope");
        assert_eq!(UsecaseError::NotFound("gone".into()).to_string(), "gone");
        assert_eq!(UsecaseError::BadRequest("bad".into()).to_string(), "bad");
        assert_eq!(UsecaseError::Internal("err".into()).to_string(), "err");
    }
}
