import { test, expect } from '@playwright/test';
import { seedTestData, authStorageState } from '../helpers/auth';

let authState: ReturnType<typeof authStorageState>;

test.beforeAll(async () => {
  const { token } = await seedTestData();
  authState = authStorageState(token);
});

test.use({
  storageState: async ({}, use) => {
    await use(authState);
  },
});

test.describe('Authenticated App Shell', () => {
  test('shows top bar with connection tabs area', async ({ page }) => {
    await page.goto('/');

    // Should NOT show login page
    await expect(page.locator('.login-card')).not.toBeVisible();

    // Top bar should be visible
    await expect(page.locator('.conn-topbar')).toBeVisible();
  });

  test('shows sidebar with app title', async ({ page }) => {
    await page.goto('/');

    const sidebar = page.locator('.sidebar');
    await expect(sidebar).toBeVisible();

    // App title "DBWorks" should appear in the sidebar logo
    await expect(sidebar.locator('.logo')).toBeVisible();
  });

  test('shows welcome page with connect button', async ({ page }) => {
    await page.goto('/');

    const welcome = page.locator('.welcome');
    await expect(welcome).toBeVisible();

    // Should have a "Connect to Database" button
    const connectBtn = welcome.locator('.btn-primary');
    await expect(connectBtn).toBeVisible();
  });

  test('sidebar can be collapsed and expanded', async ({ page }) => {
    await page.goto('/');

    const sidebar = page.locator('.sidebar');
    const toggleBtn = page.locator('.sidebar-toggle');

    // Initially expanded
    await expect(sidebar).not.toHaveClass(/collapsed/);

    // Collapse
    await toggleBtn.click();
    await expect(sidebar).toHaveClass(/collapsed/);

    // Expand
    await toggleBtn.click();
    await expect(sidebar).not.toHaveClass(/collapsed/);
  });

  test('logout button returns to login page', async ({ page }) => {
    await page.goto('/');

    // Click logout button (⏻)
    await page.locator('button[title]').filter({ hasText: '⏻' }).click();

    // Should show login page
    await expect(page.locator('.login-card')).toBeVisible();
  });
});
