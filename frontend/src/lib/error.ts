import type { FetchError } from '@/api/client'

// ─────────────────────────────────────────────
// Error handling utilities for API errors
// ─────────────────────────────────────────────

export interface ParsedError {
  title: string
  detail: string
  code: string
  status: number
  traceId?: string
  isAuthError: boolean
  isServerError: boolean
  isClientError: boolean
  isNetworkError: boolean
}

export function parseError(error: unknown): ParsedError {
  // FetchError from client.ts
  if (isFetchError(error)) {
    const status = error.status
    return {
      title: error.title,
      detail: error.detail,
      code: error.code,
      status,
      traceId: error.trace_id,
      isAuthError: status === 401 || status === 403,
      isServerError: status >= 500,
      isClientError: status >= 400 && status < 500,
      isNetworkError: false,
    }
  }

  // Network error (fetch itself failed)
  if (error instanceof TypeError && error.message === 'Failed to fetch') {
    return {
      title: 'Connection Error',
      detail: 'Unable to reach the server. Please check your connection.',
      code: 'NETWORK_ERROR',
      status: 0,
      isAuthError: false,
      isServerError: false,
      isClientError: false,
      isNetworkError: true,
    }
  }

  // Standard Error
  if (error instanceof Error) {
    return {
      title: 'Error',
      detail: error.message,
      code: 'UNKNOWN_ERROR',
      status: 0,
      isAuthError: false,
      isServerError: false,
      isClientError: false,
      isNetworkError: false,
    }
  }

  return {
    title: 'Unknown Error',
    detail: String(error),
    code: 'UNKNOWN_ERROR',
    status: 0,
    isAuthError: false,
    isServerError: false,
    isClientError: false,
    isNetworkError: false,
  }
}

function isFetchError(err: unknown): err is FetchError {
  return (
    typeof err === 'object' &&
    err !== null &&
    'code' in err &&
    'title' in err &&
    'status' in err &&
    'detail' in err
  )
}
