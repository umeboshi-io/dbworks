use uuid::Uuid;

use crate::domain::connection::ConnectionInfo;
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;

pub async fn list_connections(
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    scope: Option<&str>,
) -> Result<Vec<ConnectionInfo>, UsecaseError> {
    let connections = match scope {
        Some("personal") => connection_manager.list_personal(&caller.id).await,
        Some(s) if s.starts_with("org:") => {
            let org_id_str = &s[4..];
            let org_id = Uuid::parse_str(org_id_str)
                .map_err(|_| UsecaseError::BadRequest("Invalid org ID in scope".to_string()))?;
            connection_manager.list_by_org(&org_id).await
        }
        _ => connection_manager.list().await,
    };
    Ok(connections)
}
