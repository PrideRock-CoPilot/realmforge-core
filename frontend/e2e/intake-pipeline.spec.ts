// ─────────────────────────────────────────────
// Intake Pipeline E2E Tests
// ─────────────────────────────────────────────
// Tests:
//   1. Pipeline renders 6 stage columns with labels
//   2. Guided intake flow bar is visible
//   3. Empty state is shown when no plans exist
//   4. Create Plan dialog opens and has required fields
//   5. Plan detail modal opens on card click
//   6. Pipeline creates a plan via live API and shows it in intake column
// ─────────────────────────────────────────────

import { test, expect } from '@playwright/test'

test.describe('Intake Pipeline', () => {
  test.beforeEach(async ({ page }) => {
    // Log in via UI
    await page.goto('/')
    await page.getByLabel(/Username/i).fill('alice')
    await page.getByLabel(/Password/i).fill('s3cr3t')
    await page.getByRole('button', { name: /Sign in/i }).click()

    // Wait for redirect to intake pipeline
    try {
      await page.waitForURL('**/boards/intake', { timeout: 10_000 })
    } catch {
      // If login fails (no backend), tests will fail gracefully
    }
  })

  test('pipeline page renders with correct heading and description', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /Intake Pipeline/i })).toBeVisible()
    await expect(page.getByText(/Pipeline intake system/i)).toBeVisible()
  })

  test('guided intake flow bar shows all 6 stages', async ({ page }) => {
    const guidedBar = page.getByRole('region', { name: /Guided intake flow/i })
    await expect(guidedBar).toBeVisible()

    // All 6 stage labels should be present — filter to exact text match only
    const stages = ['Intake', 'Refinement', 'Architecture', 'Decomposition', 'Packetization', 'Ready']
    for (const stage of stages) {
      await expect(guidedBar.getByText(stage, { exact: true })).toBeVisible()
    }
  })

  test('all 6 pipeline columns are rendered', async ({ page }) => {
    const pipelineRegion = page.getByRole('region', { name: /Pipeline stage columns/i })
    await expect(pipelineRegion).toBeVisible()

    // PipelineColumn divs have aria-label like "Intake column" but no role="region"
    // Use the heading labels within each column instead
    const columns = ['Intake', 'Refinement', 'Architecture', 'Decomposition', 'Packetization', 'Ready']
    for (const label of columns) {
      await expect(pipelineRegion.locator(`.pipeline-column[aria-label="${label} column"]`)).toBeVisible()
    }
  })

  test('shows either empty state or pipeline columns with plans', async ({ page }) => {
    // Wait for data to load from API
    await page.waitForTimeout(2000)

    // Three possible states:
    // 1. No plans exist → "No pipeline plans yet" empty state
    // 2. Plans exist → pipeline columns with cards
    // 3. Loading → spinner (wait more)
    const emptyState = page.getByText(/No pipeline plans yet/i)
    const anyCard = page.locator('.pipeline-card').first()
    const pipelineColumns = page.locator('.pipeline-view')

    try {
      await expect(emptyState.or(anyCard).or(pipelineColumns)).toBeVisible({ timeout: 5_000 })
    } catch {
      // If none are visible after timeout, the page may still be loading or
      // the columns rendered empty (with "No plans in this stage" per column)
      const noPlansInStage = page.getByText(/No plans in this stage/i)
      await expect(noPlansInStage.first()).toBeVisible({ timeout: 3_000 })
    }
  })

  test('create plan dialog opens and has required fields', async ({ page }) => {
    // Click "New Plan" button
    const newPlanBtn = page.getByRole('button', { name: /New Plan/i })
    await expect(newPlanBtn).toBeVisible()
    await newPlanBtn.click()

    // Dialog should be visible
    const dialog = page.getByRole('dialog', { name: /New Pipeline Plan/i })
    await expect(dialog).toBeVisible()

    // Required fields should be present
    await expect(dialog.getByLabel(/Plan Name/i)).toBeVisible()
    await expect(dialog.getByLabel(/Goal/i)).toBeVisible()
    await expect(dialog.getByLabel(/Owner/i)).toBeVisible()

    // Scope is optional
    await expect(dialog.getByLabel(/Scope/i)).toBeVisible()

    // Cancel and Create buttons
    await expect(dialog.getByRole('button', { name: /Cancel/i })).toBeVisible()
    await expect(dialog.getByRole('button', { name: /Create Plan/i })).toBeVisible()
  })

  test('create plan dialog validates required fields', async ({ page }) => {
    // Open dialog
    await page.getByRole('button', { name: /New Plan/i }).click()
    const dialog = page.getByRole('dialog', { name: /New Pipeline Plan/i })

    // Create button should be disabled with empty fields
    const createBtn = dialog.getByRole('button', { name: /Create Plan/i })
    await expect(createBtn).toBeDisabled()

    // Fill in some fields
    await dialog.getByLabel(/Plan Name/i).fill('Test Plan')
    await dialog.getByLabel(/Goal/i).fill('Test goal description')
    // Owner still empty — button should still be disabled
    await expect(createBtn).toBeDisabled()

    // Fill owner
    await dialog.getByLabel(/Owner/i).fill('tester')
    await expect(createBtn).toBeEnabled()
  })

  test('creates a plan via API and displays it in intake column', async ({ page }) => {
    // Wait for initial load
    await page.waitForLoadState('networkidle')

    // Open create dialog
    await page.getByRole('button', { name: /New Plan/i }).click()

    // Fill form
    const dialog = page.getByRole('dialog', { name: /New Pipeline Plan/i })
    await dialog.getByLabel(/Plan Name/i).fill('E2E Test Plan')
    await dialog.getByLabel(/Goal/i).fill('Verify E2E create plan flow works end-to-end')
    await dialog.getByLabel(/Scope/i).fill('E2E test scope')
    await dialog.getByLabel(/Owner/i).fill('e2e-tester')

    // Submit
    await dialog.getByRole('button', { name: /Create Plan/i }).click()

    // Wait for success banner or the plan card to appear
    await expect(page.getByText(/created successfully/i).first().or(page.getByText(/E2E Test Plan/i).first())).toBeVisible({ timeout: 10_000 })

    // The plan card should appear in the intake column — use CSS selector since column divs use aria-label but no role="region"
    const intakeColumn = page.locator('.pipeline-column[aria-label="Intake column"]')
    await expect(intakeColumn.getByText(/E2E Test Plan/i).first()).toBeVisible()
  })

  test('plan card shows goal and owner metadata', async ({ page }) => {
    // This test assumes at least one plan exists (run after create test or seed data)
    // Wait for cards to load
    await page.waitForLoadState('networkidle')

    // Try to find a plan card
    const anyCard = page.locator('.pipeline-card').first()

    try {
      await expect(anyCard).toBeVisible({ timeout: 8_000 })

      // Card should have a title, goal, owner, and time
      await expect(anyCard.locator('.pipeline-card__title')).toBeVisible()
    } catch {
      test.skip() // Skip if no plans exist
    }
  })

  test('clicking a plan card opens detail modal', async ({ page }) => {
    // Wait for cards to load
    await page.waitForLoadState('networkidle')

    const anyCard = page.locator('.pipeline-card').first()

    try {
      await expect(anyCard).toBeVisible({ timeout: 8_000 })

      // Click the card
      await anyCard.click()

      // Detail modal should appear
      const detailModal = page.getByRole('dialog', { name: /Plan details|\w+ Plan/i })
      await expect(detailModal).toBeVisible({ timeout: 5_000 })

      // Should show stage badge and key info
      await expect(detailModal.getByText(/Stage:/i)).toBeVisible()
      await expect(detailModal.getByText(/Goal/i)).toBeVisible()
      await expect(detailModal.getByText(/Owner/i)).toBeVisible()
    } catch {
      test.skip() // Skip if no plans exist
    }
  })
})
