import type {
  Connection,
  ConnectionRequest,
  Organization,
  CreateOrganizationRequest,
  AppUser,
  CreateUserRequest,
  Group,
  CreateGroupRequest,
  UserConnectionPermission,
  GrantUserConnectionPermissionRequest,
  UserTablePermission,
  GrantUserTablePermissionRequest,
  GroupConnectionPermission,
  GrantGroupConnectionPermissionRequest,
  GroupTablePermission,
  GrantGroupTablePermissionRequest,
  TableInfo,
  TableSchema,
  RowsResponse,
  RowData,
  ListRowsParams,
} from '../types';

const API_BASE = 'http://localhost:3001/api';

let authToken: string | null = null;

export function setAuthToken(token: string | null) {
  authToken = token;
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  if (authToken) {
    headers['Authorization'] = `Bearer ${authToken}`;
  }

  const res = await fetch(`${API_BASE}${path}`, {
    headers: {
      ...headers,
      ...options.headers,
    },
    ...options,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'Request failed');
  }
  if (res.status === 204) return null as T;
  return res.json() as Promise<T>;
}

export const api = {
  // Auth
  getMe: (): Promise<AppUser> =>
    request<AppUser>('/auth/me'),

  // Organizations
  createOrganization: (data: CreateOrganizationRequest): Promise<Organization> =>
    request<Organization>('/organizations', { method: 'POST', body: JSON.stringify(data) }),
  listOrganizations: (): Promise<Organization[]> =>
    request<Organization[]>('/organizations'),

  // Users
  createUser: (orgId: string, data: CreateUserRequest): Promise<AppUser> =>
    request<AppUser>(`/organizations/${orgId}/users`, { method: 'POST', body: JSON.stringify(data) }),
  listUsers: (orgId: string): Promise<AppUser[]> =>
    request<AppUser[]>(`/organizations/${orgId}/users`),

  // Groups
  createGroup: (orgId: string, data: CreateGroupRequest): Promise<Group> =>
    request<Group>(`/organizations/${orgId}/groups`, { method: 'POST', body: JSON.stringify(data) }),
  listGroups: (orgId: string): Promise<Group[]> =>
    request<Group[]>(`/organizations/${orgId}/groups`),
  addGroupMember: (groupId: string, userId: string): Promise<null> =>
    request<null>(`/groups/${groupId}/members`, { method: 'POST', body: JSON.stringify({ user_id: userId }) }),
  removeGroupMember: (groupId: string, userId: string): Promise<null> =>
    request<null>(`/groups/${groupId}/members/${userId}`, { method: 'DELETE' }),
  listGroupMembers: (groupId: string): Promise<AppUser[]> =>
    request<AppUser[]>(`/groups/${groupId}/members`),

  // Connections
  createConnection: (data: ConnectionRequest): Promise<Connection> =>
    request<Connection>('/connections', { method: 'POST', body: JSON.stringify(data) }),
  listConnections: (): Promise<Connection[]> =>
    request<Connection[]>('/connections'),
  deleteConnection: (id: string): Promise<null> =>
    request<null>(`/connections/${id}`, { method: 'DELETE' }),

  // User Connection Permissions
  grantUserConnPermission: (connId: string, data: GrantUserConnectionPermissionRequest): Promise<UserConnectionPermission> =>
    request<UserConnectionPermission>(`/connections/${connId}/user-permissions`, { method: 'POST', body: JSON.stringify(data) }),
  revokeUserConnPermission: (connId: string, userId: string): Promise<null> =>
    request<null>(`/connections/${connId}/user-permissions/${userId}`, { method: 'DELETE' }),
  listUserConnPermissions: (connId: string): Promise<UserConnectionPermission[]> =>
    request<UserConnectionPermission[]>(`/connections/${connId}/user-permissions`),

  // User Table Permissions
  grantUserTablePermission: (connId: string, userId: string, data: GrantUserTablePermissionRequest): Promise<UserTablePermission> =>
    request<UserTablePermission>(`/connections/${connId}/user-permissions/${userId}/tables`, { method: 'POST', body: JSON.stringify(data) }),
  revokeUserTablePermission: (connId: string, userId: string, table: string): Promise<null> =>
    request<null>(`/connections/${connId}/user-permissions/${userId}/tables/${table}`, { method: 'DELETE' }),
  listUserTablePermissions: (connId: string, userId: string): Promise<UserTablePermission[]> =>
    request<UserTablePermission[]>(`/connections/${connId}/user-permissions/${userId}/tables`),

  // Group Connection Permissions
  grantGroupConnPermission: (connId: string, data: GrantGroupConnectionPermissionRequest): Promise<GroupConnectionPermission> =>
    request<GroupConnectionPermission>(`/connections/${connId}/group-permissions`, { method: 'POST', body: JSON.stringify(data) }),
  revokeGroupConnPermission: (connId: string, groupId: string): Promise<null> =>
    request<null>(`/connections/${connId}/group-permissions/${groupId}`, { method: 'DELETE' }),
  listGroupConnPermissions: (connId: string): Promise<GroupConnectionPermission[]> =>
    request<GroupConnectionPermission[]>(`/connections/${connId}/group-permissions`),

  // Group Table Permissions
  grantGroupTablePermission: (connId: string, groupId: string, data: GrantGroupTablePermissionRequest): Promise<GroupTablePermission> =>
    request<GroupTablePermission>(`/connections/${connId}/group-permissions/${groupId}/tables`, { method: 'POST', body: JSON.stringify(data) }),
  revokeGroupTablePermission: (connId: string, groupId: string, table: string): Promise<null> =>
    request<null>(`/connections/${connId}/group-permissions/${groupId}/tables/${table}`, { method: 'DELETE' }),
  listGroupTablePermissions: (connId: string, groupId: string): Promise<GroupTablePermission[]> =>
    request<GroupTablePermission[]>(`/connections/${connId}/group-permissions/${groupId}/tables`),

  // Tables
  listTables: (connId: string): Promise<TableInfo[]> =>
    request<TableInfo[]>(`/connections/${connId}/tables`),
  getTableSchema: (connId: string, table: string): Promise<TableSchema> =>
    request<TableSchema>(`/connections/${connId}/tables/${table}/schema`),

  // Rows
  listRows: (connId: string, table: string, params: ListRowsParams = {}): Promise<RowsResponse> => {
    const qs = new URLSearchParams();
    if (params.page) qs.set('page', String(params.page));
    if (params.per_page) qs.set('per_page', String(params.per_page));
    if (params.sort_by) qs.set('sort_by', params.sort_by);
    if (params.sort_order) qs.set('sort_order', params.sort_order);
    if (params.filter) qs.set('filter', params.filter);
    return request<RowsResponse>(`/connections/${connId}/tables/${table}/rows?${qs.toString()}`);
  },
  getRow: (connId: string, table: string, pk: string): Promise<RowData> =>
    request<RowData>(`/connections/${connId}/tables/${table}/rows/${pk}`),
  createRow: (connId: string, table: string, data: RowData): Promise<RowData> =>
    request<RowData>(`/connections/${connId}/tables/${table}/rows`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateRow: (connId: string, table: string, pk: string, data: RowData): Promise<RowData> =>
    request<RowData>(`/connections/${connId}/tables/${table}/rows/${pk}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  deleteRow: (connId: string, table: string, pk: string): Promise<null> =>
    request<null>(`/connections/${connId}/tables/${table}/rows/${pk}`, {
      method: 'DELETE',
    }),
};
