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

test.describe('Connection Management', () => {
  test('opens connection modal via manage button', async ({ page }) => {
    await page.goto('/');

    // Click the database icon button to open connection modal
    await page.locator('.conn-manager-btn').click();

    // Modal should appear
    await expect(page.locator('.conn-modal')).toBeVisible();
    await expect(page.locator('.modal-header h2')).toBeVisible();
  });

  test('shows empty state when no connections', async ({ page }) => {
    await page.goto('/');
    await page.locator('.conn-manager-btn').click();

    // Should show empty state or connection list
    const modal = page.locator('.conn-modal');
    await expect(modal).toBeVisible();

    // The empty state has an "Add Connection" button
    // or the list shows existing connections
    const hasEmpty = await page.locator('.conn-modal-empty').isVisible().catch(() => false);
    const hasList = await page.locator('.conn-modal-list').isVisible().catch(() => false);
    expect(hasEmpty || hasList).toBeTruthy();
  });

  test('can close connection modal via close button', async ({ page }) => {
    await page.goto('/');
    await page.locator('.conn-manager-btn').click();
    await expect(page.locator('.conn-modal')).toBeVisible();

    // Close via X button
    await page.locator('.modal-close').click();
    await expect(page.locator('.conn-modal')).not.toBeVisible();
  });

  test('can close connection modal via overlay click', async ({ page }) => {
    await page.goto('/');
    await page.locator('.conn-manager-btn').click();
    await expect(page.locator('.conn-modal')).toBeVisible();

    // Click on overlay (outside modal content)
    await page.locator('.modal-overlay').click({ position: { x: 10, y: 10 } });
    await expect(page.locator('.conn-modal')).not.toBeVisible();
  });

  test('opens db type selection from welcome page', async ({ page }) => {
    await page.goto('/');

    // Click "Connect to Database" button on welcome page
    const connectBtn = page.locator('.welcome .btn-primary');
    await connectBtn.click();

    // DB type selection page should appear
    await expect(page.locator('.dbtype-select-page')).toBeVisible();
  });

  test('can select PostgreSQL and see connection form', async ({ page }) => {
    await page.goto('/');

    // Open DB type select
    await page.locator('.welcome .btn-primary').click();
    await expect(page.locator('.dbtype-select-page')).toBeVisible();

    // Click PostgreSQL option
    await page.locator('.dbtype-option').first().click();

    // Connection page should appear
    await expect(page.locator('.connection-page')).toBeVisible();
  });

  test('connection form has required fields', async ({ page }) => {
    await page.goto('/');

    // Navigate to connection form
    await page.locator('.welcome .btn-primary').click();
    await page.locator('.dbtype-option').first().click();

    // Check for required form fields
    await expect(page.locator('input[name="name"]')).toBeVisible();
    await expect(page.locator('input[name="host"]')).toBeVisible();
    await expect(page.locator('input[name="port"]')).toBeVisible();
    await expect(page.locator('input[name="database"]')).toBeVisible();
  });

  test('can go back from connection form to db type select', async ({ page }) => {
    await page.goto('/');

    // Open connection form
    await page.locator('.welcome .btn-primary').click();
    await page.locator('.dbtype-option').first().click();
    await expect(page.locator('.connection-page')).toBeVisible();

    // Click back button
    await page.locator('.btn-back').click();

    // Should return to db type select
    await expect(page.locator('.dbtype-select-page')).toBeVisible();
  });
});
