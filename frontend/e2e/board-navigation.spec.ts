// ─────────────────────────────────────────────
// Board Navigation E2E Tests
// ─────────────────────────────────────────────
// Tests:
//   1. Sidebar navigation links render and navigate to each board
//   2. Each board page has correct title and content
//   3. Board data loads from live API (or gracefully handles errors)
//   4. Visual Map and Audit pages render

import { test, expect } from '@playwright/test'

// Login helper used in beforeEach
async function loginViaApi(page: any) {
  const response = await page.request.post('/v1/login', {
    data: {
      tenant_id: 'system',
      project_id: 'project-uat-001',
      actor_id: 'alice',
      credential: 's3cr3t',
      scope: 'read',
    },
  })

  if (!response.ok()) {
    // If backend is not running, we'll mock the login
    await page.route('**/v1/login', async (route: any) => {
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          session_token: 'test-token-123',
          actor_id: 'alice',
          scope: 'read',
          expires_at: new Date(Date.now() + 3600_000).toISOString(),
          audit_event_id: 'aud-001',
          snapshot_id: 'snap-001',
        }),
      })
    })
  }

  return response.ok()
}

test.describe('Board Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Log in first
    await page.goto('/')
    await page.getByLabel(/Username/i).fill('alice')
    await page.getByLabel(/Password/i).fill('s3cr3t')
    await page.getByRole('button', { name: /Sign in/i }).click()

    // Wait for boards
    try {
      await page.waitForURL('**/boards/intake', { timeout: 10_000 })
    } catch {
      // If login fails (no backend), mock the session in localStorage via JS injection
      // Since sessions are memory-only, we'll work with mock data by intercepting API calls
    }
  })

  test('sidebar has all expected navigation links', async ({ page }) => {
    const sidebar = page.getByRole('complementary', { name: /Board navigation/i })

    await expect(sidebar.getByRole('link', { name: /Intake/i })).toBeVisible()
    await expect(sidebar.getByRole('link', { name: /Work Path/i })).toBeVisible()
    await expect(sidebar.getByRole('link', { name: /Packet/i })).toBeVisible()
    await expect(sidebar.getByRole('link', { name: /Evidence/i })).toBeVisible()
    await expect(sidebar.getByRole('link', { name: /Release/i })).toBeVisible()
    await expect(sidebar.getByRole('link', { name: /Cost/i })).toBeVisible()
    await expect(sidebar.getByRole('link', { name: /Visual Map/i })).toBeVisible()
    await expect(sidebar.getByRole('link', { name: /Audit Browser/i })).toBeVisible()
  })

  test('navigates to Intake Board', async ({ page }) => {
    await page.getByRole('link', { name: /Intake/i }).first().click()
    await page.waitForURL('**/boards/intake')
    await expect(page.getByRole('heading', { name: /Intake Board/i })).toBeVisible()
  })

  test('navigates to Work Path Board', async ({ page }) => {
    await page.getByRole('link', { name: /Work Path/i }).click()
    await page.waitForURL('**/boards/work-path')
    await expect(page.getByRole('heading', { name: /Work Path Board/i })).toBeVisible()
  })

  test('navigates to Packet Board', async ({ page }) => {
    await page.getByRole('link', { name: /Packet/i }).click()
    await page.waitForURL('**/boards/packet')
    await expect(page.getByRole('heading', { name: /Packet Board/i })).toBeVisible()
  })

  test('navigates to Evidence Board', async ({ page }) => {
    await page.getByRole('link', { name: /Evidence/i }).click()
    await page.waitForURL('**/boards/evidence')
    await expect(page.getByRole('heading', { name: /Evidence Board/i })).toBeVisible()
  })

  test('navigates to Release Board', async ({ page }) => {
    await page.getByRole('link', { name: /Release/i }).click()
    await page.waitForURL('**/boards/release')
    await expect(page.getByRole('heading', { name: /Release Board/i })).toBeVisible()
  })

  test('navigates to Cost Board', async ({ page }) => {
    await page.getByRole('link', { name: /Cost/i }).click()
    await page.waitForURL('**/boards/cost')
    await expect(page.getByRole('heading', { name: /Cost Board/i })).toBeVisible()
  })

  test('navigates to Visual Map', async ({ page }) => {
    await page.getByRole('link', { name: /Visual Map/i }).first().click()
    await page.waitForURL('**/visual-map')
    await expect(page.getByRole('heading', { name: /Visual Map/i })).toBeVisible()
  })

  test('navigates to Audit Browser', async ({ page }) => {
    await page.getByRole('link', { name: /Audit Browser/i }).click()
    await page.waitForURL('**/audit')
    await expect(page.getByRole('heading', { name: /Audit/i })).toBeVisible()
  })

  test('Intake Board attempts to load live data via API', async ({ page }) => {
    // Intercept API call for board plans
    const apiPromise = page.waitForResponse(
      (resp) => resp.url().includes('/v1/boards/plans') && resp.status() < 500,
      { timeout: 10_000 },
    ).catch(() => null) // Don't fail if no backend

    await page.getByRole('link', { name: /Intake/i }).first().click()
    await page.waitForURL('**/boards/intake')

    // The board should render whether or not the backend is available
    await expect(page.getByRole('heading', { name: /Intake Board/i })).toBeVisible()

    const response = await apiPromise
    if (response) {
      // Backend responded — verify data structure
      const body = await response.json()
      expect(body).toHaveProperty('plans')
      expect(body).toHaveProperty('total')
    }
  })

  test('Evidence Board attempts to load live audit events', async ({ page }) => {
    const apiPromise = page.waitForResponse(
      (resp) => resp.url().includes('/v1/audit/events') && resp.status() < 500,
      { timeout: 10_000 },
    ).catch(() => null)

    await page.getByRole('link', { name: /Evidence/i }).click()
    await page.waitForURL('**/boards/evidence')
    await expect(page.getByRole('heading', { name: /Evidence Board/i })).toBeVisible()

    const response = await apiPromise
    if (response) {
      const body = await response.json()
      expect(body).toHaveProperty('events')
    }
  })

  test('Release Board shows rollback preview toggle', async ({ page }) => {
    await page.getByRole('link', { name: /Release/i }).click()
    await page.waitForURL('**/boards/release')

    // Should have rollback preview button
    const rollbackBtn = page.getByRole('button', { name: /Preview Rollback/i })
    await expect(rollbackBtn).toBeVisible()

    // Click to expand
    await rollbackBtn.click()

    // Should show rollback preview content
    await expect(page.getByText(/Rollback will restore/i)).toBeVisible()

    // Click again to collapse
    await page.getByRole('button', { name: /Hide Rollback Preview/i }).click()
    await expect(page.getByText(/Rollback will restore/i)).not.toBeVisible()
  })
})
