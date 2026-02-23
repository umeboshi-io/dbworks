import { test, expect } from '@playwright/test';

test.describe('Login Page', () => {
  test('shows login page when not authenticated', async ({ page }) => {
    await page.goto('/');

    // Should display the login card
    await expect(page.locator('.login-card')).toBeVisible();
    await expect(page.locator('.login-card h1')).toHaveText('DBWorks');
  });

  test('has Google login button', async ({ page }) => {
    await page.goto('/');

    const googleBtn = page.locator('.login-btn-google');
    await expect(googleBtn).toBeVisible();
    await expect(googleBtn).toHaveAttribute(
      'href',
      /\/api\/auth\/google$/
    );
  });

  test('has GitHub login button', async ({ page }) => {
    await page.goto('/');

    const githubBtn = page.locator('.login-btn-github');
    await expect(githubBtn).toBeVisible();
    await expect(githubBtn).toHaveAttribute(
      'href',
      /\/api\/auth\/github$/
    );
  });

  test('does not show app content when unauthenticated', async ({ page }) => {
    await page.goto('/');

    // Sidebar and topbar should NOT be visible
    await expect(page.locator('.conn-topbar')).not.toBeVisible();
    await expect(page.locator('.sidebar')).not.toBeVisible();
  });
});
