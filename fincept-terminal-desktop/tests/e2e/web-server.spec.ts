import { test, expect } from '@playwright/test';

test('serves API documentation', async ({ page }) => {
  await page.goto('/');
  await expect(
    page.getByRole('heading', { name: /Fincept Terminal API/i })
  ).toBeVisible();
});

test('health endpoint responds', async ({ request }) => {
  const response = await request.get('/api/health');
  expect(response.ok()).toBeTruthy();
  const payload = await response.json();
  expect(payload.status).toBe('healthy');
});
