import { chromium } from '@playwright/test';
import * as crypto from 'crypto';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function generateJWT(): string {
  const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
  const payload = Buffer.from(JSON.stringify({
    sub: '00000000-0000-0000-0000-e2e000000001',
    role: 'member',
    email: 'demo@dbworks.local',
    exp: Math.floor(Date.now() / 1000) + 86400,
  })).toString('base64url');
  const signature = crypto
    .createHmac('sha256', 'dbworks-dev-secret-change-me')
    .update(`${header}.${payload}`)
    .digest('base64url');
  return `${header}.${payload}.${signature}`;
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 },
    recordVideo: {
      dir: path.join(__dirname, '../docs'),
      size: { width: 1280, height: 720 },
    },
  });

  const page = await context.newPage();
  const token = generateJWT();

  // Step 1: Set up auth + English
  await page.goto('http://localhost:5173/');
  await page.evaluate((t) => {
    localStorage.setItem('qo_token', t);
    localStorage.setItem('i18nextLng', 'en');
  }, token);
  await page.goto('http://localhost:5173/');
  await page.waitForTimeout(2000);

  // Step 2: Show dashboard
  await page.waitForTimeout(2000);

  // Step 3: Click "Connect to Database"
  await page.getByRole('button', { name: /connect to database/i }).click();
  await page.waitForTimeout(2000);

  // Step 4: Select PostgreSQL
  await page.getByText('PostgreSQL').click();
  await page.waitForTimeout(2000);

  // Step 5: Fill form
  await page.getByLabel(/connection name/i).fill('Demo Database');
  await page.waitForTimeout(800);
  await page.getByLabel(/^host$/i).fill('localhost');
  await page.waitForTimeout(800);
  await page.getByLabel(/^port$/i).fill('5432');
  await page.waitForTimeout(800);
  await page.getByLabel(/^database$/i).fill('dbworks_dev');
  await page.waitForTimeout(800);
  await page.getByLabel(/^user$/i).fill('dbworks');
  await page.waitForTimeout(800);
  await page.getByLabel(/^password$/i).fill('dbworks');
  await page.waitForTimeout(1500);

  // Step 6: Connect
  await page.getByRole('button', { name: /^connect$/i }).click();
  await page.waitForTimeout(3000);

  // Step 7: Browse tables (skip app_users to avoid showing personal data)
  await page.getByText('organizations').click();
  await page.waitForTimeout(3000);

  await page.getByText('organization_members').click();
  await page.waitForTimeout(3000);

  await page.getByText('groups').click();
  await page.waitForTimeout(3000);

  // Close to finalize video
  await context.close();
  await browser.close();

  console.log('Video recorded to docs/ directory');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
