-- Allow users without an organization to own connections directly

-- app_users: allow org-less users
ALTER TABLE app_users ALTER COLUMN organization_id DROP NOT NULL;

-- saved_connections: allow org-less connections (user-owned)
ALTER TABLE saved_connections ALTER COLUMN organization_id DROP NOT NULL;
ALTER TABLE saved_connections ADD COLUMN owner_user_id UUID REFERENCES app_users(id) ON DELETE CASCADE;

-- A connection must belong to either an org or a user (or both)
ALTER TABLE saved_connections ADD CONSTRAINT chk_connection_owner
  CHECK (organization_id IS NOT NULL OR owner_user_id IS NOT NULL);
