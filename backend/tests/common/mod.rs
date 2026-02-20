use sqlx::PgPool;

/// Connect to the test database and run migrations.
/// Uses `TEST_DATABASE_URL` env var if set, otherwise defaults to dbworks_test.
pub async fn setup_test_db() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dbworks:dbworks@localhost:5432/dbworks_test".to_string());

    let pool = PgPool::connect(&url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Clean all tables (order matters due to FK constraints)
    sqlx::query(
        r#"
        TRUNCATE
            group_table_permissions,
            group_connection_permissions,
            user_table_permissions,
            user_connection_permissions,
            group_members,
            groups,
            saved_connections,
            app_users,
            organizations
        CASCADE
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to truncate tables");

    pool
}
