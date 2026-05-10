# API/MCP/CLI Surfaces — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the API/MCP/CLI transport layers to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2025-01-XX  
**Author:** Architecture Team  
**Reviewers:** Marcus Webb (API Architect), Dmitri Volkov (Backend Engineer), Rena Okafor (CTO)

**Important:** This document captures WHAT the API/MCP/CLI surfaces are supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

The API/MCP/CLI surfaces are RealmForge's transport layers - the thin protocol adapters that expose control plane services to users, agents, and external systems. These three surfaces exist to provide multiple interaction modes (HTTP REST, Model Context Protocol, command-line) without duplicating business logic.

The Transport Surfaces solve three critical problems:
1. **Protocol Adaptation**: Translate HTTP/MCP/CLI requests into service layer calls
2. **Error Translation**: Convert service-level errors into protocol-specific responses (HTTP status codes, MCP errors, CLI exit codes)
3. **Authentication Integration**: Extract credentials and session tokens from protocol headers/flags

The Transport Surfaces are NOT business logic layers - they are thin adapters that delegate all logic to the control plane service layer.

---

## 2. Responsibilities

The API/MCP/CLI surfaces own:

### Control API (REST)
* **HTTP Server**: Axum-based REST API server
* **Endpoint Routing**: Map HTTP paths to service layer methods
* **Request Parsing**: Deserialize JSON/form data into service types
* **Response Formatting**: Serialize service results into JSON responses
* **Status Code Mapping**: Convert service errors to HTTP status codes (401, 403, 404, 500, etc.)
* **Authentication**: Extract session tokens from Authorization headers
* **CORS**: Handle cross-origin requests for web clients

### Agent MCP (Model Context Protocol)
* **MCP Server**: Model Context Protocol server for AI agents
* **Tool Definitions**: Define MCP tools mapping to service methods
* **Tool Invocation**: Route tool calls to service layer
* **Result Formatting**: Format service results as MCP tool responses
* **Error Handling**: Convert service errors to MCP error responses
* **Authentication**: Extract agent credentials from MCP context

### Operator CLI
* **Command Parser**: Parse CLI commands and flags (clap)
* **Command Routing**: Map CLI commands to service layer methods
* **Output Formatting**: Format service results as text/JSON/table
* **Exit Code Mapping**: Convert service errors to exit codes (0 success, 1 error)
* **Configuration**: Load CLI config from files/environment
* **Interactive Mode**: Support interactive prompts and confirmations

---

## 3. Boundaries (What This Area Does NOT Own)

The API/MCP/CLI surfaces explicitly do NOT:

* **Business Logic**: No domain logic, policy evaluation, or orchestration - all in service layer
* **Data Persistence**: Do not access PostgreSQL or RFSource directly - all through service layer
* **Authorization**: Do not make authorization decisions - call service layer which calls policy engine
* **Session Management**: Do not create sessions - call session service
* **Audit Logging**: Do not write audit events - service layer handles that
* **Error Business Logic**: Do not implement error handling beyond translation - service layer handles retries, fallbacks

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Type-safe protocol handling, async I/O for API/MCP servers, memory safety
  * All three surfaces are Rust crates

**Why Rust?**  
Transport layers are the attack surface - bugs here can expose the system to injection, DoS, or authentication bypass. Rust's type system ensures protocol parsing is safe. The borrow checker prevents memory corruption in request handling. Async Tokio runtime enables high concurrency without thread explosion.

---

## 5. High-Level Architecture

* **Layer:** Transport Layer (outermost layer, directly exposed to users/agents)
* **Position:** Consumes control plane services, exposes protocols

**Key Components:**

### Control API (control-api crate)

1. **HTTP Server (Axum)**
   * Binds to port (e.g., 8080)
   * Routes requests to handlers
   * Middleware: logging, tracing, CORS, authentication

2. **Endpoint Handlers**
   * Session endpoints: POST /v1/sessions (issue), DELETE /v1/sessions/:id (revoke)
   * Command endpoints: POST /v1/commands (propose), POST /v1/commands/:id/authorize (authorize)
   * Audit endpoints: GET /v1/audit/events (query), GET /v1/audit/verify/:project_id (verify chain)
   * Snapshot endpoints: POST /v1/snapshots (create), GET /v1/snapshots/:id (get)

3. **Error Mapper**
   * ServiceError → HTTP status code + JSON error body

### Agent MCP (agent-mcp crate)

1. **MCP Server**
   * Implements MCP protocol (JSON-RPC over stdio/HTTP)
   * Registers tool definitions (core_authorize_command, core_propose_command, etc.)

2. **Tool Handlers**
   * `core_authorize_command`: Authorize a command
   * `core_propose_command`: Propose a command
   * `core_query_audit`: Query audit log
   * `core_create_snapshot`: Create snapshot
   * `core_list_resources`: List catalog resources

3. **Error Mapper**
   * ServiceError → MCP error response

### Operator CLI (operator-cli crate)

1. **Command Parser (clap)**
   * Defines CLI structure (commands, subcommands, flags)
   * Parses argv into structured commands

2. **Command Handlers**
   * `session issue`: Issue session
   * `command propose`: Propose command
   * `audit query`: Query audit log
   * `snapshot create`: Create snapshot
   * `catalog list`: List resources

3. **Output Formatter**
   * Text output (default)
   * JSON output (--json flag)
   * Table output (--table flag)

**Data Flow:**
* HTTP Request → Axum Router → Handler → Service Layer → Handler → HTTP Response
* MCP Tool Call → MCP Server → Handler → Service Layer → Handler → MCP Response
* CLI Command → clap Parser → Handler → Service Layer → Handler → CLI Output

---

## 6. Key Concepts

* **Transport Layer**: Protocol adapter (HTTP, MCP, CLI) with no business logic
* **Thin Adapter**: Minimal logic - parse protocol, call service, format response
* **Protocol Translation**: Convert between protocol types and service types
* **Error Mapping**: Convert service errors to protocol-specific error representations
* **Authentication Extraction**: Pull credentials/tokens from protocol headers/flags
* **Unified Service Layer**: All three transports call the same service methods

---

## 7. Success Criteria

**Correctness:**
* All service methods are exposed through at least one transport
* Protocol parsing is correct (no malformed requests accepted)
* Error mapping is consistent (same service error → same protocol error)
* Authentication is enforced on all endpoints/tools/commands

**Performance:**
* API request latency p99 < 10ms (excluding service layer time)
* MCP tool invocation latency p99 < 5ms (excluding service layer time)
* CLI command startup latency < 100ms

**Reliability:**
* Transport failures do not corrupt state (service layer handles rollback)
* Invalid requests return clear error messages
* All transport layers handle same load (no bottlenecks)

**Security:**
* Transport layers validate all inputs (no injection attacks)
* Authentication is mandatory (no anonymous operations)
* Authorization is checked via service layer (no bypass)

---

## 8. Dependencies

The API/MCP/CLI surfaces depend on:

* **Control Service**: All business logic delegated to service layer
* **Axum**: HTTP server framework for control-api
* **Tokio**: Async runtime for API/MCP servers
* **clap**: CLI parsing for operator-cli
* **serde**: JSON serialization/deserialization
* **tracing**: Observability (logs, traces)

---

## 9. Consumers

The API/MCP/CLI surfaces are consumed by:

* **Web Frontend**: Calls REST API for UI interactions
* **AI Agents**: Invoke MCP tools for autonomous operations
* **Operators**: Use CLI for administration and debugging
* **External Systems**: Call REST API for integration (SIEM, monitoring, CI/CD)
* **Scripts**: Use CLI in automation scripts

---

## 10. Production Readiness Considerations

**Data Integrity:**
* Transport layers do not mutate state directly (all through service layer)
* Request parsing validates all inputs (no malformed data)
* Error responses are consistent (no silent failures)

**Failure Modes:**
* **Invalid Request**: Return 400 Bad Request / MCP error / CLI error with clear message
* **Service Layer Failure**: Propagate error to client (500 Internal Server Error)
* **Authentication Failure**: Return 401 Unauthorized / MCP auth error / CLI exit code 1

**Observability:**
* Metrics: request latency (p50/p95/p99), error rates, authentication failures, endpoint usage
* Logs: every request (method, path, status, latency), every authentication attempt
* Traces: distributed tracing from transport through service layer

**Scalability:**
* Horizontal scaling: stateless transport layers can be load-balanced
* Vertical scaling: more CPU for request parsing, more connections to service layer
* Connection pooling: reuse database connections across requests

---

## 11. Risk Profile

**Risk 1: Injection Attacks (CRITICAL)**
* **Scenario**: Transport layer accepts malformed input, causes SQL injection or command injection
* **Impact**: Data breach, unauthorized access, system compromise
* **Mitigation**: Input validation, parameterized queries in service layer, fuzz testing

**Risk 2: Authentication Bypass (CRITICAL)**
* **Scenario**: Endpoint/tool/command accessible without authentication
* **Impact**: Unauthorized operations, security breach
* **Mitigation**: Middleware enforcement, authentication tests for every endpoint/tool/command

**Risk 3: Layer Violation (CRITICAL)**
* **Scenario**: Transport layer bypasses service layer to access database/policy/audit directly
* **Impact**: Authorization bypass, audit gaps, data corruption
* **Mitigation**: Code review, architectural tests, dependency constraints

**Risk 4: Inconsistent Error Handling (MEDIUM)**
* **Scenario**: Same service error returns different HTTP codes or messages
* **Impact**: Client confusion, incorrect retry logic
* **Mitigation**: Centralized error mapping, error response tests

**Risk 5: DoS via Malformed Requests (MEDIUM)**
* **Scenario**: Large or malformed requests cause parser to hang or panic
* **Impact**: API unavailability, resource exhaustion
* **Mitigation**: Request size limits, timeouts, fuzz testing

---

## 12. Open Questions (If Any)

1. **API Versioning**: Should API use URL versioning (/v1/, /v2/) or header versioning?
   * Decision needed by: API Architect (Marcus) + CTO (Rena)

2. **MCP Protocol Version**: Which MCP protocol version to support (stable vs. latest)?
   * Decision needed by: API Architect (Marcus) + Backend Engineer (Dmitri)

3. **CLI Configuration**: Should CLI config be in ~/.realmforge/config.toml or environment variables?
   * Decision needed by: API Architect (Marcus) + Infrastructure Architect (Nadia)

4. **Rate Limiting**: Should rate limiting be in transport layer or service layer?
   * Decision needed by: Security Architect (Fatima) + API Architect (Marcus)

5. **Batch Operations**: Should API/MCP/CLI support batch requests (multiple commands in one call)?
   * Decision needed by: API Architect (Marcus) + CTO (Rena)

---

## Appendix: Timeline and Status

* **Document Created:** 2025-01-XX
* **Last Updated:** 2025-01-XX
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Marcus Webb (API Architect) - Protocol design and API contracts
  * [ ] Dmitri Volkov (Backend Engineer) - Transport implementation
  * [ ] Rena Okafor (CTO) - Architecture alignment with layered system

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**END OF API/MCP/CLI SURFACES AREA CONTEXT**
