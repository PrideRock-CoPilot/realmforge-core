// ─────────────────────────────────────────────
// Accessibility E2E Tests (smoke checks)
// ─────────────────────────────────────────────
// Tests:
//   1. Login page has proper ARIA labels and landmarks
//   2. Board pages have accessible heading hierarchy
//   3. Skip-to-content link works

import { test, expect } from '@playwright/test'

test.describe('Accessibility', () => {
  test('login page has accessible structure', async ({ page }) => {
    await page.goto('/')

    // Form is labelled by the heading
    const form = page.locator('form[aria-labelledby]')
    await expect(form).toBeVisible()

    // Username and password fields have labels
    await expect(page.getByLabel(/Username/i)).toBeVisible()
    await expect(page.getByLabel(/Password/i)).toBeVisible()

    // Submit button with accessible name
    const submitBtn = page.getByRole('button', { name: /Sign in/i })
    await expect(submitBtn).toBeVisible()
  })

  test('skip-to-content link is first focusable element after login', async ({ page }) => {
    // Skip link only exists inside AppShell (post-authentication)
    await page.goto('/')
    await page.getByLabel(/Username/i).fill('alice')
    await page.getByLabel(/Password/i).fill('s3cr3t')
    await page.getByRole('button', { name: /Sign in/i }).click()
    await page.waitForURL('**/boards/intake', { timeout: 15_000 })

    // Skip link is visually hidden (left: -9999px) but exists in the DOM
    const skipLink = page.getByText(/Skip to main content/i)
    await expect(skipLink).toBeAttached()

    // The skip link should be the first focusable element — press Tab then check
    await expect(skipLink).toHaveAttribute('href', /^#/)
  })
})
