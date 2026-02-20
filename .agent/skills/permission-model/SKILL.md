---
description: Permission model design and resolution logic for Quick Operation
---

# Permission Model

Quick Operation uses a dual-level permission system: permissions can be assigned to **individual users** and/or **groups**. User-level permissions always take priority.

## Entities

| Entity           | Description                                                 |
| ---------------- | ----------------------------------------------------------- |
| **Organization** | Top-level tenant. Owns connections, users, and groups.      |
| **User**         | Member of an organization. Role: `super_admin` or `member`. |
| **Group**        | Organizational unit. Users can belong to multiple groups.   |
| **Connection**   | A saved database connection owned by the organization.      |

## Permission Levels

- `none` — Explicit deny (user-level only; overrides group permissions)
- `read` — Read-only access
- `write` — Read + write access
- `admin` — Full access including schema operations

## Permission Granularity

Permissions are applied at two levels:

1. **Connection level** — Controls access to the entire connection
   - `all_tables = true` → Access to all tables in the connection
   - `all_tables = false` → Only tables listed in table permissions
2. **Table level** — Controls access to specific tables within a connection

Both levels exist for user permissions AND group permissions (4 tables total).

## Resolution Algorithm

```
resolve_permission(user, connection, table):
  1. if user.role == "super_admin":
       return "admin"

  2. user_conn_perm = lookup user_connection_permissions(user, connection)
     if user_conn_perm exists:
       if user_conn_perm.permission == "none":
         return DENY
       if user_conn_perm.all_tables == true:
         # Check user table override
         user_table_perm = lookup user_table_permissions(user, connection, table)
         return user_table_perm ?? user_conn_perm.permission
       else:
         user_table_perm = lookup user_table_permissions(user, connection, table)
         return user_table_perm ?? DENY

  3. group_perms = for each group user belongs to:
       lookup group_connection_permissions(group, connection)
     if any group_perms exist:
       best = max(group_perms.permission)
       # Check table level
       if best.all_tables:
         group_table_perm = max of group_table_permissions for all groups
         return group_table_perm ?? best.permission
       else:
         group_table_perm = max of group_table_permissions for all groups
         return group_table_perm ?? DENY

  4. return DENY
```

## Database Tables

```sql
-- User-level permissions
user_connection_permissions (user_id, connection_id, permission, all_tables)
user_table_permissions (user_id, connection_id, table_name, permission)

-- Group-level permissions
group_connection_permissions (group_id, connection_id, permission, all_tables)
group_table_permissions (group_id, connection_id, table_name, permission)
```

## API Endpoints

### User Permissions

- `POST /api/connections/{conn_id}/user-permissions` — Grant
- `DELETE /api/connections/{conn_id}/user-permissions/{user_id}` — Revoke
- `GET /api/connections/{conn_id}/user-permissions` — List
- `POST /api/connections/{conn_id}/user-permissions/{user_id}/tables` — Grant table
- `DELETE /api/connections/{conn_id}/user-permissions/{user_id}/tables/{table}` — Revoke table
- `GET /api/connections/{conn_id}/user-permissions/{user_id}/tables` — List tables

### Group Permissions

- `POST /api/connections/{conn_id}/group-permissions` — Grant
- `DELETE /api/connections/{conn_id}/group-permissions/{group_id}` — Revoke
- `GET /api/connections/{conn_id}/group-permissions` — List
- `POST /api/connections/{conn_id}/group-permissions/{group_id}/tables` — Grant table
- `DELETE /api/connections/{conn_id}/group-permissions/{group_id}/tables/{table}` — Revoke table
- `GET /api/connections/{conn_id}/group-permissions/{group_id}/tables` — List tables

All permission management endpoints require **SuperAdmin** role.
