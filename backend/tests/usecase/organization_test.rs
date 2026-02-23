use crate::common;
use dbworks_backend::domain::repository::{
    OrganizationMemberRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::infrastructure::database::organization_member_repo::PgOrganizationMemberRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::usecase::organization;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn create_organization_via_usecase() {
    let pool = common::setup_test_db().await;
    let repo = PgOrganizationRepository::new(pool);

    let org = organization::create_organization(&repo, "Usecase Org")
        .await
        .unwrap();

    assert_eq!(org.name, "Usecase Org");
    assert!(org.created_at.is_some());
}

#[tokio::test]
#[serial]
async fn list_organizations_returns_only_member_orgs() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let org_member_repo = PgOrganizationMemberRepository::new(pool);

    let user = user_repo
        .create("Test User", "test@example.com", "member")
        .await
        .unwrap();

    let org_a = org_repo.create("Alpha").await.unwrap();
    let _org_b = org_repo.create("Beta").await.unwrap();

    // Only add user to Alpha
    org_member_repo
        .add_member(&org_a.id, &user.id, "owner")
        .await
        .unwrap();

    let orgs = organization::list_organizations(&org_repo, &user)
        .await
        .unwrap();

    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0].name, "Alpha");
}
