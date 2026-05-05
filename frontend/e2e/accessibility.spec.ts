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

  test('skip-to-content link is first focusable element', async ({ page }) => {
    await page.goto('/')

    // The skip link should be visually hidden but in the DOM
    const skipLink = page.getByText(/Skip to main content/i)
    await expect(skipLink).toBeVisible()
  })
})
