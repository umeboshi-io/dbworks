-- Add organization_members join table for 1:N org membership with roles

CREATE TABLE IF NOT EXISTS organization_members (
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (organization_id, user_id)
);

-- Migrate existing data from app_users.organization_id
INSERT INTO organization_members (organization_id, user_id, role, joined_at)
SELECT organization_id, id, role, created_at
FROM app_users
WHERE organization_id IS NOT NULL
ON CONFLICT DO NOTHING;

-- Drop the organization_id column from app_users
ALTER TABLE app_users DROP COLUMN IF EXISTS organization_id;
