use uuid::Uuid;

use crate::domain::repository::UserRepository;
use crate::domain::user::AppUser;

use super::UsecaseError;
use super::error::require_super_admin;

pub async fn create_user(
    user_repo: &dyn UserRepository,
    caller: &AppUser,
    org_id: &Uuid,
    name: &str,
    email: &str,
    role: &str,
) -> Result<AppUser, UsecaseError> {
    require_super_admin(caller)?;
    user_repo
        .create(org_id, name, email, role)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn list_users(
    user_repo: &dyn UserRepository,
    org_id: &Uuid,
) -> Result<Vec<AppUser>, UsecaseError> {
    user_repo
        .list_by_org(org_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

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

    struct MockUserRepo {
        fail: bool,
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(
            &self,
            org_id: &Uuid,
            name: &str,
            email: &str,
            role: &str,
        ) -> anyhow::Result<AppUser> {
            if self.fail {
                anyhow::bail!("create failed");
            }
            Ok(AppUser {
                id: Uuid::new_v4(),
                organization_id: Some(*org_id),
                name: name.to_string(),
                email: email.to_string(),
                role: role.to_string(),
                auth_provider: None,
                provider_id: None,
                avatar_url: None,
                created_at: None,
                updated_at: None,
            })
        }

        async fn list_by_org(&self, _org_id: &Uuid) -> anyhow::Result<Vec<AppUser>> {
            if self.fail {
                anyhow::bail!("list failed");
            }
            Ok(vec![make_user("member", None)])
        }

        async fn get(&self, _user_id: &Uuid) -> anyhow::Result<Option<AppUser>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn create_user_as_super_admin() {
        let repo = MockUserRepo { fail: false };
        let caller = make_user("super_admin", None);
        let org_id = Uuid::new_v4();

        let result =
            create_user(&repo, &caller, &org_id, "Alice", "alice@test.com", "member").await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, "alice@test.com");
    }

    #[tokio::test]
    async fn create_user_as_member_forbidden() {
        let repo = MockUserRepo { fail: false };
        let caller = make_user("member", None);
        let org_id = Uuid::new_v4();

        let result =
            create_user(&repo, &caller, &org_id, "Alice", "alice@test.com", "member").await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn create_user_repo_failure() {
        let repo = MockUserRepo { fail: true };
        let caller = make_user("super_admin", None);
        let org_id = Uuid::new_v4();

        let result =
            create_user(&repo, &caller, &org_id, "Alice", "alice@test.com", "member").await;
        assert!(matches!(result.unwrap_err(), UsecaseError::BadRequest(_)));
    }

    #[tokio::test]
    async fn list_users_success() {
        let repo = MockUserRepo { fail: false };
        let org_id = Uuid::new_v4();

        let result = list_users(&repo, &org_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn list_users_failure() {
        let repo = MockUserRepo { fail: true };
        let org_id = Uuid::new_v4();

        let result = list_users(&repo, &org_id).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Internal(_)));
    }
}
