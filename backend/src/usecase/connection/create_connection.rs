use uuid::Uuid;

use crate::domain::connection::ConnectionInfo;
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;

#[allow(clippy::too_many_arguments)]
pub async fn create_connection(
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    name: String,
    db_type: String,
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
    scope_org_id: Option<Uuid>,
) -> Result<ConnectionInfo, UsecaseError> {
    // Use explicit scope_org_id if provided, otherwise personal connection
    let (organization_id, owner_user_id) = if let Some(org_id) = scope_org_id {
        (Some(org_id), None)
    } else {
        (None, Some(caller.id))
    };

    let result = match db_type.as_str() {
        "mysql" => {
            connection_manager
                .add_mysql(
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
        }
        "postgres" => {
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
        }
        other => {
            return Err(UsecaseError::BadRequest(format!(
                "Unsupported database type: '{}'. Supported types: postgres, mysql",
                other
            )));
        }
    };

    result.map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
