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

test.describe('Organization Management', () => {
  test('opens organization page via settings button', async ({ page }) => {
    await page.goto('/');

    // Click the settings (⚙) button to open org page
    await page.locator('button[title]').filter({ hasText: '⚙' }).click();

    // Organization page should appear
    await expect(page.locator('.org-page')).toBeVisible();
    await expect(page.locator('.org-card')).toBeVisible();
  });

  test('create organization tab is shown by default', async ({ page }) => {
    await page.goto('/');
    await page.locator('button[title]').filter({ hasText: '⚙' }).click();

    // Create tab should be active
    const createTab = page.locator('.org-tab').first();
    await expect(createTab).toHaveClass(/active/);

    // Form should be visible with name input
    await expect(page.locator('#org-name')).toBeVisible();
  });

  test('can create a new organization', async ({ page }) => {
    await page.goto('/');
    await page.locator('button[title]').filter({ hasText: '⚙' }).click();

    const orgName = `Test Org ${Date.now()}`;
    await page.locator('#org-name').fill(orgName);
    await page.locator('.org-card .btn-primary[type="submit"]').click();

    // After creation, org page should close (onJoined callback)
    await expect(page.locator('.org-page')).not.toBeVisible();
  });

  test('can view organization list', async ({ page }) => {
    await page.goto('/');
    await page.locator('button[title]').filter({ hasText: '⚙' }).click();

    // Switch to list tab
    const listTab = page.locator('.org-tab').last();
    await listTab.click();

    // Should show the organization list
    await expect(page.locator('.org-list')).toBeVisible();
  });

  test('can close organization page', async ({ page }) => {
    await page.goto('/');
    await page.locator('button[title]').filter({ hasText: '⚙' }).click();
    await expect(page.locator('.org-page')).toBeVisible();

    // Click close button
    await page.locator('.org-card .modal-close').click();
    await expect(page.locator('.org-page')).not.toBeVisible();
  });
});
