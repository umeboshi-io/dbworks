-- ============================================================
-- DBWorks - App Schema
-- ============================================================

-- Organizations
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- App Users (members of an organization)
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

-- Groups
CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Group Members (many-to-many)
CREATE TABLE IF NOT EXISTS group_members (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
);

-- Saved Connections (owned by organization)
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

-- User-level connection permissions
CREATE TABLE IF NOT EXISTS user_connection_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    all_tables BOOLEAN NOT NULL DEFAULT true,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, connection_id)
);

-- User-level table permissions
CREATE TABLE IF NOT EXISTS user_table_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    table_name VARCHAR(200) NOT NULL,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, connection_id, table_name)
);

-- Group-level connection permissions
CREATE TABLE IF NOT EXISTS group_connection_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    all_tables BOOLEAN NOT NULL DEFAULT true,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(group_id, connection_id)
);

-- Group-level table permissions
CREATE TABLE IF NOT EXISTS group_table_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES saved_connections(id) ON DELETE CASCADE,
    table_name VARCHAR(200) NOT NULL,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(group_id, connection_id, table_name)
);

-- ============================================================
-- Seed Data
-- ============================================================

-- Default organization
INSERT INTO organizations (id, name) VALUES
    ('00000000-0000-0000-0000-000000000001', 'Default Organization');

-- Default super admin
INSERT INTO app_users (id, organization_id, name, email, role) VALUES
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Admin', 'admin@dbworks.local', 'super_admin');

-- Default admin group
INSERT INTO groups (id, organization_id, name, description) VALUES
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Admins', 'Organization administrators');

-- Add super admin to admin group
INSERT INTO group_members (group_id, user_id) VALUES
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001');

-- ============================================================
-- Sample data (for testing connected databases)
-- ============================================================

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    age INTEGER,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Products table
CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    price NUMERIC(10, 2) NOT NULL,
    stock INTEGER DEFAULT 0,
    category VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Orders table
CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    total_amount NUMERIC(10, 2),
    status VARCHAR(50) DEFAULT 'pending',
    ordered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Sample data
INSERT INTO users (name, email, age, is_active) VALUES
    ('田中太郎', 'tanaka@example.com', 30, true),
    ('佐藤花子', 'sato@example.com', 25, true),
    ('鈴木一郎', 'suzuki@example.com', 35, false),
    ('高橋美咲', 'takahashi@example.com', 28, true),
    ('伊藤健太', 'ito@example.com', 42, true);

INSERT INTO products (name, description, price, stock, category) VALUES
    ('ノートパソコン', '高性能ビジネスノートPC', 89800.00, 15, 'Electronics'),
    ('ワイヤレスマウス', 'Bluetooth対応マウス', 3980.00, 50, 'Electronics'),
    ('プログラミング入門', 'Rustで学ぶプログラミング', 2980.00, 100, 'Books'),
    ('USBハブ', 'USB-C 4ポートハブ', 2480.00, 30, 'Electronics'),
    ('デスクライト', 'LED調光デスクライト', 5980.00, 20, 'Office');

INSERT INTO orders (user_id, product_id, quantity, total_amount, status) VALUES
    (1, 1, 1, 89800.00, 'completed'),
    (1, 2, 2, 7960.00, 'completed'),
    (2, 3, 1, 2980.00, 'pending'),
    (3, 4, 3, 7440.00, 'shipped'),
    (4, 5, 1, 5980.00, 'pending');
