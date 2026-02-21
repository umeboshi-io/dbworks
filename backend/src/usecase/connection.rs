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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user(role: &str, org_id: Option<Uuid>) -> AppUser {
        AppUser {
            id: Uuid::new_v4(),
            organization_id: org_id,
            name: "Caller".to_string(),
            email: "caller@test.com".to_string(),
            role: role.to_string(),
            auth_provider: None,
            provider_id: None,
            avatar_url: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[tokio::test]
    async fn list_connections_all() {
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member", None);

        let result = list_connections(&cm, &caller, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_connections_personal() {
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member", None);

        let result = list_connections(&cm, &caller, Some("personal")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_connections_by_org() {
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member", None);
        let org_id = Uuid::new_v4();

        let result = list_connections(&cm, &caller, Some(&format!("org:{}", org_id))).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_connections_invalid_org_scope() {
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member", None);

        let result = list_connections(&cm, &caller, Some("org:invalid-uuid")).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::BadRequest(_)));
    }

    #[tokio::test]
    async fn delete_connection_as_member_forbidden() {
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member", None);

        let result = delete_connection(&cm, &caller, &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn delete_connection_not_found() {
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("super_admin", None);

        let result = delete_connection(&cm, &caller, &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
    }

    #[tokio::test]
    async fn create_connection_org_user_as_member_forbidden() {
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member", Some(Uuid::new_v4()));

        let result = create_connection(
            &cm,
            &caller,
            "test".into(),
            "localhost".into(),
            5432,
            "db".into(),
            "user".into(),
            "pass".into(),
        )
        .await;
        // Org user + member role → Forbidden
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }
}
