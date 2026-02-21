use uuid::Uuid;

use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

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
