use crate::common;
use dbworks_backend::domain::repository::{OrganizationRepository, UserRepository};
use dbworks_backend::domain::user::AppUser;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::usecase::{self, UsecaseError};
use serial_test::serial;
use uuid::Uuid;

struct TestFixture {
    org_id: Uuid,
    admin: AppUser,
    member: AppUser,
    user_repo: PgUserRepository,
}

async fn setup() -> TestFixture {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool);

    let org = org_repo.create("Test Org").await.unwrap();
    let admin = user_repo
        .create(&org.id, "Admin", "admin@test.com", "super_admin")
        .await
        .unwrap();
    let member = user_repo
        .create(&org.id, "Member", "member@test.com", "member")
        .await
        .unwrap();

    TestFixture {
        org_id: org.id,
        admin,
        member,
        user_repo,
    }
}

#[tokio::test]
#[serial]
async fn create_user_as_super_admin() {
    let f = setup().await;

    let user = usecase::user::create_user(
        &f.user_repo,
        &f.admin,
        &f.org_id,
        "Alice",
        "alice@test.com",
        "member",
    )
    .await
    .unwrap();

    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@test.com");
    assert_eq!(user.role, "member");
    assert_eq!(user.organization_id, Some(f.org_id));
}

#[tokio::test]
#[serial]
async fn create_user_as_member_forbidden() {
    let f = setup().await;

    let result = usecase::user::create_user(
        &f.user_repo,
        &f.member,
        &f.org_id,
        "Alice",
        "alice@test.com",
        "member",
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn list_users_via_usecase() {
    let f = setup().await;

    // admin + member already exist from setup
    let users = usecase::user::list_users(&f.user_repo, &f.org_id)
        .await
        .unwrap();

    assert_eq!(users.len(), 2);
}
