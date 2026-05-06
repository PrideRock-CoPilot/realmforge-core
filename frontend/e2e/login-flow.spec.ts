// ─────────────────────────────────────────────
// Login Flow E2E Tests
// ─────────────────────────────────────────────
// Tests:
//   1. Login page renders correctly (unauthenticated)
//   2. Successful login with alice/s3cr3t redirects to Intake Pipeline
//   3. Failed login shows error message
//   4. Logout clears session and returns to login page

import { test, expect } from '@playwright/test'

test.describe('Login Flow', () => {
  test('shows login page when unauthenticated', async ({ page }) => {
    await page.goto('/')

    // Should see the login form
    await expect(page.getByRole('heading', { name: /RealmForge/i })).toBeVisible()
    await expect(page.getByLabel(/Username/i)).toBeVisible()
    await expect(page.getByLabel(/Password/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /Sign in/i })).toBeVisible()

    // Should NOT be on a board
    await expect(page.getByText(/Intake Pipeline/i)).not.toBeVisible()
  })

  test('successful login with valid credentials redirects to boards', async ({ page }) => {
    await page.goto('/')

    // Fill in valid credentials
    await page.getByLabel(/Username/i).fill('alice')
    await page.getByLabel(/Password/i).fill('s3cr3t')

    // Submit
    await page.getByRole('button', { name: /Sign in/i }).click()

    // Wait for navigation to boards — the login POST goes to /v1/login
    // which requires the backend to be running. We wait for the Intake Pipeline to appear.
    await page.waitForURL('**/boards/intake', { timeout: 15_000 })

    // Verify board content is visible
    await expect(page.getByRole('heading', { name: /Intake Pipeline/i })).toBeVisible()

    // Verify the top nav shows the actor name
    await expect(page.getByText('alice')).toBeVisible()
  })

  test('failed login shows error message', async ({ page }) => {
    // Mock login endpoint to return 401
    await page.route('**/v1/login', async (route) => {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ detail: 'Invalid credentials' }),
      })
    })

    await page.goto('/')

    // Fill in invalid credentials
    await page.getByLabel(/Username/i).fill('alice')
    await page.getByLabel(/Password/i).fill('wrong-password')

    // Submit
    await page.getByRole('button', { name: /Sign in/i }).click()

    // Should show error
    await expect(page.getByRole('alert')).toBeVisible()
    await expect(page.getByRole('alert')).toContainText(/Invalid credentials/i)

    // Should still be on login page
    await expect(page.getByLabel(/Username/i)).toBeVisible()
  })

  test('logout clears session and returns to login', async ({ page }) => {
    await page.goto('/')

    // Login first
    await page.getByLabel(/Username/i).fill('alice')
    await page.getByLabel(/Password/i).fill('s3cr3t')
    await page.getByRole('button', { name: /Sign in/i }).click()

    // Wait for boards
    await page.waitForURL('**/boards/intake', { timeout: 15_000 })
    await expect(page.getByRole('heading', { name: /Intake Pipeline/i })).toBeVisible()

    // Click logout button
    await page.getByRole('button', { name: /Sign out/i }).click()

    // Should redirect to login page
    await expect(page.getByLabel(/Username/i)).toBeVisible()
    await expect(page.getByRole('heading', { name: /RealmForge/i })).toBeVisible()
  })
})
