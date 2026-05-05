// ADR-0004: Bearer token authentication.
// The session_token from POST /v1/login is sent as Authorization: Bearer <token>
// on every API request. Token is stored in Zustand session store and never in localStorage.
//
// ADR-0005: Same-origin hosting. API_BASE_URL is empty string.
// All requests go to the same origin — no CORS headers required.

import { useSessionStore } from '@/store/session.store'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? ''

export interface FetchError {
  code: string
  title: string
  status: number
  detail: string
  trace_id?: string
}

// orval mutator: called for every generated API hook's fetch
export async function customFetch<T>(
  url: string,
  options?: RequestInit,
): Promise<T> {
  const token = useSessionStore.getState().sessionToken

  const headers = new Headers(options?.headers)
  headers.set('Content-Type', 'application/json')

  // ADR-0004: Bearer token — never log this header
  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  const response = await fetch(`${API_BASE_URL}${url}`, {
    ...(options ?? {}),
    headers,
  })

  if (!response.ok) {
    const problem: FetchError = await response.json().catch(() => ({
      code: 'UNKNOWN_ERROR',
      title: response.statusText,
      status: response.status,
      detail: 'An unexpected error occurred.',
    }))

    // 401 means the session has expired — clear local session state
    if (response.status === 401) {
      useSessionStore.getState().clearSession()
    }

    throw problem
  }

  // 204 No Content — return undefined cast to T
  if (response.status === 204) {
    return undefined as T
  }

  return response.json() as Promise<T>
}
