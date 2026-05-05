// ─────────────────────────────────────────────
// Auth setup — shared login helper for e2e tests
// ─────────────────────────────────────────────
// Logs in via POST /v1/login with seed credentials
// and stores the session token for subsequent requests.

import { Page, test as setup } from '@playwright/test'

const AUTH_FILE = '.e2e-auth.json'

export async function loginAsAlice(page: Page): Promise<string> {
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
    const body = await response.text()
    throw new Error(`Login failed (${response.status()}): ${body}`)
  }

  const data = await response.json()
  return data.session_token
}

export { AUTH_FILE }
