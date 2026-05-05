# ADR-0003: SSE as Real-Time Transport for Live Feeds

**Date:** 2026-05-04
**Status:** Accepted
**Deciders:** Rena Okafor (CTO)
**Consulted:** Kai Sato (Frontend), Dmitri Volkov (Backend), Marcus Webb (API Architect)

## Context

The Evidence Board and Build Watch views require live event streaming from server to client.
Two transport options exist: SSE (Server-Sent Events) over HTTP and WebSocket (full-duplex).
The choice affects server implementation complexity, browser API used in `<RealTimeFeed>`,
and the contract for the streaming endpoints Dmitri must add to `control-api`.

## Decision Drivers

- All current live-feed use cases are strictly unidirectional (server pushes, client reads)
- No existing spec requires the client to send events back on a live-feed connection
- Operational simplicity: fewer stateful server resources at the current scale
- Security: SSE requires no protocol upgrade and has a well-understood threat model

## Options Considered

### Option A: SSE (Server-Sent Events)

Unidirectional, HTTP-native. Browser's native `EventSource` API handles reconnection
via `Last-Event-ID`. Works over HTTP/2 multiplexed streams. Axum supports SSE via
`axum::response::sse::Sse<S>` — no additional dependencies.

- Pro: No protocol upgrade. Standard HTTP request/response model. Automatic reconnect.
- Pro: No server-side connection registry. Stateless at the frame level.
- Pro: Works through HTTP/2 without the 6-connection-per-domain limit of HTTP/1.1.
- Con: Unidirectional only. If client-to-server events on the same channel are ever needed,
  the transport must be replaced.

### Option B: WebSocket

Full-duplex. Handles bidirectional event flow. Supported in Rust via `tokio-tungstenite`.

- Pro: Future-proof if interactive annotations or collaborative features are added.
- Con: Requires protocol upgrade handling in Axum. Stateful connection registry on the server.
  Explicit heartbeat management. Significant complexity for a strictly read-only current spec.

## Decision

SSE is the transport for all RealmForge live feeds.

## Rationale

Every current live-feed surface (Evidence Board, Build Watch, Live Watch signals) is
read-only: the server pushes, the client displays. SSE is the correct tool. The browser
`EventSource` API provides automatic reconnection and `Last-Event-ID` resumability at
no implementation cost. WebSocket complexity (connection registry, heartbeat, upgrade
handling) is not justified until a bidirectional use case exists in the spec. If that use
case arrives, it will receive its own transport decision as a new surface — not a retrofit
of the existing live-feed layer.

## Consequences

**Positive:**
- `<RealTimeFeed>` is implemented against the standard browser `EventSource` API
- Axum SSE endpoints are simpler than WebSocket upgrade handlers
- No server-side connection state to garbage-collect
- Security surface is standard HTTP — no WebSocket upgrade attack vectors

**Negative / Trade-offs:**
- If bidirectional client events are added to Evidence Board, this decision must be revisited
- HTTP/1.1 limits 6 concurrent SSE connections per domain (mitigated by HTTP/2)

**Risks:**
- HTTP/2 must be enabled in Axum TLS configuration. Nadia to confirm before `<RealTimeFeed>`
  goes to production. On HTTP/1.1, concurrent SSE connections per tab are browser-capped.

## Review Date

2027-05-04, or earlier if a bidirectional live-feed use case enters the spec.
