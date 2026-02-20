// ---- Organization ----
export interface Organization {
  id: string;
  name: string;
  created_at: string | null;
  updated_at: string | null;
}

export interface CreateOrganizationRequest {
  name: string;
}

// ---- User ----
export interface AppUser {
  id: string;
  organization_id: string;
  name: string;
  email: string;
  role: string;
  auth_provider: string | null;
  provider_id: string | null;
  avatar_url: string | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface CreateUserRequest {
  name: string;
  email: string;
  role?: string;
}

// ---- Group ----
export interface Group {
  id: string;
  organization_id: string;
  name: string;
  description: string | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface CreateGroupRequest {
  name: string;
  description?: string;
}

// ---- Connection ----
export interface ConnectionRequest {
  name: string;
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
  organization_id?: string;
}

export interface Connection {
  id: string;
  name: string;
  host: string;
  port: number;
  database: string;
  user: string;
  organization_id?: string;
}

// ---- Permissions ----
export interface UserConnectionPermission {
  id: string;
  user_id: string;
  connection_id: string;
  permission: string;
  all_tables: boolean;
  granted_at: string | null;
}

export interface GrantUserConnectionPermissionRequest {
  user_id: string;
  permission: string;
  all_tables?: boolean;
}

export interface UserTablePermission {
  id: string;
  user_id: string;
  connection_id: string;
  table_name: string;
  permission: string;
  granted_at: string | null;
}

export interface GrantUserTablePermissionRequest {
  table_name: string;
  permission: string;
}

export interface GroupConnectionPermission {
  id: string;
  group_id: string;
  connection_id: string;
  permission: string;
  all_tables: boolean;
  granted_at: string | null;
}

export interface GrantGroupConnectionPermissionRequest {
  group_id: string;
  permission: string;
  all_tables?: boolean;
}

export interface GroupTablePermission {
  id: string;
  group_id: string;
  connection_id: string;
  table_name: string;
  permission: string;
  granted_at: string | null;
}

export interface GrantGroupTablePermissionRequest {
  table_name: string;
  permission: string;
}

// ---- Table / Schema ----
export interface TableInfo {
  table_name: string;
  table_type: string;
}

export interface ColumnInfo {
  column_name: string;
  data_type: string;
  is_nullable: boolean;
  column_default: string | null;
  is_primary_key: boolean;
  max_length: number | null;
}

export interface TableSchema {
  table_name: string;
  columns: ColumnInfo[];
  primary_key_columns: string[];
}

// ---- Rows ----
export type RowData = Record<string, unknown>;

export interface RowsResponse {
  rows: RowData[];
  total_count: number;
  page: number;
  per_page: number;
}

export interface ListRowsParams {
  page?: number;
  per_page?: number;
  sort_by?: string;
  sort_order?: string;
  filter?: string;
}
