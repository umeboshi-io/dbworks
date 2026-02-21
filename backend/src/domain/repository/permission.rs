use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::permission::*;
use crate::domain::user::AppUser;

#[async_trait]
pub trait PermissionRepository: Send + Sync {
    // User Connection Permissions
    async fn grant_user_connection_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
        permission: &str,
        all_tables: bool,
    ) -> anyhow::Result<UserConnectionPermission>;
    async fn revoke_user_connection_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<bool>;
    async fn list_user_connection_permissions(
        &self,
        conn_id: &Uuid,
    ) -> anyhow::Result<Vec<UserConnectionPermission>>;

    // User Table Permissions
    async fn grant_user_table_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
        table_name: &str,
        permission: &str,
    ) -> anyhow::Result<UserTablePermission>;
    async fn revoke_user_table_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
        table_name: &str,
    ) -> anyhow::Result<bool>;
    async fn list_user_table_permissions(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Vec<UserTablePermission>>;

    // Group Connection Permissions
    async fn grant_group_connection_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
        permission: &str,
        all_tables: bool,
    ) -> anyhow::Result<GroupConnectionPermission>;
    async fn revoke_group_connection_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
    ) -> anyhow::Result<bool>;
    async fn list_group_connection_permissions(
        &self,
        conn_id: &Uuid,
    ) -> anyhow::Result<Vec<GroupConnectionPermission>>;

    // Group Table Permissions
    async fn grant_group_table_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
        table_name: &str,
        permission: &str,
    ) -> anyhow::Result<GroupTablePermission>;
    async fn revoke_group_table_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
        table_name: &str,
    ) -> anyhow::Result<bool>;
    async fn list_group_table_permissions(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
    ) -> anyhow::Result<Vec<GroupTablePermission>>;

    // Permission Resolution
    async fn resolve_connection_permission(
        &self,
        user: &AppUser,
        conn_id: &Uuid,
    ) -> anyhow::Result<(PermissionLevel, bool)>;
    async fn resolve_table_permission(
        &self,
        user: &AppUser,
        conn_id: &Uuid,
        table_name: &str,
    ) -> anyhow::Result<PermissionLevel>;
}
