use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::repository::GroupRepository;
use crate::domain::user::AppUser;

use super::UsecaseError;
use super::error::require_super_admin;

pub async fn create_group(
    group_repo: &dyn GroupRepository,
    caller: &AppUser,
    org_id: &Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Group, UsecaseError> {
    require_super_admin(caller)?;
    group_repo
        .create(org_id, name, description)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn list_groups(
    group_repo: &dyn GroupRepository,
    org_id: &Uuid,
) -> Result<Vec<Group>, UsecaseError> {
    group_repo
        .list_by_org(org_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

pub async fn add_group_member(
    group_repo: &dyn GroupRepository,
    caller: &AppUser,
    group_id: &Uuid,
    user_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
    group_repo
        .add_member(group_id, user_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn remove_group_member(
    group_repo: &dyn GroupRepository,
    caller: &AppUser,
    group_id: &Uuid,
    user_id: &Uuid,
) -> Result<bool, UsecaseError> {
    require_super_admin(caller)?;
    group_repo
        .remove_member(group_id, user_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn list_group_members(
    group_repo: &dyn GroupRepository,
    group_id: &Uuid,
) -> Result<Vec<AppUser>, UsecaseError> {
    group_repo
        .list_members(group_id)
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

    struct MockGroupRepo {
        fail: bool,
    }

    #[async_trait]
    impl GroupRepository for MockGroupRepo {
        async fn create(
            &self,
            org_id: &Uuid,
            name: &str,
            description: Option<&str>,
        ) -> anyhow::Result<Group> {
            if self.fail {
                anyhow::bail!("create failed");
            }
            Ok(Group {
                id: Uuid::new_v4(),
                organization_id: *org_id,
                name: name.to_string(),
                description: description.map(String::from),
                created_at: None,
                updated_at: None,
            })
        }

        async fn list_by_org(&self, _org_id: &Uuid) -> anyhow::Result<Vec<Group>> {
            if self.fail {
                anyhow::bail!("list failed");
            }
            Ok(vec![])
        }

        async fn add_member(&self, _group_id: &Uuid, _user_id: &Uuid) -> anyhow::Result<()> {
            if self.fail {
                anyhow::bail!("add failed");
            }
            Ok(())
        }

        async fn remove_member(&self, _group_id: &Uuid, _user_id: &Uuid) -> anyhow::Result<bool> {
            if self.fail {
                anyhow::bail!("remove failed");
            }
            Ok(true)
        }

        async fn list_members(&self, _group_id: &Uuid) -> anyhow::Result<Vec<AppUser>> {
            if self.fail {
                anyhow::bail!("list members failed");
            }
            Ok(vec![make_user("member")])
        }
    }

    #[tokio::test]
    async fn create_group_as_super_admin() {
        let repo = MockGroupRepo { fail: false };
        let caller = make_user("super_admin");
        let org_id = Uuid::new_v4();

        let result = create_group(&repo, &caller, &org_id, "Engineers", Some("Eng team")).await;
        assert!(result.is_ok());
        let group = result.unwrap();
        assert_eq!(group.name, "Engineers");
        assert_eq!(group.description, Some("Eng team".to_string()));
    }

    #[tokio::test]
    async fn create_group_as_member_forbidden() {
        let repo = MockGroupRepo { fail: false };
        let caller = make_user("member");
        let org_id = Uuid::new_v4();

        let result = create_group(&repo, &caller, &org_id, "Team", None).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn add_group_member_as_super_admin() {
        let repo = MockGroupRepo { fail: false };
        let caller = make_user("super_admin");

        let result = add_group_member(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn add_group_member_as_member_forbidden() {
        let repo = MockGroupRepo { fail: false };
        let caller = make_user("member");

        let result = add_group_member(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn remove_group_member_as_super_admin() {
        let repo = MockGroupRepo { fail: false };
        let caller = make_user("super_admin");

        let result = remove_group_member(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // true = removed
    }

    #[tokio::test]
    async fn remove_group_member_as_member_forbidden() {
        let repo = MockGroupRepo { fail: false };
        let caller = make_user("member");

        let result = remove_group_member(&repo, &caller, &Uuid::new_v4(), &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn list_groups_success() {
        let repo = MockGroupRepo { fail: false };
        let result = list_groups(&repo, &Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_group_members_success() {
        let repo = MockGroupRepo { fail: false };
        let result = list_group_members(&repo, &Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn list_group_members_failure() {
        let repo = MockGroupRepo { fail: true };
        let result = list_group_members(&repo, &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Internal(_)));
    }
}
