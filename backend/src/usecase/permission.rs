use uuid::Uuid;

use crate::domain::permission::*;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;

use super::UsecaseError;
use super::error::require_super_admin;

// ============================================================
// User Connection Permissions
// ============================================================

pub async fn grant_user_connection_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
    permission: &str,
    all_tables: bool,
) -> Result<UserConnectionPermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_user_connection_permission(conn_id, user_id, permission, all_tables)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn revoke_user_connection_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
    let revoked = permission_repo
        .revoke_user_connection_permission(conn_id, user_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))?;
    if revoked {
        Ok(())
    } else {
        Err(UsecaseError::NotFound("Permission not found".to_string()))
    }
}

pub async fn list_user_connection_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
) -> Result<Vec<UserConnectionPermission>, UsecaseError> {
    permission_repo
        .list_user_connection_permissions(conn_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

// ============================================================
// User Table Permissions
// ============================================================

pub async fn grant_user_table_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
    table_name: &str,
    permission: &str,
) -> Result<UserTablePermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_user_table_permission(conn_id, user_id, table_name, permission)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn revoke_user_table_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
    table_name: &str,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
    let revoked = permission_repo
        .revoke_user_table_permission(conn_id, user_id, table_name)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))?;
    if revoked {
        Ok(())
    } else {
        Err(UsecaseError::NotFound("Permission not found".to_string()))
    }
}

pub async fn list_user_table_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
    user_id: &Uuid,
) -> Result<Vec<UserTablePermission>, UsecaseError> {
    permission_repo
        .list_user_table_permissions(conn_id, user_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

// ============================================================
// Group Connection Permissions
// ============================================================

pub async fn grant_group_connection_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    group_id: &Uuid,
    permission: &str,
    all_tables: bool,
) -> Result<GroupConnectionPermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_group_connection_permission(conn_id, group_id, permission, all_tables)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn revoke_group_connection_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    group_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
    let revoked = permission_repo
        .revoke_group_connection_permission(conn_id, group_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))?;
    if revoked {
        Ok(())
    } else {
        Err(UsecaseError::NotFound("Permission not found".to_string()))
    }
}

pub async fn list_group_connection_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
) -> Result<Vec<GroupConnectionPermission>, UsecaseError> {
    permission_repo
        .list_group_connection_permissions(conn_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

// ============================================================
// Group Table Permissions
// ============================================================

pub async fn grant_group_table_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    group_id: &Uuid,
    table_name: &str,
    permission: &str,
) -> Result<GroupTablePermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_group_table_permission(conn_id, group_id, table_name, permission)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn revoke_group_table_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    group_id: &Uuid,
    table_name: &str,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
    let revoked = permission_repo
        .revoke_group_table_permission(conn_id, group_id, table_name)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))?;
    if revoked {
        Ok(())
    } else {
        Err(UsecaseError::NotFound("Permission not found".to_string()))
    }
}

pub async fn list_group_table_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
    group_id: &Uuid,
) -> Result<Vec<GroupTablePermission>, UsecaseError> {
    permission_repo
        .list_group_table_permissions(conn_id, group_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
