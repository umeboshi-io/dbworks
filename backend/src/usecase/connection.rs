use uuid::Uuid;

use crate::domain::connection::ConnectionInfo;
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;

use super::UsecaseError;
use super::error::require_super_admin;

pub async fn create_connection(
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    name: String,
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
) -> Result<ConnectionInfo, UsecaseError> {
    // Determine ownership: org user → org connection (requires super_admin), no org → personal
    let (organization_id, owner_user_id) = if let Some(org_id) = caller.organization_id {
        require_super_admin(caller)?;
        (Some(org_id), None)
    } else {
        (None, Some(caller.id))
    };

    connection_manager
        .add_postgres(
            name,
            host,
            port,
            database,
            user,
            password,
            organization_id,
            owner_user_id,
        )
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

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

pub async fn delete_connection(
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
    if connection_manager.remove(conn_id).await {
        Ok(())
    } else {
        Err(UsecaseError::NotFound("Connection not found".to_string()))
    }
}
