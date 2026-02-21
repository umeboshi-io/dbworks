use crate::domain::connection::ConnectionInfo;
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

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
