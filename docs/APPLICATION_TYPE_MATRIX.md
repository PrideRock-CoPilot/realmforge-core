# Application Type Matrix

**Purpose:** Comprehensive taxonomy of all application types that RealmForge may be asked to build, with complete requirements breakdown for each.

**Usage:** When a user request comes in, classify it using this matrix to determine which components, skills, phases, and considerations apply.

---

## Quick Reference: Application Types

| ID | Type | Frontend | Backend | Database | Auth | External APIs | Complexity |
|----|------|----------|---------|----------|------|---------------|------------|
| **01** | Static Website | ✅ | ❌ | ❌ | ❌ | ❌ | Low |
| **02** | Static Site + Forms | ✅ | ✅ (API) | ✅ | ❌ | ❌ | Low |
| **03** | Content Site (CMS) | ✅ | ✅ | ✅ | ✅ (Admin) | ❌ | Medium |
| **04** | Internal Web App | ✅ | ✅ | ✅ | ✅ (SSO) | ❌ | Medium |
| **05** | External Web App (Basic Auth) | ✅ | ✅ | ✅ | ✅ (Email/PW) | ❌ | Medium |
| **06** | External Web App (SSO) | ✅ | ✅ | ✅ | ✅ (OAuth2) | ✅ (IdP) | Medium-High |
| **07** | SaaS Application | ✅ | ✅ | ✅ | ✅ (Multi-tenant) | ✅ | High |
| **08** | API-Only Service | ❌ | ✅ | ✅ | ✅ (API keys) | ✅ | Medium |
| **09** | Microservices Architecture | ✅ | ✅ (Multiple) | ✅ (Multiple) | ✅ | ✅ | High |
| **10** | Mobile App Backend | ❌ | ✅ | ✅ | ✅ (JWT) | ✅ | Medium-High |
| **11** | Real-time Application | ✅ | ✅ (WebSocket) | ✅ | ✅ | ✅ | High |
| **12** | Data Pipeline/ETL | ❌ | ✅ (Jobs) | ✅ | ❌ | ✅ | Medium |
| **13** | AI/ML Application | ✅ | ✅ | ✅ | ✅ | ✅ (LLM) | High |
| **14** | Admin Dashboard | ✅ | ✅ | ✅ | ✅ | ❌ | Medium |
| **15** | E-commerce Platform | ✅ | ✅ | ✅ | ✅ | ✅ (Payment) | High |

---

## Type 01: Static Website

**Description:** HTML/CSS/JS site with no backend, no user accounts, no dynamic data.

**Examples:** Marketing site, portfolio, documentation site, landing page.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | HTML/CSS/JS or framework (React, Vue) |
| Backend | ❌ | None |
| Database | ❌ | None |
| Authentication | ❌ | None |
| Hosting | ✅ | Static hosting (S3, Netlify, Vercel) |

### Skills/Personas Required

* **PM (Alex):** Scope, deliverables
* **Frontend (Kai):** Build UI, ensure accessibility
* **Tech Writer (Clara):** Documentation if needed
* **DevOps (via Release Manager Sam):** Deploy to static hosting

### Workflow Phases

* ✅ Phase 0: Requirements (simple)
* ✅ Phase 1: Design/Mockup
* ⚠️ Phase 2: Technical Discovery (minimal - hosting only)
* ⚠️ Phase 3: Architecture (minimal - static site structure)
* ❌ Phase 4: Financial Review (usually cheap/free)
* ✅ Phase 5: Planning
* ❌ Phase 6: Security Review (no attack surface)
* ✅ Phase 7: Implementation
* ✅ Phase 8: Deployment
* ⚠️ Phase 9: Post-Launch (minimal monitoring)

### Deliverables

* Static HTML/CSS/JS files
* Build scripts (if using framework)
* Deployment configuration
* README with deployment instructions

### Special Considerations

* **SEO:** Meta tags, sitemap, robots.txt
* **Performance:** Image optimization, lazy loading, CDN
* **Accessibility:** WCAG compliance
* **Analytics:** Optional Google Analytics or similar

---

## Type 02: Static Site + Form Submission

**Description:** Static site with contact/signup forms that submit to a backend API.

**Examples:** Landing page with lead capture, contact forms, newsletter signup.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | HTML/CSS/JS with form handling |
| Backend | ✅ | API endpoint for form submission |
| Database | ✅ | Store form submissions |
| Authentication | ❌ | None (public forms) |
| Email Service | ✅ | Send notifications/confirmations |

### Skills/Personas Required

* **PM (Alex):** Requirements, data retention policies
* **Frontend (Kai):** Build forms with validation
* **Backend (Dmitri):** API endpoint, validation, rate limiting
* **Database Engineer:** Schema for submissions
* **Security Architect (Fatima):** CSRF protection, input validation, rate limiting
* **DevOps (Sam):** Deploy frontend + backend

### Workflow Phases

* ✅ Phase 0: Requirements (form fields, validation, notifications)
* ✅ Phase 1: Design (form UX)
* ✅ Phase 2: Technical Discovery (email service, storage)
* ✅ Phase 3: Architecture (API design)
* ⚠️ Phase 4: Financial Review (email service costs)
* ✅ Phase 5: Planning
* ✅ Phase 6: Security Review (CSRF, rate limiting, spam prevention)
* ✅ Phase 7: Implementation
* ✅ Phase 8: Deployment
* ✅ Phase 9: Post-Launch (monitor submission rates, spam)

### Deliverables

* Static frontend with forms
* Backend API endpoint
* Database schema
* Email templates
* Rate limiting configuration
* Spam filtering (reCAPTCHA or similar)

### Special Considerations

* **GDPR:** Privacy policy, data retention, consent checkboxes
* **Spam Prevention:** reCAPTCHA, rate limiting, honeypot fields
* **Data Validation:** Server-side validation (never trust client)
* **Email Deliverability:** SPF, DKIM, DMARC records

---

## Type 03: Content Management Site (CMS)

**Description:** Website with admin panel for content editing (blog, news, pages).

**Examples:** Blog platform, news site, documentation with admin editing.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Public-facing site |
| Backend | ✅ | CMS API, content delivery |
| Database | ✅ | Content storage (posts, pages, media) |
| Authentication | ✅ | Admin login only |
| File Storage | ✅ | Images, videos, documents |
| Search | ⚠️ | Optional (full-text search) |

### Skills/Personas Required

* **PM (Alex):** Content model, user roles
* **Frontend (Kai):** Public site + admin interface
* **Backend (Dmitri):** Content API, media handling
* **Database Engineer:** Content schema, indexes
* **Security Architect (Fatima):** Admin auth, RBAC, XSS prevention
* **QA (Meg):** Content workflow testing
* **DevOps (Sam):** CDN for media, backup strategy

### Workflow Phases

* ✅ All phases (full lifecycle)

### Deliverables

* Public-facing site
* Admin dashboard
* Content API
* Database schema
* Media upload/management
* User roles (admin, editor, contributor)
* Content versioning (optional)

### Special Considerations

* **Content Versioning:** Draft/publish workflow
* **Media Management:** Image resizing, CDN integration
* **SEO:** Dynamic meta tags, sitemap generation
* **Caching:** Content caching for performance
* **Backup:** Regular content backups

---

## Type 04: Internal Web Application

**Description:** Application for company employees, internal tools, admin panels.

**Examples:** Internal dashboard, employee portal, admin tools.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Internal UI |
| Backend | ✅ | Business logic, API |
| Database | ✅ | Application data |
| Authentication | ✅ | SSO (Okta, Azure AD, Google Workspace) |
| Authorization | ✅ | Role-based access control (RBAC) |

### Skills/Personas Required

* **PM (Alex):** User stories, department workflows
* **Frontend (Kai):** Internal UI (accessibility less critical)
* **Backend (Dmitri):** Business logic, API
* **Database Engineer:** Schema design
* **Security Architect (Fatima):** SSO integration, RBAC
* **Infra Architect (Nadia):** VPN, network access
* **QA (Meg):** Workflow testing

### Workflow Phases

* ✅ All phases (full lifecycle)

### Deliverables

* Internal web application
* SSO integration
* RBAC implementation
* Database schema
* Deployment to internal infrastructure
* User documentation

### Special Considerations

* **SSO Integration:** SAML, OAuth2, OpenID Connect
* **Network Access:** VPN, IP whitelisting
* **Audit Logging:** Track user actions for compliance
* **Data Privacy:** Internal data access controls
* **Uptime:** Less critical than external apps (business hours)

---

## Type 05: External Web App (Basic Auth)

**Description:** Public web application with email/password registration.

**Examples:** Productivity tool, social platform, community site.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Public UI |
| Backend | ✅ | Business logic, API |
| Database | ✅ | User accounts + app data |
| Authentication | ✅ | Email/password registration |
| Email Service | ✅ | Verification, password reset |
| Session Management | ✅ | JWT or server-side sessions |

### Skills/Personas Required

* **PM (Alex):** User flows, feature requirements
* **Biz User (Iris):** User stories, acceptance criteria
* **Frontend (Kai):** Public UI, login/signup flows
* **Backend (Dmitri):** Auth system, business logic
* **Database Engineer:** User schema, indexes
* **Security Architect (Fatima):** Password hashing, session security, OWASP Top 10
* **QA (Meg):** Security testing, user flow testing
* **DevOps (Sam):** Scalable infrastructure, monitoring

### Workflow Phases

* ✅ All phases (full lifecycle)

### Deliverables

* Public web application
* User authentication system
* Email verification workflow
* Password reset workflow
* Database schema
* Session management
* Security headers (CSP, HSTS, etc.)
* Monitoring and alerting

### Special Considerations

* **Password Security:** bcrypt/argon2, min length, complexity
* **Email Verification:** Prevent fake accounts
* **Password Reset:** Secure token-based reset
* **Session Security:** CSRF protection, secure cookies
* **GDPR/Privacy:** Terms of service, privacy policy, data export/deletion
* **Rate Limiting:** Login attempts, API calls

---

## Type 06: External Web App (OAuth2/SSO)

**Description:** Public application with "Sign in with Google/Apple/GitHub" etc.

**Examples:** Any app with social login or enterprise SSO.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Public UI with OAuth buttons |
| Backend | ✅ | OAuth2 flow, business logic |
| Database | ✅ | User accounts linked to OAuth IDs |
| Authentication | ✅ | OAuth2 (Google, Apple, GitHub, etc.) |
| Session Management | ✅ | JWT or server-side sessions |
| External API | ✅ | Identity provider (IdP) APIs |

### Skills/Personas Required

* **PM (Alex):** OAuth provider selection, user flows
* **Frontend (Kai):** OAuth button flows, account linking
* **Backend (Dmitri):** OAuth2 implementation, token management
* **Database Engineer:** User identity schema (multiple OAuth IDs)
* **Security Architect (Fatima):** OAuth2 security, token storage, account takeover prevention
* **API Architect (Marcus):** IdP API integration
* **QA (Meg):** OAuth flow testing (happy path + edge cases)
* **DevOps (Sam):** Manage OAuth app credentials (secrets)

### Workflow Phases

* ✅ All phases (full lifecycle)
* **Critical:** Phase 6 (Security Review) - OAuth2 is complex and error-prone

### Deliverables

* OAuth2 integration (Google, Apple, etc.)
* Account linking (email + OAuth)
* Token refresh logic
* User identity schema
* OAuth app registration (provider consoles)
* Security audit (OAuth flows)

### Special Considerations

* **OAuth2 Flows:** Authorization code flow (most secure)
* **Token Storage:** Secure storage, rotation
* **Account Linking:** Handle users with multiple OAuth IDs
* **Scope Management:** Request minimal permissions
* **Provider Downtime:** Graceful fallback (email/password?)
* **Privacy:** User consent for data access
* **PKCE:** Use for mobile/SPA security

---

## Type 07: SaaS Application (Multi-Tenant)

**Description:** Software-as-a-Service with multiple organizations/workspaces.

**Examples:** Project management tool, CRM, collaboration platform.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Public UI + tenant dashboard |
| Backend | ✅ | Multi-tenant business logic |
| Database | ✅ | Tenant isolation (schema or row-level) |
| Authentication | ✅ | User accounts + tenant membership |
| Authorization | ✅ | RBAC within tenants |
| Billing | ✅ | Subscription management (Stripe, Paddle) |
| Email Service | ✅ | Invitations, notifications |
| Analytics | ✅ | Usage tracking, billing metrics |

### Skills/Personas Required

* **PM (Alex):** Tenant model, pricing tiers, features
* **Biz User (Iris):** User journeys, org workflows
* **Frontend (Kai):** Tenant switching, workspace UI
* **Backend (Dmitri):** Multi-tenancy, billing integration
* **Database Engineer:** Tenant isolation strategy
* **Security Architect (Fatima):** Tenant data isolation, access control
* **Accountant (Bob):** Billing reconciliation, revenue tracking
* **QA (Meg):** Multi-tenant testing, isolation verification
* **DevOps (Sam):** Scalable infrastructure, tenant provisioning

### Workflow Phases

* ✅ All phases (full lifecycle) + extra complexity
* **Critical:** Phase 2 (Technical Discovery) - tenant isolation strategy
* **Critical:** Phase 3 (Architecture) - multi-tenant design patterns
* **Critical:** Phase 6 (Security Review) - data isolation, OWASP for multi-tenancy

### Deliverables

* Multi-tenant application
* Tenant provisioning system
* Billing integration (Stripe/Paddle)
* User invitation system
* RBAC within tenants
* Usage analytics
* Tenant data isolation (verified)
* Billing reconciliation dashboard

### Special Considerations

* **Tenant Isolation:** Database schema per tenant, shared schema with row-level security, or separate databases
* **Billing:** Subscription plans, usage-based pricing, invoicing
* **Onboarding:** Tenant provisioning, trial periods
* **Data Migration:** Tenant data import/export
* **Performance:** Ensure one tenant can't impact others (noisy neighbor)
* **Compliance:** GDPR, SOC2, HIPAA (depending on industry)

---

## Type 08: API-Only Service

**Description:** Backend service exposing REST/GraphQL API, no UI.

**Examples:** Third-party API, internal microservice, data service.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ❌ | None (API consumers build their own) |
| Backend | ✅ | API service |
| Database | ✅ | Data storage |
| Authentication | ✅ | API keys, OAuth2, JWT |
| Rate Limiting | ✅ | Prevent abuse |
| Documentation | ✅ | OpenAPI/Swagger |

### Skills/Personas Required

* **PM (Alex):** API requirements, use cases
* **API Architect (Marcus):** API design, versioning, standards
* **Backend (Dmitri):** Implement API
* **Database Engineer:** Schema design, query optimization
* **Security Architect (Fatima):** API security, rate limiting, key management
* **Tech Writer (Clara):** API documentation, examples
* **QA (Meg):** API testing (contract tests, load tests)
* **DevOps (Sam):** API gateway, monitoring, throttling

### Workflow Phases

* ✅ All phases (full lifecycle)
* **Critical:** Phase 3 (Architecture) - API design, versioning strategy
* **Critical:** Phase 6 (Security Review) - API security (OWASP API Security Top 10)

### Deliverables

* REST or GraphQL API
* API authentication (keys, OAuth2)
* Rate limiting
* OpenAPI/Swagger documentation
* API versioning strategy
* Error response standards
* API gateway configuration
* Monitoring and alerting

### Special Considerations

* **Versioning:** URL versioning, header versioning, or semantic versioning
* **Rate Limiting:** Per-key or per-IP limits
* **Documentation:** Interactive docs (Swagger UI, Postman)
* **Error Handling:** Consistent error responses (RFC 7807)
* **Idempotency:** POST/PUT/PATCH should be idempotent
* **CORS:** If accessed from browsers
* **Deprecation:** Sunset headers, deprecation notices

---

## Type 09: Microservices Architecture

**Description:** Distributed system with multiple independent services.

**Examples:** Large-scale platform, complex business application.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Gateway or BFF (Backend for Frontend) |
| Backend | ✅ | Multiple services (5-50+) |
| Database | ✅ | Per-service databases |
| Authentication | ✅ | Centralized auth service or gateway |
| Service Mesh | ✅ | Inter-service communication (Istio, Linkerd) |
| Message Queue | ✅ | Async communication (Kafka, RabbitMQ) |
| API Gateway | ✅ | Entry point, routing, rate limiting |
| Service Discovery | ✅ | Consul, Kubernetes DNS |
| Distributed Tracing | ✅ | Jaeger, Zipkin |

### Skills/Personas Required

* **CEO (Victor):** Investment approval (high cost/complexity)
* **PM (Alex):** Service decomposition, team coordination
* **CTO (Rena):** Architecture decisions, service boundaries
* **Domain Architect (Yusuf):** Bounded contexts, domain modeling
* **API Architect (Marcus):** Inter-service contracts, versioning
* **Backend (Dmitri):** Implement services
* **Data Architect (Chen):** Data consistency, event sourcing
* **Infra Architect (Nadia):** Kubernetes, service mesh, observability
* **Security Architect (Fatima):** Zero-trust networking, mTLS
* **QA (Meg):** Contract testing, chaos engineering
* **DevOps (Sam):** CI/CD pipelines per service, deployment orchestration

### Workflow Phases

* ✅ All phases (full lifecycle) + extensive planning
* **Critical:** Phase 2 (Technical Discovery) - service boundaries, data consistency
* **Critical:** Phase 3 (Architecture) - service contracts, event schemas
* **Critical:** Phase 6 (Security Review) - inter-service auth, network policies

### Deliverables

* Multiple microservices
* API gateway configuration
* Service mesh setup
* Message queue infrastructure
* Service discovery
* Distributed tracing
* Centralized logging
* CI/CD pipelines per service
* Service contracts (OpenAPI)
* Deployment strategies (blue-green, canary)

### Special Considerations

* **Service Boundaries:** Domain-driven design
* **Data Consistency:** Eventual consistency, saga pattern
* **Inter-Service Communication:** Sync (HTTP/gRPC) vs async (message queue)
* **Failure Handling:** Circuit breakers, retries, timeouts
* **Observability:** Distributed tracing, centralized logging
* **Deployment Complexity:** Coordinated releases, version compatibility
* **Cost:** High infrastructure and operational costs

---

## Type 10: Mobile App Backend

**Description:** Backend API for iOS/Android mobile apps.

**Examples:** Mobile app backend, hybrid app backend.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ❌ | Mobile app (separate team/project) |
| Backend | ✅ | REST or GraphQL API |
| Database | ✅ | User data, app content |
| Authentication | ✅ | JWT, refresh tokens |
| Push Notifications | ✅ | FCM (Android), APNs (iOS) |
| File Storage | ✅ | Images, videos, documents |
| Offline Sync | ⚠️ | Optional (complex) |

### Skills/Personas Required

* **PM (Alex):** Mobile UX flows, offline behavior
* **API Architect (Marcus):** Mobile-friendly API design
* **Backend (Dmitri):** Implement API, push notifications
* **Database Engineer:** Schema design, sync strategy
* **Security Architect (Fatima):** Token security, certificate pinning
* **QA (Meg):** Mobile API testing, offline scenarios
* **DevOps (Sam):** Scalable backend, CDN for media

### Workflow Phases

* ✅ All phases (full lifecycle)
* **Critical:** Phase 3 (Architecture) - offline sync, push notifications

### Deliverables

* Mobile-optimized API
* JWT authentication + refresh tokens
* Push notification service
* File upload/download
* Offline sync strategy (if needed)
* API versioning (for app store updates)
* Monitoring and alerting

### Special Considerations

* **Token Security:** Short-lived access tokens, long-lived refresh tokens
* **Certificate Pinning:** Prevent MITM attacks
* **API Efficiency:** Minimize round trips, batch operations
* **Push Notifications:** FCM, APNs integration
* **Offline Sync:** Conflict resolution, delta sync
* **App Store Updates:** API versioning, graceful degradation

---

## Type 11: Real-Time Application

**Description:** Applications with real-time updates (chat, collaboration, live data).

**Examples:** Chat app, live dashboard, collaborative editor, gaming backend.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | WebSocket client, UI updates |
| Backend | ✅ | WebSocket server, pub/sub |
| Database | ✅ | Persistent storage |
| Real-Time Messaging | ✅ | Redis Pub/Sub, Kafka, WebSocket |
| Session Stickiness | ✅ | Load balancer affinity |
| Presence | ⚠️ | Online/offline status (optional) |

### Skills/Personas Required

* **PM (Alex):** Real-time requirements, latency expectations
* **Frontend (Kai):** WebSocket client, optimistic UI updates
* **Backend (Dmitri):** WebSocket server, pub/sub
* **Infra Architect (Nadia):** Redis, Kafka, load balancing
* **QA (Meg):** Real-time scenario testing, race conditions
* **DevOps (Sam):** Scalable WebSocket infrastructure

### Workflow Phases

* ✅ All phases (full lifecycle)
* **Critical:** Phase 3 (Architecture) - message delivery guarantees, scaling strategy

### Deliverables

* WebSocket server
* Frontend WebSocket client
* Pub/sub infrastructure (Redis, Kafka)
* Presence system (optional)
* Message delivery guarantees
* Load balancer configuration
* Monitoring (connection counts, latency)

### Special Considerations

* **Message Delivery:** At-most-once, at-least-once, exactly-once
* **Scaling:** Horizontal scaling with pub/sub
* **Load Balancing:** Sticky sessions or Redis pub/sub
* **Connection Management:** Heartbeats, reconnection logic
* **Conflict Resolution:** Operational transformation (OT) or CRDT
* **Latency:** Sub-100ms for good UX

---

## Type 12: Data Pipeline / ETL

**Description:** Batch or streaming data processing, no user-facing UI.

**Examples:** ETL jobs, data warehouse pipelines, analytics processing.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ❌ | None (or admin dashboard) |
| Backend | ✅ | Batch/streaming jobs |
| Source Database | ✅ | Data source |
| Target Database | ✅ | Data warehouse, data lake |
| Orchestration | ✅ | Airflow, Dagster, Databricks Workflows |
| Monitoring | ✅ | Job success/failure, data quality |

### Skills/Personas Required

* **PM (Alex):** Data requirements, SLAs
* **Data Architect (Chen):** Data modeling, warehouse design
* **Data Engineer (Priya):** Build pipelines, transformations
* **QA (Meg):** Data quality testing
* **Infra Architect (Nadia):** Infrastructure, scheduling
* **DevOps (Sam):** Deploy pipelines, monitoring

### Workflow Phases

* ✅ Most phases (no frontend/auth)
* ❌ Phase 1: Design (no UI mockups)
* ✅ Phase 2: Technical Discovery (data sources, volumes)
* ✅ Phase 3: Architecture (pipeline design)
* ✅ Phase 5: Planning
* ⚠️ Phase 6: Security (data access, PII handling)
* ✅ Phase 7: Implementation
* ✅ Phase 8: Deployment
* ✅ Phase 9: Post-Launch (monitor data quality)

### Deliverables

* ETL pipelines
* Data transformations
* Orchestration DAGs
* Data quality checks
* Monitoring and alerting
* Documentation (data lineage)

### Special Considerations

* **Idempotency:** Re-running pipelines should be safe
* **Data Quality:** Validation checks, anomaly detection
* **Scalability:** Handle growing data volumes
* **Incremental Processing:** Avoid full table scans
* **Error Handling:** Retry logic, dead letter queues
* **Monitoring:** Data freshness, pipeline failures

---

## Type 13: AI/ML Application

**Description:** Application with embedded AI models, LLM integrations, or ML predictions.

**Examples:** Chatbot, recommendation engine, image classification, sentiment analysis.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | UI for AI interactions |
| Backend | ✅ | API, model serving |
| Database | ✅ | User data, interaction history |
| ML Models | ✅ | Trained models or LLM APIs |
| Vector Database | ⚠️ | For RAG applications |
| Model Serving | ✅ | MLflow, SageMaker, Databricks serving |
| External APIs | ✅ | OpenAI, Anthropic, HuggingFace |

### Skills/Personas Required

* **PM (Alex):** AI requirements, user experience
* **Biz User (Iris):** Use cases, acceptance criteria
* **AI/ML Engineer:** Model selection, prompt engineering, RAG
* **Backend (Dmitri):** API integration, model serving
* **Data Engineer (Priya):** Training data pipelines
* **Frontend (Kai):** AI interaction UI
* **QA (Meg):** AI testing, bias detection
* **Security Architect (Fatima):** Prompt injection, PII in prompts
* **Accountant (Bob):** LLM API cost tracking
* **DevOps (Sam):** Model deployment, monitoring

### Workflow Phases

* ✅ All phases (full lifecycle)
* **Critical:** Phase 2 (Technical Discovery) - model selection, LLM provider, cost estimation
* **Critical:** Phase 3 (Architecture) - RAG vs fine-tuning, prompt design
* **Critical:** Phase 6 (Security Review) - prompt injection, jailbreaking, PII leakage

### Deliverables

* AI-powered application
* Model serving endpoint
* Prompt templates
* RAG pipeline (if applicable)
* Evaluation metrics
* Cost tracking dashboard
* AI safety guardrails

### Special Considerations

* **Cost:** LLM API costs can be high
* **Latency:** Balance model quality vs response time
* **Prompt Injection:** Validate and sanitize user inputs
* **Hallucination:** Implement fact-checking or citations
* **Bias:** Test for fairness, bias detection
* **Privacy:** Don't send PII to external APIs
* **Fallback:** Graceful degradation if model/API fails

---

## Type 14: Admin Dashboard

**Description:** Internal tool for managing users, content, settings, analytics.

**Examples:** User management, content moderation, system monitoring, analytics dashboard.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Admin UI |
| Backend | ✅ | Admin API |
| Database | ✅ | Application data |
| Authentication | ✅ | Admin login (SSO recommended) |
| Authorization | ✅ | Admin roles and permissions |
| Audit Logging | ✅ | Track admin actions |

### Skills/Personas Required

* **PM (Alex):** Admin workflows, permissions
* **Frontend (Kai):** Admin UI (tables, forms, charts)
* **Backend (Dmitri):** Admin API, RBAC
* **Database Engineer:** Efficient queries for dashboards
* **Security Architect (Fatima):** Admin access control, audit logging
* **QA (Meg):** Admin workflow testing

### Workflow Phases

* ✅ All phases (full lifecycle)
* **Critical:** Phase 6 (Security Review) - admin access control, audit logging

### Deliverables

* Admin dashboard UI
* Admin API
* RBAC implementation
* Audit logging
* User management features
* Analytics/reporting

### Special Considerations

* **Access Control:** Multi-level admin roles
* **Audit Logging:** Log all admin actions
* **Search/Filtering:** Efficient data queries
* **Bulk Operations:** Safe bulk actions (with confirmation)
* **Export:** CSV/Excel export for reports

---

## Type 15: E-Commerce Platform

**Description:** Online store with products, cart, checkout, payments.

**Examples:** Online shop, marketplace, subscription box service.

### Components Required

| Component | Required | Details |
|-----------|----------|---------|
| Frontend | ✅ | Storefront, cart, checkout |
| Backend | ✅ | Product catalog, order management |
| Database | ✅ | Products, orders, customers |
| Authentication | ✅ | Customer accounts |
| Payment Gateway | ✅ | Stripe, PayPal, Square |
| Inventory Management | ✅ | Stock tracking |
| Email Service | ✅ | Order confirmations, shipping updates |
| Search | ✅ | Product search (Elasticsearch, Algolia) |
| Analytics | ✅ | Sales, conversions, attribution |

### Skills/Personas Required

* **CEO (Victor):** Business model, revenue projections
* **PM (Alex):** E-commerce flows, checkout optimization
* **Biz User (Iris):** Customer journeys, merchandising
* **Frontend (Kai):** Storefront UI, checkout flow
* **Backend (Dmitri):** Order processing, payment integration
* **Database Engineer:** Product catalog, order schema
* **Security Architect (Fatima):** PCI compliance, payment security
* **Accountant (Bob):** Revenue tracking, tax calculations
* **QA (Meg):** Checkout testing, payment testing
* **DevOps (Sam):** High availability, CDN for product images

### Workflow Phases

* ✅ All phases (full lifecycle) + extra complexity
* **Critical:** Phase 4 (Financial Review) - payment gateway fees, revenue projections
* **Critical:** Phase 6 (Security Review) - PCI compliance, payment security

### Deliverables

* E-commerce website
* Product catalog
* Shopping cart
* Checkout flow
* Payment gateway integration
* Order management system
* Inventory tracking
* Email notifications
* Admin dashboard
* PCI compliance documentation

### Special Considerations

* **PCI Compliance:** Never store raw credit card data
* **Inventory Management:** Stock levels, backorders
* **Shipping Integration:** Calculate shipping rates, print labels
* **Tax Calculations:** Sales tax (Avalara, TaxJar)
* **Fraud Prevention:** Address verification, 3D Secure
* **Abandoned Cart:** Recovery emails
* **Product Search:** Fast, relevant search (Elasticsearch)

---

## Application Complexity Matrix

| Complexity | Application Types | Estimated Timeline | Team Size | Critical Risks |
|------------|-------------------|-------------------|-----------|----------------|
| **Low** | 01, 02 | 1-2 weeks | 1-2 | Minimal |
| **Medium** | 03, 04, 05, 08, 12, 14 | 4-8 weeks | 2-4 | Security, scalability |
| **Medium-High** | 06, 10 | 8-12 weeks | 3-5 | OAuth security, mobile specifics |
| **High** | 07, 09, 11, 13, 15 | 3-6 months | 4-10+ | Architecture, cost, compliance |

---

## Skills/Personas Required by Application Type

| Skill/Persona | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 10 | 11 | 12 | 13 | 14 | 15 |
|---------------|----|----|----|----|----|----|----|----|----|----|----|----|----|----|-----|
| **CEO (Victor)** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **PM (Alex)** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **CTO (Rena)** | ❌ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ✅ | ⚠️ | ✅ | ⚠️ | ✅ | ⚠️ | ✅ | ❌ | ✅ |
| **Domain Architect (Yusuf)** | ❌ | ❌ | ❌ | ⚠️ | ⚠️ | ⚠️ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Security Architect (Fatima)** | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | ✅ | ✅ | ✅ |
| **API Architect (Marcus)** | ❌ | ⚠️ | ❌ | ❌ | ⚠️ | ⚠️ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ❌ | ⚠️ | ❌ | ⚠️ |
| **Infra Architect (Nadia)** | ❌ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ✅ | ⚠️ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ | ❌ | ✅ |
| **Data Architect (Chen)** | ❌ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ✅ | ⚠️ | ✅ | ❌ | ❌ | ✅ | ⚠️ | ❌ | ⚠️ |
| **Backend (Dmitri)** | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Frontend (Kai)** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ |
| **Data Engineer (Priya)** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚠️ | ❌ | ⚠️ | ❌ | ❌ | ✅ | ✅ | ❌ | ⚠️ |
| **QA (Meg)** | ⚠️ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Accountant (Bob)** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ⚠️ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| **Release Manager (Sam)** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Tech Writer (Clara)** | ⚠️ | ⚠️ | ⚠️ | ✅ | ⚠️ | ⚠️ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Biz User (Iris)** | ❌ | ❌ | ⚠️ | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ | ⚠️ | ⚠️ | ❌ | ✅ | ⚠️ | ✅ |
| **AI/ML Engineer** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| **Database Engineer** | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Legend:**
* ✅ **Required** - Must be involved
* ⚠️ **Optional** - May be needed depending on complexity
* ❌ **Not needed** - Not typically required

---

## Usage Guide

### Step 1: Classify the Request

When a user request comes in, determine which application type(s) it matches:

1. **Read the request carefully**
2. **Identify key components** (frontend, backend, auth, etc.)
3. **Match to application type ID** (01-15)
4. **If hybrid** (e.g., "web app with admin panel"), identify multiple types

### Step 2: Determine Required Components

Based on the application type:

1. **Check the Components Required table**
2. **Identify all ✅ components**
3. **Consider ⚠️ components** based on specific requirements

### Step 3: Assemble the Team

Using the Skills/Personas matrix:

1. **Include all ✅ Required skills**
2. **Consider ⚠️ Optional skills** based on complexity
3. **Invoke skills in proper order** (see workflow chain in CLAUDE.md)

### Step 4: Run Through Workflow Phases

Follow the 9-phase workflow from the earlier document:

1. **Phase 0:** Requirements
2. **Phase 1:** Design/Mockup
3. **Phase 2:** Technical Discovery
4. **Phase 3:** Architecture
5. **Phase 4:** Financial Review
6. **Phase 5:** Planning
7. **Phase 6:** Security Review
8. **Phase 7:** Implementation
9. **Phase 8:** Deployment
10. **Phase 9:** Post-Launch

Check each phase's applicability for the specific application type.

### Step 5: Document Special Considerations

For each application type, note the special considerations section and ensure they're addressed in planning.

---

## Next Steps

This matrix should be used in conjunction with:

1. **MASTER_BUILD_PLAN.md** - For RealmForge-specific build process
2. **CLAUDE.md** - For skill invocation protocol
3. **AGENTS.md** - For engineering constraints

**TODO:** Create automated classification tool that:
* Takes user request as input
* Suggests matching application type(s)
* Lists required components and skills
* Generates initial project plan
