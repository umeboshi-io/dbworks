-- DBWorks App Schema
-- organizations, app_users, groups, group_members,
-- saved_connections, and permission tables

CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS app_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    auth_provider VARCHAR(20),
    provider_id VARCHAR(255),
    avatar_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(organization_id, email),
    UNIQUE(auth_provider, provider_id)
);

CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS group_members (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
);

CREATE TABLE IF NOT EXISTS saved_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    host VARCHAR(500) NOT NULL,
    port INTEGER NOT NULL DEFAULT 5432,
    database_name VARCHAR(200) NOT NULL,
    username VARCHAR(200) NOT NULL,
    encrypted_password TEXT NOT NULL,
    created_by UUID REFERENCES app_users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_connection_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    all_tables BOOLEAN NOT NULL DEFAULT true,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, connection_id)
);

CREATE TABLE IF NOT EXISTS user_table_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    table_name VARCHAR(200) NOT NULL,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, connection_id, table_name)
);

CREATE TABLE IF NOT EXISTS group_connection_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    all_tables BOOLEAN NOT NULL DEFAULT true,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(group_id, connection_id)
);

CREATE TABLE IF NOT EXISTS group_table_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    table_name VARCHAR(200) NOT NULL,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(group_id, connection_id, table_name)
);
