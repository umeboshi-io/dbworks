use std::fmt;

use uuid::Uuid;

use crate::domain::repository::{ConnectionRepository, OrganizationMemberRepository};

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

/// Check that the caller is an `owner` of the given organization.
pub(crate) async fn require_org_owner(
    org_member_repo: &dyn OrganizationMemberRepository,
    user_id: &Uuid,
    org_id: &Uuid,
) -> Result<(), UsecaseError> {
    let role = org_member_repo
        .get_role(org_id, user_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))?;
    match role.as_deref() {
        Some("owner") => Ok(()),
        _ => Err(UsecaseError::Forbidden(
            "Organization owner role required".to_string(),
        )),
    }
}

/// Check that the caller can administer a connection.
/// For org connections: caller must be `owner` of the org.
/// For personal connections: caller must be the `owner_user_id`.
pub(crate) async fn require_conn_owner(
    org_member_repo: &dyn OrganizationMemberRepository,
    conn_repo: &dyn ConnectionRepository,
    caller_id: &Uuid,
    conn_id: &Uuid,
) -> Result<(), UsecaseError> {
    let ownership = conn_repo
        .get_ownership(conn_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))?;

    match ownership {
        None => Err(UsecaseError::NotFound("Connection not found".to_string())),
        Some((Some(org_id), _)) => {
            // Org connection: require org owner
            require_org_owner(org_member_repo, caller_id, &org_id).await
        }
        Some((None, Some(owner_id))) => {
            // Personal connection: require owner_user_id match
            if &owner_id == caller_id {
                Ok(())
            } else {
                Err(UsecaseError::Forbidden(
                    "Only the connection owner can manage this connection".to_string(),
                ))
            }
        }
        Some((None, None)) => Err(UsecaseError::Forbidden(
            "Connection has no ownership info".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_variants() {
        assert_eq!(UsecaseError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(UsecaseError::Forbidden("nope".into()).to_string(), "nope");
        assert_eq!(UsecaseError::NotFound("gone".into()).to_string(), "gone");
        assert_eq!(UsecaseError::BadRequest("bad".into()).to_string(), "bad");
        assert_eq!(UsecaseError::Internal("err".into()).to_string(), "err");
    }
}
