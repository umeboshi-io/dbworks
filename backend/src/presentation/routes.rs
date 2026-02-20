use axum::Router;
use axum::routing::{delete, get, post, put};

use crate::infrastructure::auth::oauth;
use crate::presentation::handler::{connection, data, group, organization, permission, user};
use crate::presentation::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        // Auth routes
        .route("/api/auth/google", get(oauth::google_login))
        .route("/api/auth/google/callback", get(oauth::google_callback))
        .route("/api/auth/github", get(oauth::github_login))
        .route("/api/auth/github/callback", get(oauth::github_callback))
        .route("/api/auth/me", get(oauth::me))
        // Organization management
        .route("/api/organizations", post(organization::create_organization))
        .route("/api/organizations", get(organization::list_organizations))
        // User management
        .route(
            "/api/organizations/{org_id}/users",
            post(user::create_user),
        )
        .route(
            "/api/organizations/{org_id}/users",
            get(user::list_users),
        )
        // Group management
        .route(
            "/api/organizations/{org_id}/groups",
            post(group::create_group),
        )
        .route(
            "/api/organizations/{org_id}/groups",
            get(group::list_groups),
        )
        .route(
            "/api/groups/{group_id}/members",
            post(group::add_group_member),
        )
        .route(
            "/api/groups/{group_id}/members",
            get(group::list_group_members),
        )
        .route(
            "/api/groups/{group_id}/members/{user_id}",
            delete(group::remove_group_member),
        )
        // Connection management
        .route("/api/connections", post(connection::create_connection))
        .route("/api/connections", get(connection::list_connections))
        .route(
            "/api/connections/{conn_id}",
            delete(connection::delete_connection),
        )
        // User connection permissions
        .route(
            "/api/connections/{conn_id}/user-permissions",
            post(permission::grant_user_conn_permission),
        )
        .route(
            "/api/connections/{conn_id}/user-permissions",
            get(permission::list_user_conn_permissions),
        )
        .route(
            "/api/connections/{conn_id}/user-permissions/{user_id}",
            delete(permission::revoke_user_conn_permission),
        )
        // User table permissions
        .route(
            "/api/connections/{conn_id}/user-permissions/{user_id}/tables",
            post(permission::grant_user_table_permission),
        )
        .route(
            "/api/connections/{conn_id}/user-permissions/{user_id}/tables",
            get(permission::list_user_table_permissions),
        )
        .route(
            "/api/connections/{conn_id}/user-permissions/{user_id}/tables/{table}",
            delete(permission::revoke_user_table_permission),
        )
        // Group connection permissions
        .route(
            "/api/connections/{conn_id}/group-permissions",
            post(permission::grant_group_conn_permission),
        )
        .route(
            "/api/connections/{conn_id}/group-permissions",
            get(permission::list_group_conn_permissions),
        )
        .route(
            "/api/connections/{conn_id}/group-permissions/{group_id}",
            delete(permission::revoke_group_conn_permission),
        )
        // Group table permissions
        .route(
            "/api/connections/{conn_id}/group-permissions/{group_id}/tables",
            post(permission::grant_group_table_permission),
        )
        .route(
            "/api/connections/{conn_id}/group-permissions/{group_id}/tables",
            get(permission::list_group_table_permissions),
        )
        .route(
            "/api/connections/{conn_id}/group-permissions/{group_id}/tables/{table}",
            delete(permission::revoke_group_table_permission),
        )
        // Table introspection
        .route(
            "/api/connections/{conn_id}/tables",
            get(data::list_tables),
        )
        .route(
            "/api/connections/{conn_id}/tables/{table}/schema",
            get(data::get_table_schema),
        )
        // Row CRUD
        .route(
            "/api/connections/{conn_id}/tables/{table}/rows",
            get(data::list_rows),
        )
        .route(
            "/api/connections/{conn_id}/tables/{table}/rows",
            post(data::create_row),
        )
        .route(
            "/api/connections/{conn_id}/tables/{table}/rows/{pk}",
            get(data::get_row),
        )
        .route(
            "/api/connections/{conn_id}/tables/{table}/rows/{pk}",
            put(data::update_row),
        )
        .route(
            "/api/connections/{conn_id}/tables/{table}/rows/{pk}",
            delete(data::delete_row),
        )
}
