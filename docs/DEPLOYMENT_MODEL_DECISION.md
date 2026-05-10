# RealmForge Deployment Model - Strategic Decision

**Date:** 2025-01-XX  
**Decision Authority:** User (Product Owner)  
**Status:** ✅ **CLOSED** - Strategic direction confirmed  
**Impact:** Critical - defines entire deployment architecture

---

## The Decision

**RealmForge will support a hybrid deployment model with universal tracking:**

1. **Multi-Target Deployment**
   * Deploy to RealmForge-managed infrastructure ("the realm")
   * Deploy to customer's own cloud (AWS is primary target - user specified)
   * Single unified deployment automation for both targets

2. **Universal Tracking**
   * Track app state regardless of hosting location
   * Centralized observability for all deployments
   * Unified cost tracking across environments

3. **Automation First**
   * Deployment must be easy (low friction)
   * Deployment must be automated (no manual steps)
   * Same deployment workflow regardless of target

---

## User's Requirements (Direct Quote)

> "For the deployment model we want the ability to track the app wherever it has been deployed whether that is within the realm or customer own. The deployment should be easy and automated."

> "User's cloud provider is AWS."

**Key Points:**
* **Hybrid hosting:** Both "the realm" (RealmForge-managed) AND customer's own infrastructure
* **Universal tracking:** Track apps WHEREVER they're deployed
* **Easy:** Low friction, clear UX
* **Automated:** No manual steps
* **Customer cloud = AWS:** Primary BYOC target is AWS (Azure/GCP future)

---

## What This Means

### User Experience

**For apps deployed to "the realm" (RealmForge-managed):**
```
User: "Deploy my app"
RealmForge: Provisions infrastructure, deploys app, configures DNS, issues SSL
User: Gets URL, app is live at https://myapp.realm.io
RealmForge: Tracks all metrics, costs, health natively
```

**For apps deployed to customer's AWS (BYOC):**
```
User: "Deploy my app to my AWS account"
RealmForge: "Please provide AWS credentials (IAM role or access keys)"
Policy: Validates customer credentials, scopes to least privilege
RealmForge: Generates IaC, provisions via customer's AWS APIs
App: Runs in customer's AWS account (ECS/EKS/Lambda/EC2)
RealmForge: Embeds monitoring agent, tracks state/health/costs via AWS APIs
User: Gets URL, app is live at https://myapp.customer.com
RealmForge: Shows same unified dashboard for both realm and AWS apps
```

---

## Architecture Implications

**This is MORE complex than single-target deployment:**

### Must Support

1. **Multiple Hosting Targets**
   * RealmForge-managed infrastructure ("the realm")
   * Customer's AWS account (ECS, EKS, Fargate, Lambda, EC2)
   * (Future: Azure, GCP)

2. **Heterogeneous Observability**
   * **Realm apps:** Native monitoring via RealmForge infrastructure
   * **AWS apps:** Agent-based monitoring + AWS CloudWatch integration
   * **Unified dashboard:** Same UI/UX regardless of hosting location

3. **Multi-Source Cost Tracking**
   * **Realm apps:** Direct cost calculation (RealmForge owns infrastructure)
   * **AWS apps:** Pull from AWS Cost Explorer API (customer grants read access)
   * **Unified cost dashboard:** Aggregated view across all environments

4. **Flexible Deployment Routing**
   * User selects target: Realm or AWS
   * Same app artifact can deploy to either target
   * Can migrate: realm → AWS or AWS → realm

5. **Security & Permissions**
   * **Realm:** RealmForge has full control (managed service)
   * **AWS:** Least privilege IAM roles, customer-controlled, revocable
   * **Credential management:** Secure storage of customer AWS credentials

---

## Comparison to Original Options

### Original Option A: Artifact Generation Only
**Scope:** Generate Docker, Terraform, users deploy manually  
**Timeline:** 4-6 weeks  
**User Decision:** ❌ **Rejected** - "deployment should be... automated" (not manual)

### Original Option B: BYOC Deployment Only
**Scope:** Deploy to customer's cloud with automation  
**Timeline:** 4-8 weeks design + 12-20 weeks implementation  
**User Decision:** 🟡 **Partial** - Yes to AWS BYOC, but also want realm hosting

### Original Option C: RealmForge Managed Hosting Only
**Scope:** Multi-tenant hosting platform  
**Timeline:** 6+ months  
**User Decision:** 🟡 **Partial** - Yes to realm hosting, but also want AWS BYOC

### **NEW: Option D - Hybrid Model**
**Scope:** Both realm hosting AND AWS BYOC, with universal tracking  
**Timeline:** 6-10 weeks design + 16-24 weeks implementation (22-34 weeks total)  
**User Decision:** ✅ **SELECTED** - This matches user's requirements exactly

---

## Why Hybrid Is More Complex

**Single-target (Option B or C alone):**
* One cloud provider adapter
* One cost model
* One monitoring approach
* One deployment workflow

**Hybrid (Option D - user's choice):**
* Multiple cloud provider adapters (realm + AWS, future Azure/GCP)
* Heterogeneous cost models (direct calculation + AWS Cost Explorer)
* Multi-mode monitoring (native + agent-based)
* Flexible deployment routing (target selection)
* Credential management (secure storage of customer AWS creds)
* **BUT:** Single unified user experience

---

## The 8 Missing Components - Updated for Hybrid Model

### 1. Deployment Architecture Designer
**Purpose:** Visual canvas for designing hosting architecture

**Hybrid Requirements:**
* ✅ Select deployment target: **Realm** OR **AWS** (primary BYOC target)
* ✅ If AWS: Specify region, account, deployment mode (ECS/EKS/Fargate/Lambda/EC2)
* ✅ Design resource topology: compute, database, storage, networking
* ✅ Estimate costs for BOTH scenarios (realm direct, AWS Cost Explorer simulation)
* ✅ Multi-environment strategy (dev in realm, prod in AWS OR vice versa)

**Implementing Areas:** `frontend` (UI canvas), `control-plane` (orchestration)

---

### 2. Infrastructure Provisioner
**Purpose:** Provision cloud resources via IaC or APIs

**Hybrid Requirements:**
* ✅ **Realm Provisioner** - Direct API calls to RealmForge infrastructure (Kubernetes cluster, databases, storage)
* ✅ **AWS Provisioner** - Generate Terraform OR CloudFormation, execute via customer's AWS credentials
  * Support: ECS, EKS, Fargate, Lambda, EC2, RDS, S3, VPC, IAM
  * Idempotent (can re-run safely)
  * Rollback on failure (clean up partial provisioning)
* ✅ **Credential Management** - Securely store customer AWS IAM role or access keys
* ✅ **Least Privilege** - Scope customer credentials to minimum permissions needed

**Implementing Areas:** `deployment-engine`, `hosting-adapter`, `policy` (credential storage/validation)

---

### 3. Dependency Installer
**Purpose:** Install databases, runtimes, dependencies

**Hybrid Requirements:**
* ✅ **Realm deployments:** Direct installation via RealmForge infrastructure
* ✅ **AWS deployments:** Use AWS managed services (RDS, ElastiCache, etc.) OR provision on EC2
* ✅ Dependency graph resolution (app needs PostgreSQL → provision RDS PostgreSQL)
* ✅ Version compatibility (app requires Postgres 15 → provision compatible version)
* ✅ Automated scripts (Ansible, cloud-init, Terraform modules)

**Implementing Area:** `deployment-engine`

---

### 4. Build System
**Purpose:** Create deployable artifacts in multiple formats

**Hybrid Requirements:**
* ✅ **Docker images** (for realm K8s, AWS ECS/EKS/Fargate)
* ✅ **Serverless packages** (for AWS Lambda, future Azure Functions, GCP Cloud Functions)
* ✅ **VM images** (AMI for AWS EC2 if user prefers VMs)
* ✅ **RealmForge native bundle** (optimized for realm deployments)
* ✅ Multi-architecture builds (x86, ARM for AWS Graviton)
* ✅ **Artifact registry:** ECR (for AWS) or RealmForge registry (for realm)

**Implementing Area:** `infrastructure`, `deployment-engine`

---

### 5. Deployment Orchestrator
**Purpose:** Deploy app to target environment with health checks and rollback

**Hybrid Requirements:**
* ✅ **Realm deployment** - Direct orchestration via RealmForge control plane (K8s apply)
* ✅ **AWS deployment** - Deploy via AWS APIs:
  * ECS: Create/update service
  * EKS: kubectl apply via AWS API
  * Fargate: ECS Fargate mode
  * Lambda: Upload function code, update config
  * EC2: Provision instances, install app
* ✅ **Blue-green deployments** (both realm and AWS)
* ✅ **Health checks** (HTTP endpoint checks, readiness/liveness probes)
* ✅ **Rollback mechanisms** (snapshot-based for both targets)
* ✅ **Multi-region support** (future: deploy to multiple AWS regions)

**Implementing Area:** `deployment-engine`, `control-plane`

---

### 6. Configuration Manager
**Purpose:** Manage environment-specific config and secrets

**Hybrid Requirements:**
* ✅ **Realm deployments:** Native RealmForge secrets vault (encrypted PostgreSQL? HashiCorp Vault?)
* ✅ **AWS deployments:** Use AWS Secrets Manager (customer's account)
* ✅ Environment-specific config (dev/staging/prod configs)
* ✅ Feature flags (same flags work in realm or AWS)
* ✅ Runtime injection (no secrets baked into Docker images)

**Implementing Area:** `environment-manager`, `policy`

---

### 7. Monitoring & Alerting
**Purpose:** Health checks, metrics, logs, alerts, cost tracking

**Hybrid Requirements:**
* ✅ **Realm deployments:** Native RealmForge observability
  * Metrics: Prometheus/Grafana (or cloud-native)
  * Logs: Centralized logging (ELK? Loki?)
  * Traces: Distributed tracing (Jaeger? Tempo?)
* ✅ **AWS deployments:** Agent-based monitoring
  * **RealmForge monitoring agent SDK** embedded in app (Rust? Python? Multi-language?)
  * Agent reports metrics/logs/traces to RealmForge observability API
  * OR: RealmForge pulls from AWS CloudWatch via customer credentials
* ✅ **Unified dashboard:** Same UI for realm and AWS apps
* ✅ **Cost tracking:**
  * Realm: Direct calculation (RealmForge knows resource costs)
  * AWS: Pull from AWS Cost Explorer API (customer grants `ce:GetCostAndUsage`)
* ✅ **Alerting:** Unified alerting rules regardless of hosting location
* ✅ **Health checks:** HTTP endpoint monitoring, synthetic checks

**Implementing Areas:** `infrastructure` (observability platform), `audit` (event log), `deployment-engine` (agent SDK)

---

### 8. User Access Manager
**Purpose:** User onboarding, DNS, SSL, documentation

**Hybrid Requirements:**
* ✅ **Realm deployments:** RealmForge manages everything
  * DNS: myapp.realm.io (RealmForge-managed domain)
  * SSL: Auto-provision via Let's Encrypt
  * User provisioning: RealmForge user management
* ✅ **AWS deployments:** Flexible options
  * DNS Option A: Customer manages their own DNS (myapp.customer.com)
  * DNS Option B: RealmForge assists with Route53 automation (if customer delegates)
  * SSL: Let's Encrypt via ACME protocol (works for both)
* ✅ **Custom domains:** Support custom domains for both realm and AWS
* ✅ **User documentation:** Auto-generate docs with deployment-specific URLs

**Implementing Areas:** `frontend` (user onboarding UI), `control-plane`, `deployment-engine`

---

## Observability Standards for Deployed Apps

### Overview

**CRITICAL REQUIREMENT:** Every app deployed via RealmForge (realm OR AWS) MUST implement standard observability interfaces.

**Purpose:**
* Enable proactive issue detection before user impact
* Provide consistent monitoring regardless of hosting location
* Support automated alerting and incident response
* Allow RealmForge monitoring agent SDK to collect standardized telemetry

**Enforcement:**
* Build system validates observability endpoints exist before deployment
* Health checks fail if required endpoints don't respond correctly
* Deployment orchestrator refuses to promote apps without observability compliance

---

### 1. Mandatory Health Check Endpoints

Every app MUST expose these HTTP endpoints:

#### `/health` - Overall Health Status
**Purpose:** Is the app responding at all?

**Response Format:**
```json
{
  "status": "healthy" | "degraded" | "unhealthy",
  "timestamp": "2025-01-15T10:30:00Z",
  "version": "1.2.3",
  "uptime_seconds": 3600
}
```

**Status Codes:**
* `200` - App is healthy
* `503` - App is unhealthy (503 Service Unavailable)

**Used By:**
* Load balancers (ECS, ALB, K8s) for routing decisions
* RealmForge dashboard for overall status indicator
* Alerting system for critical failure detection

---

#### `/ready` - Readiness Check
**Purpose:** Is the app ready to accept traffic?

**Response Format:**
```json
{
  "ready": true | false,
  "timestamp": "2025-01-15T10:30:00Z",
  "checks": {
    "database": "connected",
    "cache": "connected",
    "external_api": "connected"
  },
  "dependencies_ready": 3,
  "dependencies_total": 3
}
```

**Status Codes:**
* `200` - App is ready
* `503` - App is NOT ready (still initializing, or dependency down)

**Used By:**
* K8s readiness probe (don't route traffic until ready)
* ECS health checks
* Blue-green deployment cutover (wait for ready before switching)

---

#### `/live` - Liveness Check
**Purpose:** Should we restart this instance?

**Response Format:**
```json
{
  "alive": true | false,
  "timestamp": "2025-01-15T10:30:00Z",
  "last_successful_operation": "2025-01-15T10:29:55Z",
  "error_count_last_minute": 0
}
```

**Status Codes:**
* `200` - App is alive
* `500` - App is NOT alive (deadlocked, OOM, unrecoverable state)

**Used By:**
* K8s liveness probe (restart pod if this fails)
* ECS task replacement
* Auto-recovery workflows

---

### 2. Metrics Endpoint

#### `/metrics` - Prometheus Format
**Purpose:** Expose metrics for collection by monitoring systems

**Format:** Prometheus exposition format (text-based)

**Required Metrics:**

**Request Metrics:**
```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",endpoint="/api/users",status="200"} 1543
http_requests_total{method="POST",endpoint="/api/orders",status="201"} 892

# HELP http_request_duration_seconds HTTP request latency
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{method="GET",endpoint="/api/users",le="0.1"} 1200
http_request_duration_seconds_bucket{method="GET",endpoint="/api/users",le="0.5"} 1500
http_request_duration_seconds_sum{method="GET",endpoint="/api/users"} 87.3
http_request_duration_seconds_count{method="GET",endpoint="/api/users"} 1543
```

**System Metrics:**
```
# HELP process_cpu_seconds_total Total CPU time
# TYPE process_cpu_seconds_total counter
process_cpu_seconds_total 127.5

# HELP process_resident_memory_bytes Resident memory size in bytes
# TYPE process_resident_memory_bytes gauge
process_resident_memory_bytes 134217728

# HELP process_open_fds Number of open file descriptors
# TYPE process_open_fds gauge
process_open_fds 42
```

**Business Metrics (App-Specific):**
```
# HELP orders_created_total Total orders created
# TYPE orders_created_total counter
orders_created_total 892

# HELP active_users Current active users
# TYPE active_users gauge
active_users 47
```

**Used By:**
* RealmForge monitoring agent SDK (scrapes every 60s)
* Prometheus/Grafana (realm deployments)
* AWS CloudWatch (custom metrics)

---

### 3. Structured Logging Standard

**Format:** JSON Lines (one JSON object per line)

**Required Fields:**
```json
{
  "timestamp": "2025-01-15T10:30:00.123Z",
  "level": "INFO" | "WARN" | "ERROR" | "DEBUG",
  "message": "User logged in successfully",
  "app_id": "myapp",
  "app_version": "1.2.3",
  "environment": "production",
  "trace_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "span_id": "1234567890abcdef",
  "user_id": "user-12345",
  "request_id": "req-67890",
  "duration_ms": 123,
  "status_code": 200,
  "error": null,
  "context": {
    "user_agent": "Mozilla/5.0",
    "ip_address": "192.168.1.1"
  }
}
```

**Log Levels:**
* `DEBUG` - Detailed diagnostic info (disabled in production by default)
* `INFO` - Normal operational events
* `WARN` - Warning conditions (degraded performance, recoverable errors)
* `ERROR` - Error conditions (failed requests, caught exceptions)
* `FATAL` - Critical errors that cause app termination

**Best Practices:**
* Log to STDOUT/STDERR (container-native logging)
* Include `trace_id` and `span_id` for distributed tracing correlation
* Include `user_id` or `session_id` for user-specific debugging
* Include `duration_ms` for performance analysis
* Use consistent field names across all apps

**Used By:**
* RealmForge monitoring agent SDK (ships logs to central aggregator)
* ELK/Loki (realm deployments)
* AWS CloudWatch Logs (AWS deployments)
* Log analysis and alerting rules

---

### 4. Distributed Tracing Integration

**Standard:** OpenTelemetry (OTEL)

**Required:**
* Propagate trace context via HTTP headers (`traceparent`, `tracestate`)
* Create spans for all significant operations (HTTP requests, DB queries, external API calls)
* Annotate spans with relevant metadata

**Trace Context Propagation:**
```
HTTP Request Headers:
traceparent: 00-a1b2c3d4e5f67890abcdef1234567890-1234567890abcdef-01
tracestate: realmforge=user-12345
```

**Span Creation (Python example):**
```python
from opentelemetry import trace

tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("process_order") as span:
    span.set_attribute("order.id", order_id)
    span.set_attribute("order.amount", 99.99)
    
    # Do work
    result = process_order(order_id)
    
    span.set_attribute("order.status", result.status)
```

**Used By:**
* RealmForge monitoring agent SDK (exports traces to OTLP endpoint)
* Jaeger/Tempo (realm deployments)
* AWS X-Ray (AWS deployments)
* Distributed request flow visualization

---

### 5. Monitoring API Endpoints

**Purpose:** Endpoints specifically for monitoring and diagnostics (NOT for app functionality)

#### `/api/monitoring/status` - Detailed Status
**Purpose:** Comprehensive status report for monitoring dashboard

**Response:**
```json
{
  "app": {
    "name": "myapp",
    "version": "1.2.3",
    "environment": "production",
    "deployed_at": "2025-01-10T08:00:00Z",
    "uptime_seconds": 432000
  },
  "health": {
    "status": "healthy",
    "checks": {
      "database": { "status": "healthy", "latency_ms": 5 },
      "cache": { "status": "healthy", "latency_ms": 2 },
      "external_api": { "status": "degraded", "latency_ms": 1500, "error": "High latency" }
    }
  },
  "performance": {
    "requests_per_second": 47.3,
    "avg_response_time_ms": 123,
    "p95_response_time_ms": 450,
    "p99_response_time_ms": 890,
    "error_rate_percent": 0.2
  },
  "resources": {
    "cpu_percent": 45.2,
    "memory_used_mb": 512,
    "memory_total_mb": 2048,
    "disk_used_percent": 32.1,
    "open_connections": 42
  }
}
```

---

#### `/api/monitoring/dependencies` - Dependency Health
**Purpose:** Track health of all dependencies

**Response:**
```json
{
  "dependencies": [
    {
      "name": "postgres",
      "type": "database",
      "status": "healthy",
      "latency_ms": 5,
      "last_checked": "2025-01-15T10:30:00Z"
    },
    {
      "name": "redis",
      "type": "cache",
      "status": "healthy",
      "latency_ms": 2,
      "last_checked": "2025-01-15T10:30:00Z"
    },
    {
      "name": "external_payment_api",
      "type": "external_api",
      "status": "degraded",
      "latency_ms": 1500,
      "last_checked": "2025-01-15T10:30:00Z",
      "error": "High latency detected"
    }
  ],
  "healthy_count": 2,
  "degraded_count": 1,
  "unhealthy_count": 0
}
```

---

#### `/api/monitoring/errors` - Recent Errors
**Purpose:** Last N errors for quick debugging

**Response:**
```json
{
  "errors": [
    {
      "timestamp": "2025-01-15T10:29:55Z",
      "level": "ERROR",
      "message": "Failed to process order",
      "trace_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "user_id": "user-12345",
      "error_type": "DatabaseConnectionError",
      "stack_trace": "..."
    }
  ],
  "total_errors_last_hour": 5,
  "error_rate_last_hour": 0.2
}
```

---

### 6. Alert-Friendly Metrics

**Purpose:** Metrics designed for alerting rules

**Examples:**

**Error Rate Above Threshold:**
```
# Alert if error rate > 5% for 5 minutes
expr: (rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])) > 0.05
```

**High Latency:**
```
# Alert if p99 latency > 1 second for 5 minutes
expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 1.0
```

**Dependency Down:**
```
# Alert if database check fails
expr: realmforge_dependency_status{name="postgres"} == 0
```

**Memory Leak Detection:**
```
# Alert if memory grows >10% per hour for 3 hours
expr: increase(process_resident_memory_bytes[1h]) > 104857600  # 100MB increase per hour
```

---

### 7. RealmForge Monitoring Agent SDK

**Purpose:** Standard library embedded in every deployed app

**Capabilities:**
* Auto-instruments HTTP frameworks (Flask, FastAPI, Express, etc.)
* Collects standard metrics (requests, latency, errors, system resources)
* Ships logs to RealmForge observability API
* Exports traces to OTLP endpoint
* Reports app metadata (version, environment, uptime)

**Configuration (via environment variables):**
```bash
REALMFORGE_AGENT_ENABLED=true
REALMFORGE_AGENT_API_URL=https://observability.realm.io/api/v1/telemetry
REALMFORGE_AGENT_API_KEY=<app-specific-key>
REALMFORGE_APP_ID=myapp
REALMFORGE_APP_VERSION=1.2.3
REALMFORGE_ENVIRONMENT=production
REALMFORGE_DEPLOYMENT_TARGET=aws  # or "realm"
```

**Python Example:**
```python
from realmforge_agent import RealmForgeAgent

# Initialize agent (reads env vars)
agent = RealmForgeAgent()

# Auto-instrument Flask app
from flask import Flask
app = Flask(__name__)
agent.instrument_flask(app)

# Manual span creation
with agent.tracer.start_span("custom_operation") as span:
    # Do work
    pass
```

**Rust Example:**
```rust
use realmforge_agent::RealmForgeAgent;

fn main() {
    let agent = RealmForgeAgent::from_env().unwrap();
    agent.init();  // Sets up tracing, metrics, log shipping
    
    // Your app code
}
```

---

### 8. Enforcement During Deployment

**Build Phase:**
* Build system checks for `/health`, `/ready`, `/live` endpoints in app code
* Fails build if required endpoints missing
* Validates metrics endpoint returns Prometheus format
* Checks logging is structured JSON

**Deployment Phase:**
* Deployment orchestrator calls `/health` before marking deployment successful
* Waits for `/ready` to return 200 before routing traffic
* Fails deployment if health checks don't pass within timeout (5 minutes)

**Runtime Phase:**
* Load balancer (ECS/ALB/K8s) continuously monitors `/health` and `/ready`
* Removes unhealthy instances from rotation
* Restarts instances if `/live` fails repeatedly
* RealmForge agent SDK reports metrics every 60 seconds
* Alerting rules fire on metric thresholds

---

## Observability Benefits

**Proactive Issue Detection:**
* Alert on error rate spike before users complain
* Detect memory leaks before OOM kills
* Identify slow dependencies before timeout errors

**Faster Debugging:**
* Trace ID links logs, metrics, traces for single request
* Dependency health shows which external service is slow
* Recent errors endpoint shows last failures immediately

**Cost Optimization:**
* Track resource usage per app
* Identify over-provisioned instances
* Forecast cost based on traffic patterns

**Unified Experience:**
* Same monitoring dashboard for realm and AWS apps
* Same alerting rules regardless of hosting
* Same debugging workflow across all environments

---

## Updated Deployment Flows

### Flow 1: Deploy to Realm (RealmForge-Managed)

```
1. User: "Deploy my app" (default target: realm)
2. RealmForge: Validates app is ready (tests pass, design approved)
3. Deployment Architecture Designer: Uses realm hosting template (K8s, managed DB)
4. Build System: Creates Docker image, pushes to RealmForge registry
5. Infrastructure Provisioner: Provisions resources in RealmForge K8s cluster
6. Dependency Installer: Provisions managed PostgreSQL in RealmForge
7. Deployment Orchestrator: Deploys to RealmForge Kubernetes (kubectl apply)
8. Configuration Manager: Injects secrets from RealmForge secrets vault
9. Monitoring & Alerting: Sets up native Prometheus/Grafana dashboards
10. User Access Manager: Configures DNS (myapp.realm.io), issues SSL cert
11. Smoke Testing: Runs automated health checks
12. RealmForge: "Your app is live at https://myapp.realm.io"
13. Catalog: Records deployment (target: realm, region: us-west-2, cost: $X/mo direct)
```

### Flow 2: Deploy to Customer's AWS (BYOC)

```
1. User: "Deploy my app to my AWS account"
2. RealmForge: "Please provide AWS credentials"
   * Option A: IAM role ARN (cross-account assume role)
   * Option B: Access key + secret key (least privilege)
3. Policy: Validates credentials, tests permissions, stores securely
4. Deployment Architecture Designer: User selects AWS ECS Fargate, us-east-1
5. Build System: Creates Docker image, pushes to customer's ECR
6. Infrastructure Provisioner:
   * Generates Terraform (VPC, ECS cluster, Fargate service, RDS, ALB)
   * Executes Terraform via customer's AWS credentials
   * Provisions all resources in customer's AWS account
7. Dependency Installer: Provisions RDS PostgreSQL in customer's VPC
8. Deployment Orchestrator: Deploys Docker image to ECS Fargate
9. Configuration Manager: Creates secrets in customer's AWS Secrets Manager
10. Monitoring & Alerting:
   * Embeds RealmForge monitoring agent SDK in Docker image
   * Agent reports metrics/logs to RealmForge observability API
   * OR: RealmForge pulls from CloudWatch via customer credentials
11. User Access Manager: Provides DNS setup guide (or automates via Route53)
12. Smoke Testing: Runs health checks against customer's deployed app
13. RealmForge: "Your app is live at https://myapp.customer.com"
14. Catalog: Records deployment (target: AWS, region: us-east-1, account: 123456789012)
15. Cost Tracking: Pulls cost data from AWS Cost Explorer API (daily job)
16. Monitoring Agent: Continuously reports app health to RealmForge
```

---

## Universal Tracking - How It Works

**Regardless of hosting location, RealmForge Catalog tracks:**

1. **Deployment Metadata**
   * App ID, version, deployment target (realm OR AWS)
   * Cloud provider, region, account/subscription
   * Deployment timestamp, deploying user, change description
   * Infrastructure topology (resources, dependencies, connections)

2. **Application Health**
   * **Realm apps:** Native monitoring via RealmForge infrastructure
   * **AWS apps:** Agent-based monitoring via embedded SDK
     * SDK metrics: CPU, memory, request latency, error rate, throughput
     * SDK logs: Structured JSON logs sent to RealmForge API
     * SDK traces: Distributed tracing spans
   * **Unified health dashboard:** Same UI shows all apps

3. **Cost Tracking**
   * **Realm apps:** Direct cost calculation
     * RealmForge knows: # of pods, memory/CPU, database size, storage
     * Calculate cost: resource usage × internal pricing
   * **AWS apps:** Pull from AWS Cost Explorer API
     * Customer grants `ce:GetCostAndUsage` permission
     * RealmForge polls daily: "Show me costs for my app's resources"
     * Attribute costs via AWS tags (app_id, environment, etc.)
   * **Unified cost dashboard:** Aggregate realm + AWS costs

4. **Deployment History**
   * Snapshot-based rollback for both realm and AWS
   * Audit trail: who deployed what, when, to which environment
   * Deployment diff: what changed between versions (code, config, infrastructure)

5. **State Synchronization**
   * **Realm apps:** RealmForge has complete control (always in sync)
   * **AWS apps:** Periodic reconciliation
     * Poll AWS APIs: "What's the actual state of my resources?"
     * Compare to Catalog: detect drift (customer modified resources outside RealmForge)
     * Alert on drift: "App X was modified outside RealmForge"

---

## Security & Permissions

### Realm Deployments
* **Full control:** RealmForge owns the infrastructure
* **Managed service:** Users cannot SSH to VMs, cannot modify infrastructure directly
* **Logs/metrics:** Available via RealmForge UI (no CloudWatch needed)
* **Isolation:** Multi-tenant (each app in separate namespace/VPC)

### AWS Deployments (BYOC)
* **Least Privilege:** Customer provides scoped IAM role/keys
* **Required AWS permissions:**
  ```json
  {
    "Version": "2012-10-17",
    "Statement": [
      {
        "Sid": "RealmForgeDeployment",
        "Effect": "Allow",
        "Action": [
          "ecs:*", "ecr:*", "ec2:Describe*", "ec2:CreateVpc", "ec2:CreateSubnet",
          "rds:Create*", "rds:Describe*", "elasticloadbalancing:*",
          "secretsmanager:CreateSecret", "secretsmanager:PutSecretValue"
        ],
        "Resource": "*"
      },
      {
        "Sid": "RealmForgeCostTracking",
        "Effect": "Allow",
        "Action": ["ce:GetCostAndUsage"],
        "Resource": "*"
      }
    ]
  }
  ```
* **Customer controlled:** Customer can revoke RealmForge access anytime
* **Read-only monitoring:** RealmForge can read CloudWatch, cannot modify infrastructure outside deployment
* **Audit trail:** All RealmForge API calls logged in customer's CloudTrail
* **Credential storage:** Encrypted at rest in RealmForge database (AES-256? KMS?)

---

## Timeline Impact

### Original Estimates (Single-Target)
* **Option B (AWS only):** 4-8 weeks design + 12-20 weeks implementation = **16-28 weeks**
* **Option C (Realm only):** 6+ months = **24+ weeks**

### Hybrid Model (User's Choice)
* **Design: 6-10 weeks**
  * Week 1-2: Deployment architecture (realm model + AWS adapter design)
  * Week 3-4: Cloud provider adapter interfaces (realm vs. AWS abstraction)
  * Week 5-6: Hybrid observability design (native + agent-based)
  * Week 7-8: Credential management and security model
  * Week 9-10: Cost aggregation architecture (direct + AWS Cost Explorer)
* **Implementation: 16-24 weeks**
  * Week 1-4: Realm deployment (K8s, managed DB, native monitoring)
  * Week 5-8: AWS adapter (ECS/Fargate deployment, Terraform generation)
  * Week 9-12: Monitoring agent SDK (embedded in app, reports to RealmForge)
  * Week 13-16: Cost tracking (AWS Cost Explorer integration, unified dashboard)
  * Week 17-20: Configuration manager (secrets, env-specific config for both targets)
  * Week 21-24: Integration testing (deploy same app to realm vs. AWS, validate tracking)
* **Total: 22-34 weeks (5.5-8.5 months)**

**Why longer than single-target:**
* Must design TWO hosting models (realm + AWS)
* Must support TWO cost models (direct + API-sourced)
* Must support TWO monitoring approaches (native + agent-based)
* Credential management adds complexity
* State synchronization across heterogeneous environments

**Trade-off:**
* More upfront work
* BUT: Maximum flexibility for users
* Users can choose: realm (easy, managed) OR AWS (control, compliance, existing infrastructure)
* Can migrate between targets as needs change

---

## Scope Refinement - MVP vs. Future

### Must Have (MVP - Weeks 1-22)
1. ✅ **Realm deployment** - K8s-based, managed PostgreSQL, native monitoring
2. ✅ **AWS ECS Fargate deployment** - Most common AWS compute option
3. ✅ **Monitoring agent SDK** - Embedded in app, reports to RealmForge
4. ✅ **Cost tracking** - Direct (realm) + AWS Cost Explorer (AWS)
5. ✅ **Unified dashboard** - Same UI for realm and AWS apps
6. ✅ **Credential management** - Secure storage of customer AWS IAM role/keys

### Should Have (Post-MVP - Weeks 23-34)
7. 🟡 **AWS EKS deployment** - For users who want Kubernetes on AWS
8. 🟡 **AWS Lambda deployment** - Serverless option
9. 🟡 **Blue-green deployments** - Zero-downtime updates
10. 🟡 **Multi-region support** - Deploy to multiple AWS regions
11. 🟡 **RDS automatic failover** - High availability

### Could Have (Future - 6+ months out)
12. 🔵 **Azure deployment** (BYOC to Azure)
13. 🔵 **GCP deployment** (BYOC to GCP)
14. 🔵 **Multi-cloud cost optimization** - Automated cost comparison (realm vs. AWS vs. Azure)
15. 🔵 **Kubernetes adapter** - Cloud-agnostic K8s (works on any K8s cluster)

### Won't Have (Out of Scope)
* ❌ On-premises deployment (customer's own data center)
* ❌ Edge deployment (CDN edge functions)
* ❌ Multi-cloud split workloads (half on realm, half on AWS)

---

## Immediate Next Steps (Weeks 1-8)

### Week 1-2: Freeze & Design Kickoff
1. ✅ **FREEZE current implementation** (policy, audit, snapshot, catalog)
   * Communicate hybrid model decision to team
   * Halt new feature work (continue bug fixes, documentation)
2. ✅ **Design realm hosting model**
   * What is "the realm"? (Kubernetes cluster on AWS/GCP managed by RealmForge?)
   * Single-tenant or multi-tenant?
   * What regions? (Start with us-west-2, expand later?)
3. ✅ **Design AWS adapter**
   * Which AWS service for compute? (Start with ECS Fargate - easiest)
   * Terraform vs. CloudFormation? (Terraform - more flexible)
   * Which AWS services for dependencies? (RDS for DB, S3 for storage, ALB for load balancing)

### Week 3-4: Cloud Provider Abstraction
4. ✅ **Design hosting-adapter interface**
   * Abstract API: `provision()`, `deploy()`, `health_check()`, `get_cost()`, `deprovision()`
   * Two implementations: `RealmAdapter`, `AwsAdapter`
   * Future: `AzureAdapter`, `GcpAdapter`
5. ✅ **Design credential flow**
   * How does customer provide AWS credentials? (UI form? CLI command?)
   * Where stored? (Encrypted PostgreSQL column? External vault?)
   * How validated? (Test permissions before storing? Scope check?)

### Week 5-6: Observability Design
6. ✅ **Design monitoring agent SDK**
   * Language? (Rust first, Python/Node.js later for broader adoption)
   * What metrics? (CPU, memory, request latency, error rate, custom metrics)
   * How reported? (Push to RealmForge API every 60s? Batch?)
   * How authenticated? (App-specific API key? JWT?)
7. ✅ **Design unified dashboard**
   * Same UI for realm and AWS apps
   * Deployment target indicator (badge: "Realm" or "AWS us-east-1")
   * Metrics panels, logs, traces, cost chart

### Week 7-8: Cost & Security Design
8. ✅ **Design cost aggregation**
   * How to pull from AWS Cost Explorer? (Daily cron job? Real-time?)
   * How to attribute costs? (AWS tags: `app_id`, `environment`, `managed_by=realmforge`)
   * How to normalize costs? (AWS $ vs. realm $ - same currency, same UI)
9. ✅ **Design security model**
   * Least privilege IAM policy for customer AWS
   * Encryption for stored credentials (AES-256? AWS KMS?)
   * Audit trail (every AWS API call logged in Catalog)

### Week 9-10: Prototype & Validation
10. ✅ **Create deployment prototype**
    * Deploy simple "Hello World" app to realm (K8s)
    * Deploy same app to AWS ECS Fargate
    * Validate: Both show in unified dashboard
    * Validate: Both show cost tracking
    * Validate: Agent SDK works (metrics from AWS app appear in RealmForge)
11. ✅ **Stakeholder review**
    * Demo hybrid deployment to team
    * Confirm MVP scope (realm + AWS ECS Fargate first)
    * Get sign-off on 22-34 week timeline
    * Get sign-off on credential management approach

---

## Success Criteria

**The hybrid model is successful when:**

1. ✅ User can deploy the SAME app to realm OR AWS with one command
2. ✅ Both deployments appear in same unified dashboard
3. ✅ Metrics from both show in real-time (realm native, AWS via agent)
4. ✅ Costs from both show accurately (realm direct, AWS via Cost Explorer)
5. ✅ Deployment is automated (no manual Terraform, no AWS Console)
6. ✅ Deployment is easy (clear UX, helpful error messages)
7. ✅ Customer can revoke RealmForge's AWS access → app keeps running (agent still reports)
8. ✅ RealmForge can still track AWS app health via agent SDK

---

## Key Design Questions to Answer (Before Implementation)

**Before Week 9 (implementation kickoff), answer:**

1. **Realm Hosting Model**
   * What is "the realm"? (K8s on AWS managed by RealmForge? GKE on GCP?)
   * Single-tenant (each customer gets own K8s cluster) OR multi-tenant (shared cluster, isolated namespaces)?
   * What regions? (Start with us-west-2? Expand to eu-west-1, ap-southeast-1?)

2. **Credential Management**
   * How does customer provide AWS credentials? (Web form? CLI? IAM role assumption?)
   * Where stored? (PostgreSQL encrypted column? HashiCorp Vault? AWS Secrets Manager?)
   * How long cached? (Persist? Or request each time?)
   * How validated? (Test permissions before accepting? Real-time scope check?)

3. **Observability Agent**
   * Language? (Rust for performance? Python for ease? Multi-language SDKs?)
   * Metrics collected? (Standard: CPU, mem, latency, errors + custom app metrics)
   * Push vs. pull? (Agent pushes to RealmForge API? Or RealmForge pulls from agent?)
   * Authentication? (App-specific API key? JWT? mTLS?)

4. **Cost Aggregation**
   * AWS Cost Explorer polling frequency? (Daily? Hourly? Real-time via EventBridge?)
   * Cost attribution? (AWS tags? Resource naming convention?)
   * Forecasting? (Historical trends? ML model? AWS Cost Forecast API?)

5. **Deployment Target Selection**
   * How does user choose realm vs. AWS? (Dropdown in UI? CLI flag? Config file?)
   * Can user change target after first deployment? (Migrate realm → AWS? AWS → realm?)
   * Can same app deploy to BOTH simultaneously? (Multi-region: prod in AWS, staging in realm?)

---

## Conclusion

**Decision:** ✅ **Hybrid deployment model with universal tracking**

**User's Requirements:**
* Track apps wherever deployed (realm OR customer AWS)
* Deployment must be easy
* Deployment must be automated

**What This Delivers:**
* Flexibility: Users choose realm (easy, managed) OR AWS (control, compliance)
* Universal tracking: Same dashboard for all apps, regardless of hosting
* Automation: One-command deployment to either target
* Unified observability: Metrics, logs, traces, costs in single UI
* Market reach: Appeals to users who want managed (realm) AND users who want BYOC (AWS)

**Impact:**
* Increased complexity (6-10 weeks design, 16-24 weeks implementation)
* Greater flexibility and user choice
* Stronger product-market fit (serves both "easy" and "control" user segments)

**Timeline:** 22-34 weeks (5.5-8.5 months) from design start to MVP

**MVP Scope:**
* Realm deployment (K8s-based, managed DB, native monitoring)
* AWS ECS Fargate deployment (BYOC)
* Unified tracking (metrics, logs, costs)

**Next Milestone:** Complete deployment architecture design (Weeks 1-8)

**Status:** ✅ **Decision closed, design phase starting**

---

**Related Documents:**
* [COMPLETE_LIFECYCLE_ANALYSIS.md](COMPLETE_LIFECYCLE_ANALYSIS.md) - Original gap analysis
* [00_LANDSCAPE_ANALYSIS_INDEX.md](00_LANDSCAPE_ANALYSIS_INDEX.md) - Analysis index
* [REALMFORGE_CAPABILITY_MAPPING.md](REALMFORGE_CAPABILITY_MAPPING.md) - Capability gaps
