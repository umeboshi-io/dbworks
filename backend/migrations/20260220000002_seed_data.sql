-- Default seed data: organization, admin user, admin group

INSERT INTO organizations (id, name) VALUES
    ('00000000-0000-0000-0000-000000000001', 'Default Organization');

INSERT INTO app_users (id, organization_id, name, email, role) VALUES
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Admin', 'admin@dbworks.local', 'super_admin');

INSERT INTO groups (id, organization_id, name, description) VALUES
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Admins', 'Organization administrators');

INSERT INTO group_members (group_id, user_id) VALUES
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001');
