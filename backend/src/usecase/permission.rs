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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    fn make_user(role: &str) -> AppUser {
        AppUser {
            id: Uuid::new_v4(),
            organization_id: None,
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

    /// A mock permission repo that controls grant/revoke behavior via `fail` and `revoke_result`.
    struct MockPermissionRepo {
        fail: bool,
        revoke_result: bool, // true = found & revoked, false = not found
    }

    #[async_trait]
    impl PermissionRepository for MockPermissionRepo {
        async fn grant_user_connection_permission(
            &self,
            conn_id: &Uuid,
            user_id: &Uuid,
            permission: &str,
            all_tables: bool,
        ) -> anyhow::Result<UserConnectionPermission> {
            if self.fail {
                anyhow::bail!("grant failed");
            }
            Ok(UserConnectionPermission {
                id: Uuid::new_v4(),
                user_id: *user_id,
                connection_id: *conn_id,
                permission: permission.to_string(),
                all_tables,
                granted_at: None,
            })
        }

        async fn revoke_user_connection_permission(
            &self,
            _conn_id: &Uuid,
            _user_id: &Uuid,
        ) -> anyhow::Result<bool> {
            if self.fail {
                anyhow::bail!("revoke failed");
            }
            Ok(self.revoke_result)
        }

        async fn list_user_connection_permissions(
            &self,
            _conn_id: &Uuid,
        ) -> anyhow::Result<Vec<UserConnectionPermission>> {
            if self.fail {
                anyhow::bail!("list failed");
            }
            Ok(vec![])
        }

        async fn grant_user_table_permission(
            &self,
            conn_id: &Uuid,
            user_id: &Uuid,
            table_name: &str,
            permission: &str,
        ) -> anyhow::Result<UserTablePermission> {
            if self.fail {
                anyhow::bail!("grant failed");
            }
            Ok(UserTablePermission {
                id: Uuid::new_v4(),
                user_id: *user_id,
                connection_id: *conn_id,
                table_name: table_name.to_string(),
                permission: permission.to_string(),
                granted_at: None,
            })
        }

        async fn revoke_user_table_permission(
            &self,
            _conn_id: &Uuid,
            _user_id: &Uuid,
            _table_name: &str,
        ) -> anyhow::Result<bool> {
            if self.fail {
                anyhow::bail!("revoke failed");
            }
            Ok(self.revoke_result)
        }

        async fn list_user_table_permissions(
            &self,
            _conn_id: &Uuid,
            _user_id: &Uuid,
        ) -> anyhow::Result<Vec<UserTablePermission>> {
            if self.fail {
                anyhow::bail!("list failed");
            }
            Ok(vec![])
        }

        async fn grant_group_connection_permission(
            &self,
            conn_id: &Uuid,
            group_id: &Uuid,
            permission: &str,
            all_tables: bool,
        ) -> anyhow::Result<GroupConnectionPermission> {
            if self.fail {
                anyhow::bail!("grant failed");
            }
            Ok(GroupConnectionPermission {
                id: Uuid::new_v4(),
                group_id: *group_id,
                connection_id: *conn_id,
                permission: permission.to_string(),
                all_tables,
                granted_at: None,
            })
        }

        async fn revoke_group_connection_permission(
            &self,
            _conn_id: &Uuid,
            _group_id: &Uuid,
        ) -> anyhow::Result<bool> {
            if self.fail {
                anyhow::bail!("revoke failed");
            }
            Ok(self.revoke_result)
        }

        async fn list_group_connection_permissions(
            &self,
            _conn_id: &Uuid,
        ) -> anyhow::Result<Vec<GroupConnectionPermission>> {
            if self.fail {
                anyhow::bail!("list failed");
            }
            Ok(vec![])
        }

        async fn grant_group_table_permission(
            &self,
            conn_id: &Uuid,
            group_id: &Uuid,
            table_name: &str,
            permission: &str,
        ) -> anyhow::Result<GroupTablePermission> {
            if self.fail {
                anyhow::bail!("grant failed");
            }
            Ok(GroupTablePermission {
                id: Uuid::new_v4(),
                group_id: *group_id,
                connection_id: *conn_id,
                table_name: table_name.to_string(),
                permission: permission.to_string(),
                granted_at: None,
            })
        }

        async fn revoke_group_table_permission(
            &self,
            _conn_id: &Uuid,
            _group_id: &Uuid,
            _table_name: &str,
        ) -> anyhow::Result<bool> {
            if self.fail {
                anyhow::bail!("revoke failed");
            }
            Ok(self.revoke_result)
        }

        async fn list_group_table_permissions(
            &self,
            _conn_id: &Uuid,
            _group_id: &Uuid,
        ) -> anyhow::Result<Vec<GroupTablePermission>> {
            if self.fail {
                anyhow::bail!("list failed");
            }
            Ok(vec![])
        }

        async fn resolve_connection_permission(
            &self,
            _user: &AppUser,
            _conn_id: &Uuid,
        ) -> anyhow::Result<(PermissionLevel, bool)> {
            Ok((PermissionLevel::Read, false))
        }

        async fn resolve_table_permission(
            &self,
            _user: &AppUser,
            _conn_id: &Uuid,
            _table_name: &str,
        ) -> anyhow::Result<PermissionLevel> {
            Ok(PermissionLevel::Read)
        }
    }

    // ============================================================
    // User Connection Permission Tests
    // ============================================================

    #[tokio::test]
    async fn grant_user_conn_perm_as_admin() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result = grant_user_connection_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "read",
            true,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().permission, "read");
    }

    #[tokio::test]
    async fn grant_user_conn_perm_as_member_forbidden() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("member");
        let result = grant_user_connection_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "read",
            true,
        )
        .await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn revoke_user_conn_perm_found() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: true,
        };
        let caller = make_user("super_admin");
        let result =
            revoke_user_connection_permission(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4())
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn revoke_user_conn_perm_not_found() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result =
            revoke_user_connection_permission(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4())
                .await;
        assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
    }

    // ============================================================
    // Group Connection Permission Tests
    // ============================================================

    #[tokio::test]
    async fn grant_group_conn_perm_as_admin() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result = grant_group_connection_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "write",
            false,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().permission, "write");
    }

    #[tokio::test]
    async fn grant_group_conn_perm_as_member_forbidden() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("member");
        let result = grant_group_connection_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "write",
            false,
        )
        .await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn revoke_group_conn_perm_found() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: true,
        };
        let caller = make_user("super_admin");
        let result =
            revoke_group_connection_permission(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4())
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn revoke_group_conn_perm_not_found() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result =
            revoke_group_connection_permission(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4())
                .await;
        assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
    }

    // ============================================================
    // User Table Permission Tests
    // ============================================================

    #[tokio::test]
    async fn grant_user_table_perm_as_admin() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result = grant_user_table_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "users",
            "read",
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn revoke_user_table_perm_not_found() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result =
            revoke_user_table_permission(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4(), "users")
                .await;
        assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
    }

    // ============================================================
    // Group Table Permission Tests
    // ============================================================

    #[tokio::test]
    async fn grant_group_table_perm_as_admin() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result = grant_group_table_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "orders",
            "write",
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn revoke_group_table_perm_not_found() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result = revoke_group_table_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "orders",
        )
        .await;
        assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
    }

    // ============================================================
    // List Permission Tests
    // ============================================================

    #[tokio::test]
    async fn list_user_conn_perms_success() {
        let repo = MockPermissionRepo {
            fail: false,
            revoke_result: false,
        };
        let result = list_user_connection_permissions(&repo, &Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_user_conn_perms_failure() {
        let repo = MockPermissionRepo {
            fail: true,
            revoke_result: false,
        };
        let result = list_user_connection_permissions(&repo, &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Internal(_)));
    }

    // ============================================================
    // Repo Failure Tests
    // ============================================================

    #[tokio::test]
    async fn grant_user_conn_perm_repo_failure() {
        let repo = MockPermissionRepo {
            fail: true,
            revoke_result: false,
        };
        let caller = make_user("super_admin");
        let result = grant_user_connection_permission(
            &repo,
            &caller,
            &Uuid::new_v4(),
            &Uuid::new_v4(),
            "read",
            true,
        )
        .await;
        assert!(matches!(result.unwrap_err(), UsecaseError::BadRequest(_)));
    }
}
