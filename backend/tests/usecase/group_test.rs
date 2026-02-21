use crate::common;
use dbworks_backend::domain::repository::{
    GroupRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::domain::user::AppUser;
use dbworks_backend::infrastructure::database::group_repo::PgGroupRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::usecase::{self, UsecaseError};
use serial_test::serial;
use uuid::Uuid;

struct TestFixture {
    org_id: Uuid,
    admin: AppUser,
    member: AppUser,
    group_repo: PgGroupRepository,
}

async fn setup() -> TestFixture {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let group_repo = PgGroupRepository::new(pool);

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
        group_repo,
    }
}

#[tokio::test]
#[serial]
async fn create_group_as_super_admin() {
    let f = setup().await;

    let group = usecase::group::create_group(
        &f.group_repo,
        &f.admin,
        &f.org_id,
        "Engineers",
        Some("Eng team"),
    )
    .await
    .unwrap();

    assert_eq!(group.name, "Engineers");
    assert_eq!(group.description, Some("Eng team".to_string()));
    assert_eq!(group.organization_id, f.org_id);
}

#[tokio::test]
#[serial]
async fn create_group_as_member_forbidden() {
    let f = setup().await;

    let result =
        usecase::group::create_group(&f.group_repo, &f.member, &f.org_id, "Team", None).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn list_groups_via_usecase() {
    let f = setup().await;

    f.group_repo.create(&f.org_id, "Alpha", None).await.unwrap();
    f.group_repo.create(&f.org_id, "Beta", None).await.unwrap();

    let groups = usecase::group::list_groups(&f.group_repo, &f.org_id)
        .await
        .unwrap();

    assert_eq!(groups.len(), 2);
}

#[tokio::test]
#[serial]
async fn add_and_list_group_members_via_usecase() {
    let f = setup().await;

    let group = usecase::group::create_group(&f.group_repo, &f.admin, &f.org_id, "Team", None)
        .await
        .unwrap();

    // Add member
    usecase::group::add_group_member(&f.group_repo, &f.admin, &group.id, &f.member.id)
        .await
        .unwrap();

    let members = usecase::group::list_group_members(&f.group_repo, &group.id)
        .await
        .unwrap();

    assert_eq!(members.len(), 1);
    assert_eq!(members[0].id, f.member.id);
}

#[tokio::test]
#[serial]
async fn add_group_member_as_member_forbidden() {
    let f = setup().await;

    let group = usecase::group::create_group(&f.group_repo, &f.admin, &f.org_id, "Team", None)
        .await
        .unwrap();

    let result =
        usecase::group::add_group_member(&f.group_repo, &f.member, &group.id, &f.admin.id).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn remove_group_member_via_usecase() {
    let f = setup().await;

    let group = usecase::group::create_group(&f.group_repo, &f.admin, &f.org_id, "Team", None)
        .await
        .unwrap();

    usecase::group::add_group_member(&f.group_repo, &f.admin, &group.id, &f.member.id)
        .await
        .unwrap();

    let removed =
        usecase::group::remove_group_member(&f.group_repo, &f.admin, &group.id, &f.member.id)
            .await
            .unwrap();

    assert!(removed);

    let members = usecase::group::list_group_members(&f.group_repo, &group.id)
        .await
        .unwrap();
    assert!(members.is_empty());
}

#[tokio::test]
#[serial]
async fn remove_group_member_as_member_forbidden() {
    let f = setup().await;

    let group = usecase::group::create_group(&f.group_repo, &f.admin, &f.org_id, "Team", None)
        .await
        .unwrap();

    let result =
        usecase::group::remove_group_member(&f.group_repo, &f.member, &group.id, &f.admin.id).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}
