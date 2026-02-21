use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::permission::*;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;

pub struct PgPermissionRepository {
    pool: PgPool,
}

impl PgPermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepository for PgPermissionRepository {
    // ============================================================
    // User Connection Permissions
    // ============================================================

    async fn grant_user_connection_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
        permission: &str,
        all_tables: bool,
    ) -> anyhow::Result<UserConnectionPermission> {
        let perm = sqlx::query_as::<_, UserConnectionPermission>(
            r#"INSERT INTO user_connection_permissions (user_id, connection_id, permission, all_tables)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (user_id, connection_id) DO UPDATE SET permission = $3, all_tables = $4
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(conn_id)
        .bind(permission)
        .bind(all_tables)
        .fetch_one(&self.pool)
        .await?;
        Ok(perm)
    }

    async fn revoke_user_connection_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM user_connection_permissions WHERE connection_id = $1 AND user_id = $2",
        )
        .bind(conn_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_user_connection_permissions(
        &self,
        conn_id: &Uuid,
    ) -> anyhow::Result<Vec<UserConnectionPermission>> {
        let perms = sqlx::query_as::<_, UserConnectionPermission>(
            "SELECT * FROM user_connection_permissions WHERE connection_id = $1",
        )
        .bind(conn_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(perms)
    }

    // ============================================================
    // User Table Permissions
    // ============================================================

    async fn grant_user_table_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
        table_name: &str,
        permission: &str,
    ) -> anyhow::Result<UserTablePermission> {
        let perm = sqlx::query_as::<_, UserTablePermission>(
            r#"INSERT INTO user_table_permissions (user_id, connection_id, table_name, permission)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (user_id, connection_id, table_name) DO UPDATE SET permission = $4
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(conn_id)
        .bind(table_name)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(perm)
    }

    async fn revoke_user_table_permission(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
        table_name: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM user_table_permissions WHERE connection_id = $1 AND user_id = $2 AND table_name = $3",
        )
        .bind(conn_id)
        .bind(user_id)
        .bind(table_name)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_user_table_permissions(
        &self,
        conn_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Vec<UserTablePermission>> {
        let perms = sqlx::query_as::<_, UserTablePermission>(
            "SELECT * FROM user_table_permissions WHERE connection_id = $1 AND user_id = $2",
        )
        .bind(conn_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(perms)
    }

    // ============================================================
    // Group Connection Permissions
    // ============================================================

    async fn grant_group_connection_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
        permission: &str,
        all_tables: bool,
    ) -> anyhow::Result<GroupConnectionPermission> {
        let perm = sqlx::query_as::<_, GroupConnectionPermission>(
            r#"INSERT INTO group_connection_permissions (group_id, connection_id, permission, all_tables)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (group_id, connection_id) DO UPDATE SET permission = $3, all_tables = $4
               RETURNING *"#,
        )
        .bind(group_id)
        .bind(conn_id)
        .bind(permission)
        .bind(all_tables)
        .fetch_one(&self.pool)
        .await?;
        Ok(perm)
    }

    async fn revoke_group_connection_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM group_connection_permissions WHERE connection_id = $1 AND group_id = $2",
        )
        .bind(conn_id)
        .bind(group_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_group_connection_permissions(
        &self,
        conn_id: &Uuid,
    ) -> anyhow::Result<Vec<GroupConnectionPermission>> {
        let perms = sqlx::query_as::<_, GroupConnectionPermission>(
            "SELECT * FROM group_connection_permissions WHERE connection_id = $1",
        )
        .bind(conn_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(perms)
    }

    // ============================================================
    // Group Table Permissions
    // ============================================================

    async fn grant_group_table_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
        table_name: &str,
        permission: &str,
    ) -> anyhow::Result<GroupTablePermission> {
        let perm = sqlx::query_as::<_, GroupTablePermission>(
            r#"INSERT INTO group_table_permissions (group_id, connection_id, table_name, permission)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (group_id, connection_id, table_name) DO UPDATE SET permission = $4
               RETURNING *"#,
        )
        .bind(group_id)
        .bind(conn_id)
        .bind(table_name)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(perm)
    }

    async fn revoke_group_table_permission(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
        table_name: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM group_table_permissions WHERE connection_id = $1 AND group_id = $2 AND table_name = $3",
        )
        .bind(conn_id)
        .bind(group_id)
        .bind(table_name)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_group_table_permissions(
        &self,
        conn_id: &Uuid,
        group_id: &Uuid,
    ) -> anyhow::Result<Vec<GroupTablePermission>> {
        let perms = sqlx::query_as::<_, GroupTablePermission>(
            "SELECT * FROM group_table_permissions WHERE connection_id = $1 AND group_id = $2",
        )
        .bind(conn_id)
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(perms)
    }

    // ============================================================
    // Permission Resolution
    // ============================================================

    async fn resolve_connection_permission(
        &self,
        user: &AppUser,
        conn_id: &Uuid,
    ) -> anyhow::Result<(PermissionLevel, bool)> {
        // 1. SuperAdmin → full access
        if user.role == "super_admin" {
            return Ok((PermissionLevel::Admin, true));
        }

        // 1.5. Connection owner → full access
        let is_owner = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM saved_connections WHERE id = $1 AND owner_user_id = $2)",
        )
        .bind(conn_id)
        .bind(user.id)
        .fetch_one(&self.pool)
        .await?;
        if is_owner {
            return Ok((PermissionLevel::Admin, true));
        }

        // 2. Check user-level permission
        let user_perm = sqlx::query_as::<_, UserConnectionPermission>(
            "SELECT * FROM user_connection_permissions WHERE user_id = $1 AND connection_id = $2",
        )
        .bind(user.id)
        .bind(conn_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(up) = user_perm {
            let level = PermissionLevel::from_str(&up.permission);
            return Ok((level, up.all_tables));
        }

        // 3. Check group-level permissions (max of all groups)
        let group_perms = sqlx::query_as::<_, GroupConnectionPermission>(
            r#"SELECT gcp.* FROM group_connection_permissions gcp
               INNER JOIN group_members gm ON gm.group_id = gcp.group_id
               WHERE gm.user_id = $1 AND gcp.connection_id = $2"#,
        )
        .bind(user.id)
        .bind(conn_id)
        .fetch_all(&self.pool)
        .await?;

        if group_perms.is_empty() {
            return Ok((PermissionLevel::None, false));
        }

        let mut best_level = PermissionLevel::None;
        let mut any_all_tables = false;
        for gp in &group_perms {
            let level = PermissionLevel::from_str(&gp.permission);
            if level > best_level {
                best_level = level;
            }
            if gp.all_tables {
                any_all_tables = true;
            }
        }

        Ok((best_level, any_all_tables))
    }

    async fn resolve_table_permission(
        &self,
        user: &AppUser,
        conn_id: &Uuid,
        table_name: &str,
    ) -> anyhow::Result<PermissionLevel> {
        // 1. SuperAdmin → full access
        if user.role == "super_admin" {
            return Ok(PermissionLevel::Admin);
        }

        // 2. Check connection-level permission first
        let (conn_level, all_tables) = self.resolve_connection_permission(user, conn_id).await?;
        if conn_level == PermissionLevel::None {
            return Ok(PermissionLevel::None);
        }

        // 3. If all_tables is true at connection level, check for table-level override
        if all_tables {
            let user_table = sqlx::query_as::<_, UserTablePermission>(
                "SELECT * FROM user_table_permissions WHERE user_id = $1 AND connection_id = $2 AND table_name = $3",
            )
            .bind(user.id)
            .bind(conn_id)
            .bind(table_name)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(utp) = user_table {
                return Ok(PermissionLevel::from_str(&utp.permission));
            }

            return Ok(conn_level);
        }

        // 4. all_tables = false → only allowed if explicit table permission exists
        let user_table = sqlx::query_as::<_, UserTablePermission>(
            "SELECT * FROM user_table_permissions WHERE user_id = $1 AND connection_id = $2 AND table_name = $3",
        )
        .bind(user.id)
        .bind(conn_id)
        .bind(table_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(utp) = user_table {
            return Ok(PermissionLevel::from_str(&utp.permission));
        }

        // Check group-level table permissions
        let group_table_perms = sqlx::query_as::<_, GroupTablePermission>(
            r#"SELECT gtp.* FROM group_table_permissions gtp
               INNER JOIN group_members gm ON gm.group_id = gtp.group_id
               WHERE gm.user_id = $1 AND gtp.connection_id = $2 AND gtp.table_name = $3"#,
        )
        .bind(user.id)
        .bind(conn_id)
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let best = group_table_perms
            .iter()
            .map(|p| PermissionLevel::from_str(&p.permission))
            .max()
            .unwrap_or(PermissionLevel::None);

        Ok(best)
    }
}
