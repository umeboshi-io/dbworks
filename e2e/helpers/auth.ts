import jwt from 'jsonwebtoken';

const JWT_SECRET = process.env.JWT_SECRET || 'dbworks-dev-secret-change-me';
const API_BASE = process.env.API_BASE || 'http://localhost:3001/api';

// Fixed test user ID
export const TEST_USER_ID = '00000000-0000-0000-0000-e2e000000001';
const TEST_USER_EMAIL = 'e2e-test@dbworks.local';

/**
 * Generate a valid JWT token for the given user.
 * Matches the backend Claims struct exactly.
 */
export function generateToken(user: {
  id: string;
  role: string;
  email: string;
}): string {
  const payload = {
    sub: user.id,
    role: user.role,
    email: user.email,
    exp: Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60, // 7 days
  };
  return jwt.sign(payload, JWT_SECRET);
}

/**
 * Seed a test user in the database via direct SQL call through the psql CLI.
 * This bypasses the API layer since we need a user to exist before
 * we can authenticate via JWT.
 */
export async function seedTestData(): Promise<{
  userId: string;
  orgId: string;
  token: string;
}> {
  const { execSync } = await import('child_process');
  const { resolve, dirname } = await import('path');
  const { fileURLToPath } = await import('url');
  const orgId = '00000000-0000-0000-0000-e2e000000002';

  // Resolve project root (one level up from e2e/)
  const __dirname = dirname(fileURLToPath(import.meta.url));
  const projectRoot = resolve(__dirname, '..', '..');

  // Seed user and org via psql (table is app_users, not users)
  const sql = [
    `INSERT INTO app_users (id, name, email, role) VALUES ('${TEST_USER_ID}', 'E2E Test User', '${TEST_USER_EMAIL}', 'member') ON CONFLICT (id) DO NOTHING`,
    `INSERT INTO organizations (id, name) VALUES ('${orgId}', 'E2E Test Org') ON CONFLICT (id) DO NOTHING`,
    `INSERT INTO organization_members (organization_id, user_id, role) VALUES ('${orgId}', '${TEST_USER_ID}', 'owner') ON CONFLICT (organization_id, user_id) DO NOTHING`,
  ].join('; ');

  try {
    execSync(
      `docker compose exec -T postgres psql -U dbworks -d dbworks_dev -c "${sql}"`,
      {
        cwd: projectRoot,
        stdio: 'pipe',
        timeout: 10000,
      }
    );
  } catch (err) {
    // User might already exist, continue
    console.warn('Seed SQL warning (may be safe to ignore):', (err as Error).message?.slice(0, 200));
  }

  const token = generateToken({
    id: TEST_USER_ID,
    role: 'member',
    email: TEST_USER_EMAIL,
  });

  return { userId: TEST_USER_ID, orgId, token };
}

/**
 * Storage state for authenticated Playwright sessions.
 * Sets the JWT token in localStorage so the frontend picks it up.
 */
export function authStorageState(token: string) {
  return {
    cookies: [],
    origins: [
      {
        origin: 'http://localhost:5173',
        localStorage: [{ name: 'qo_token', value: token }],
      },
    ],
  };
}
