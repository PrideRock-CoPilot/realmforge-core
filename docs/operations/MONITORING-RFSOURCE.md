---
doc_id: DOC-OPS-RFSOURCE-MONITORING-001
title: RFSource Monitoring and Alerting Strategy
status: active
version: 1.0.0
created_at: 2026-05-07
last_updated: 2026-05-07
owner: operations_team
related_docs:
  - DOC-OPS-RFSOURCE-RUNBOOK-001
  - DOC-TEST-RFSOURCE-EXTENDED-001
  - DOC-AUDIT-RFSOURCE-001
---

# RFSource Monitoring and Alerting Strategy

**Version:** 1.0.0  
**Last Updated:** 2026-05-07  
**Baseline:** 500-user extended load test (DOC-TEST-RFSOURCE-EXTENDED-001)

---

## Table of Contents

1. [Overview](#overview)
2. [Service Level Objectives (SLOs)](#service-level-objectives-slos)
3. [Metrics Collection](#metrics-collection)
4. [Alert Definitions](#alert-definitions)
5. [Dashboard Layouts](#dashboard-layouts)
6. [Monitoring Tool Setup](#monitoring-tool-setup)
7. [On-Call Procedures](#on-call-procedures)
8. [Runbook Integration](#runbook-integration)

---

## Overview

### Monitoring Philosophy

**Goals:**
1. **Early Detection** - Identify issues before user impact
2. **Root Cause Analysis** - Provide data for investigation
3. **Capacity Planning** - Track trends for scaling decisions
4. **Compliance** - Meet SLA commitments

**Approach:**
* **Symptom-based alerts** - Alert on user impact (latency, errors)
* **Cause-based metrics** - Collect for investigation (CPU, memory, disk)
* **Actionable alerts** - Every alert must have a runbook entry
* **Minimize noise** - Only alert on conditions requiring human response

### Load Test Baseline

All thresholds are derived from extended load test results:

| Metric | Baseline (500 users, 5 min) | Alert Threshold |
|--------|------------------------------|-----------------|
| Read Latency (p95) | 119.10 ms | >200 ms |
| Write Latency (p95) | 14.84 ms | >50 ms |
| Throughput | 1,282 ops/sec | <1,000 ops/sec |
| Error Rate | 0.35% | >1% |
| Peak Memory | 26.9 MB | >500 MB |
| Average CPU | 4.6% | >25% |

---

## Service Level Objectives (SLOs)

### Availability SLO

**Target:** 99.9% uptime (monthly)

* **Allowed Downtime:** 43 minutes per month
* **Measurement:** Health check endpoint returns 200 OK
* **Exclusions:** Planned maintenance windows

**Error Budget:**
* Monthly: 43 minutes
* Weekly: ~10 minutes
* Daily: ~1.4 minutes

**Burn Rate Alerts:**
* **Fast Burn (>10x):** Alert within 5 minutes (critical)
* **Moderate Burn (>3x):** Alert within 1 hour (warning)

### Latency SLO

**Read Operations:**
* **p50:** <120 ms (95% of 1-minute windows)
* **p95:** <200 ms (99% of 1-minute windows)
* **p99:** <250 ms (99.5% of 1-minute windows)

**Write Operations:**
* **p50:** <15 ms (95% of 1-minute windows)
* **p95:** <50 ms (99% of 1-minute windows)
* **p99:** <100 ms (99.5% of 1-minute windows)

### Error Rate SLO

**Target:** <1% error rate

* **Measurement:** Failed requests / total requests
* **Excluded Errors:** Client errors (4xx except 429)
* **Included Errors:** Server errors (5xx), timeouts, conflicts

### Throughput SLO

**Target:** >1,000 ops/sec sustained

* **Normal Range:** 1,200-1,400 ops/sec
* **Warning:** 1,000-1,200 ops/sec
* **Critical:** <1,000 ops/sec

---

## Metrics Collection

### RED Metrics (Primary)

**Rate, Errors, Duration - track for all operations**

#### Request Rate

```promql
# Total requests per second
rate(rfsource_requests_total[1m])

# By operation type
rate(rfsource_requests_total{method="read"}[1m])
rate(rfsource_requests_total{method="write"}[1m])
rate(rfsource_requests_total{method="branch"}[1m])
rate(rfsource_requests_total{method="admin"}[1m])
```

**Collection Frequency:** 10 seconds  
**Retention:** 30 days (raw), 1 year (aggregated)

#### Error Rate

```promql
# Overall error rate
rate(rfsource_requests_total{status="error"}[5m]) / 
rate(rfsource_requests_total[5m])

# By error type
rate(rfsource_requests_total{status="error",error_type="conflict"}[5m])
rate(rfsource_requests_total{status="error",error_type="timeout"}[5m])
rate(rfsource_requests_total{status="error",error_type="internal"}[5m])
```

**Collection Frequency:** 10 seconds  
**Retention:** 30 days (raw), 1 year (aggregated)

#### Request Duration (Latency)

```promql
# p50, p95, p99 latencies for reads
histogram_quantile(0.50, rate(rfsource_latency_seconds_bucket{method="read"}[1m]))
histogram_quantile(0.95, rate(rfsource_latency_seconds_bucket{method="read"}[1m]))
histogram_quantile(0.99, rate(rfsource_latency_seconds_bucket{method="read"}[1m]))

# p50, p95, p99 latencies for writes
histogram_quantile(0.50, rate(rfsource_latency_seconds_bucket{method="write"}[1m]))
histogram_quantile(0.95, rate(rfsource_latency_seconds_bucket{method="write"}[1m]))
histogram_quantile(0.99, rate(rfsource_latency_seconds_bucket{method="write"}[1m]))
```

**Collection Frequency:** 10 seconds  
**Retention:** 30 days (raw), 1 year (aggregated)

### USE Metrics (Secondary)

**Utilization, Saturation, Errors - for resource monitoring**

#### Utilization

```promql
# CPU usage (percentage)
rfsource_cpu_percent

# Memory usage (bytes)
rfsource_memory_bytes

# Disk usage (percentage)
rfsource_disk_usage_percent

# Active connections
rfsource_active_connections
```

**Collection Frequency:** 30 seconds  
**Retention:** 30 days

#### Saturation

```promql
# Queue depth (if applicable)
rfsource_request_queue_depth

# Connection pool saturation
rfsource_connection_pool_usage / rfsource_connection_pool_size

# Disk I/O wait time
rate(rfsource_disk_io_wait_seconds[1m])
```

**Collection Frequency:** 30 seconds  
**Retention:** 30 days

#### Resource Errors

```promql
# OOM events
increase(rfsource_oom_events_total[5m])

# Disk full errors
increase(rfsource_disk_full_errors_total[5m])

# Connection exhaustion
increase(rfsource_connection_exhausted_errors_total[5m])
```

**Collection Frequency:** 30 seconds  
**Retention:** 90 days (for incident analysis)

### Custom Business Metrics

#### Repository Metrics

```promql
# Total repositories
rfsource_repositories_total

# Repository size distribution
rfsource_repository_size_bytes_bucket

# Active repositories (accessed in last 24h)
rfsource_repositories_active_24h
```

#### Operation-Specific Metrics

```promql
# Time warp operations
rate(rfsource_time_warp_operations_total[5m])

# Branch operations
rate(rfsource_branch_operations_total[5m])

# Proposal operations
rate(rfsource_proposal_operations_total[5m])
```

---

## Alert Definitions

### Critical Alerts (SEV1)

**Immediate Response Required - Page On-Call**

#### CRIT-001: Service Down

```yaml
alert: RFSourceServiceDown
expr: up{job="rfsource"} == 0
for: 1m
severity: critical
annotations:
  summary: "RFSource service is down"
  description: "Health check has failed for 1 minute"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: SEV1 Service Down"
  action: "1. Check service status, 2. Attempt restart, 3. Check logs"
```

**Thresholds:**
* **Trigger:** Health check fails for 1 minute
* **Expected:** 0 occurrences per month
* **Impact:** Total service outage

#### CRIT-002: High Error Rate

```yaml
alert: RFSourceHighErrorRate
expr: |
  rate(rfsource_requests_total{status="error"}[5m]) / 
  rate(rfsource_requests_total[5m]) > 0.05
for: 2m
severity: critical
annotations:
  summary: "RFSource error rate exceeds 5%"
  description: "Current error rate: {{ $value | humanizePercentage }}"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: High Error Rate"
  action: "1. Check recent deployments, 2. Review error logs, 3. Consider rollback"
```

**Thresholds:**
* **Trigger:** Error rate > 5% for 2 minutes
* **Expected:** 0-1 occurrences per month
* **Impact:** Widespread user failures

#### CRIT-003: Extreme Latency Degradation

```yaml
alert: RFSourceExtremeLatency
expr: |
  histogram_quantile(0.95, 
    rate(rfsource_latency_seconds_bucket{method="read"}[5m])
  ) > 0.5
for: 5m
severity: critical
annotations:
  summary: "RFSource read latency p95 exceeds 500ms"
  description: "Current p95 latency: {{ $value | humanizeDuration }}"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: High Read Latency"
  action: "1. Check disk I/O, 2. Check resource saturation, 3. Scale if needed"
```

**Thresholds:**
* **Trigger:** Read p95 > 500ms for 5 minutes (2.5x baseline)
* **Expected:** 0-1 occurrences per month
* **Impact:** Service effectively unusable

### Warning Alerts (SEV2)

**Response Required - Notify On-Call (No Page)**

#### WARN-001: Elevated Error Rate

```yaml
alert: RFSourceElevatedErrorRate
expr: |
  rate(rfsource_requests_total{status="error"}[5m]) / 
  rate(rfsource_requests_total[5m]) > 0.01
for: 10m
severity: warning
annotations:
  summary: "RFSource error rate exceeds 1%"
  description: "Current error rate: {{ $value | humanizePercentage }} (baseline: 0.35%)"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: Concurrent Write Conflicts"
  action: "1. Check error patterns, 2. Verify if expected concurrent conflicts"
```

**Thresholds:**
* **Trigger:** Error rate > 1% for 10 minutes
* **Expected:** 0-5 occurrences per month
* **Impact:** Some user operations failing

#### WARN-002: High Read Latency

```yaml
alert: RFSourceHighReadLatency
expr: |
  histogram_quantile(0.95, 
    rate(rfsource_latency_seconds_bucket{method="read"}[5m])
  ) > 0.2
for: 5m
severity: warning
annotations:
  summary: "RFSource read latency p95 exceeds 200ms"
  description: "Current p95: {{ $value | humanizeDuration }} (baseline: 119ms)"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: High Read Latency"
  action: "1. Check disk I/O, 2. Check cache hit rate, 3. Monitor for escalation"
```

**Thresholds:**
* **Trigger:** Read p95 > 200ms for 5 minutes
* **Expected:** 2-5 occurrences per month
* **Impact:** Noticeable performance degradation

#### WARN-003: High Write Latency

```yaml
alert: RFSourceHighWriteLatency
expr: |
  histogram_quantile(0.95, 
    rate(rfsource_latency_seconds_bucket{method="write"}[5m])
  ) > 0.05
for: 5m
severity: warning
annotations:
  summary: "RFSource write latency p95 exceeds 50ms"
  description: "Current p95: {{ $value | humanizeDuration }} (baseline: 15ms)"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: High Write Latency"
  action: "1. Check disk I/O, 2. Check for write contention, 3. Monitor escalation"
```

**Thresholds:**
* **Trigger:** Write p95 > 50ms for 5 minutes
* **Expected:** 2-5 occurrences per month
* **Impact:** Slower write operations

#### WARN-004: High Memory Usage

```yaml
alert: RFSourceHighMemory
expr: rfsource_memory_bytes > 500000000
for: 10m
severity: warning
annotations:
  summary: "RFSource memory usage exceeds 500MB"
  description: "Current memory: {{ $value | humanize }}MB (baseline: 27MB)"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: Memory Usage Growing"
  action: "1. Compare to 24h projection, 2. Check for memory leak, 3. Restart if >1GB"
```

**Thresholds:**
* **Trigger:** Memory > 500 MB for 10 minutes
* **Expected:** 0-2 occurrences per month (during high load)
* **Impact:** Potential OOM risk

#### WARN-005: High CPU Usage

```yaml
alert: RFSourceHighCPU
expr: rfsource_cpu_percent > 25
for: 10m
severity: warning
annotations:
  summary: "RFSource CPU usage exceeds 25%"
  description: "Current CPU: {{ $value }}% (baseline: 4.6%)"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: High CPU Usage"
  action: "1. Check for load spike, 2. Review slow queries, 3. Consider scaling"
```

**Thresholds:**
* **Trigger:** CPU > 25% for 10 minutes
* **Expected:** 1-3 occurrences per month
* **Impact:** Approaching capacity limits

#### WARN-006: Low Throughput

```yaml
alert: RFSourceLowThroughput
expr: rate(rfsource_requests_total[5m]) < 1000
for: 15m
severity: warning
annotations:
  summary: "RFSource throughput below 1,000 ops/sec"
  description: "Current throughput: {{ $value | humanize }} ops/sec (baseline: 1,282)"
  runbook: "See DOC-OPS-RFSOURCE-RUNBOOK-001 Section: Low Throughput"
  action: "1. Check if actual low traffic or performance issue, 2. Investigate bottlenecks"
```

**Thresholds:**
* **Trigger:** Throughput < 1,000 ops/sec for 15 minutes
* **Expected:** 5-10 occurrences per month (legitimate low traffic)
* **Impact:** May indicate performance degradation or low usage

### Informational Alerts (SEV3)

**No Immediate Action - Queue for Review**

#### INFO-001: Approaching Disk Space Limit

```yaml
alert: RFSourceDiskSpaceWarning
expr: rfsource_disk_usage_percent > 70
for: 1h
severity: info
annotations:
  summary: "RFSource disk usage exceeds 70%"
  description: "Current disk usage: {{ $value }}%"
  action: "1. Plan capacity expansion, 2. Review old files for archival"
```

#### INFO-002: Large Repository Detected

```yaml
alert: RFSourceLargeRepository
expr: rfsource_repository_size_bytes > 1500000000
for: 5m
severity: info
annotations:
  summary: "Repository approaching 1.5GB size limit"
  description: "Repository: {{ $labels.repository }}, Size: {{ $value | humanize }}GB"
  action: "1. Alert repository owner, 2. Monitor for 2GB OS limit"
```

**Note:** This is critical limitation REM-002 (no explicit file size limit)

---

## Dashboard Layouts

### Primary Operations Dashboard

**Purpose:** Real-time health monitoring  
**Audience:** On-call engineers, operations team  
**Update Frequency:** 10 seconds

**Panels:**

1. **Service Status (Top Row)**
   - Health check status (green/red indicator)
   - Uptime percentage (last 30 days)
   - Current version deployed
   - Active alerts count

2. **Request Rate (Row 2)**
   - Total requests per second (line graph, last 1 hour)
   - Requests by operation type (stacked area chart)
   - Requests by status (success/error, stacked area)

3. **Latency (Row 3)**
   - Read latency p50/p95/p99 (multi-line graph, last 1 hour)
   - Write latency p50/p95/p99 (multi-line graph, last 1 hour)
   - Latency heatmap (last 6 hours)

4. **Error Rate (Row 4)**
   - Error rate percentage (line graph with threshold line, last 1 hour)
   - Errors by type (pie chart, last 15 minutes)
   - Recent error log sample (table)

5. **Resource Usage (Row 5)**
   - CPU usage (gauge + line graph, last 1 hour)
   - Memory usage (gauge + line graph, last 1 hour)
   - Disk usage (gauge)
   - Active connections (gauge)

### Capacity Planning Dashboard

**Purpose:** Long-term trend analysis  
**Audience:** Engineering team, management  
**Update Frequency:** 5 minutes

**Panels:**

1. **Trends Over Time**
   - Daily request volume (last 30 days)
   - Average latency trends (last 30 days)
   - Peak concurrent users (last 30 days)
   - Error rate trends (last 30 days)

2. **Resource Projections**
   - Memory usage trend with projection
   - CPU usage trend with projection
   - Disk usage growth rate
   - Estimated time to capacity limits

3. **SLO Compliance**
   - Availability SLO burn rate
   - Latency SLO compliance percentage
   - Error budget remaining
   - SLO violations (last 30 days)

4. **Repository Statistics**
   - Total repositories count
   - Repository size distribution
   - Most active repositories
   - Largest repositories

### Debugging Dashboard

**Purpose:** Deep investigation during incidents  
**Audience:** On-call engineers, developers  
**Update Frequency:** 5 seconds

**Panels:**

1. **Request Details**
   - Request traces (last 100 requests)
   - Slow query log (p99 latencies)
   - Error stack traces
   - Request size distribution

2. **System Internals**
   - Goroutine count (if Go-based)
   - Thread count
   - File descriptor count
   - Cache hit rate

3. **Disk I/O**
   - Read IOPS
   - Write IOPS
   - I/O wait time
   - Queue depth

4. **Network**
   - Connection states (established/time_wait/etc)
   - Bytes sent/received
   - Connection errors
   - DNS resolution time

---

## Monitoring Tool Setup

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 10s
  evaluation_interval: 10s

scrape_configs:
  - job_name: 'rfsource'
    static_configs:
      - targets: ['rfsource:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s

rule_files:
  - 'rfsource_alerts.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

### Grafana Data Source

```yaml
# grafana_datasource.yml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    access: proxy
    isDefault: true
    editable: false
```

### AlertManager Configuration

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m

route:
  receiver: 'default'
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'
      continue: true
    - match:
        severity: warning
      receiver: 'slack-warnings'
    - match:
        severity: info
      receiver: 'slack-info'

receivers:
  - name: 'default'
    slack_configs:
      - channel: '#rfsource-alerts'
        api_url: '${SLACK_WEBHOOK_URL}'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: '${PAGERDUTY_SERVICE_KEY}'
        description: '{{ .GroupLabels.alertname }}: {{ .CommonAnnotations.summary }}'

  - name: 'slack-warnings'
    slack_configs:
      - channel: '#rfsource-warnings'
        api_url: '${SLACK_WEBHOOK_URL}'
        title: 'WARNING: {{ .GroupLabels.alertname }}'

  - name: 'slack-info'
    slack_configs:
      - channel: '#rfsource-info'
        api_url: '${SLACK_WEBHOOK_URL}'
        title: 'INFO: {{ .GroupLabels.alertname }}'
```

---

## On-Call Procedures

### Alert Acknowledgment

**Within 5 minutes of critical alert:**
1. Acknowledge alert in PagerDuty
2. Post acknowledgment in `#rfsource-incidents`
3. Begin investigation

**Within 15 minutes of warning alert:**
1. Review alert details
2. Determine if immediate action needed
3. Update Slack channel

### Alert Triage

**For Each Alert:**
1. **Read the alert** - Summary, description, current value
2. **Check runbook** - Follow linked runbook section
3. **Assess impact** - Is it affecting users?
4. **Determine severity** - Confirm/adjust SEV level
5. **Take action** - Follow runbook procedures

### Escalation Criteria

**Escalate to L2 (Engineering Lead) if:**
- Cannot resolve critical alert in 30 minutes
- Multiple critical alerts firing simultaneously
- Unclear root cause after initial investigation

**Escalate to L3 (System Architect) if:**
- Data loss risk identified
- Architectural change required
- System behavior not matching design

### Post-Incident Review

**After resolving SEV1 or SEV2 incidents:**
1. Document timeline in incident tracker
2. Identify root cause
3. List corrective actions
4. Update runbook if needed
5. Schedule postmortem (SEV1 only)

---

## Runbook Integration

### Alert → Runbook Mapping

| Alert | Runbook Section |
|-------|-----------------|
| CRIT-001: Service Down | Section: SEV1 Service Down |
| CRIT-002: High Error Rate | Section: Write Operation Failures |
| CRIT-003: Extreme Latency | Section: High Read Latency |
| WARN-001: Elevated Error Rate | Section: Concurrent Write Conflicts |
| WARN-002: High Read Latency | Section: High Read Latency |
| WARN-003: High Write Latency | Section: Write Operation Failures |
| WARN-004: High Memory Usage | Section: Memory Usage Growing |
| WARN-005: High CPU Usage | Section: Capacity Planning |
| WARN-006: Low Throughput | Section: Capacity Planning |

### Automated Remediation

**Current Status:** Manual remediation only

**Future Enhancements (Post-Phase 1):**
- Auto-restart on service down (with circuit breaker)
- Auto-scale on high CPU/memory
- Auto-clear cache on memory pressure
- Auto-throttle on high error rate

---

## Appendix

### Metric Naming Conventions

**Format:** `rfsource_{component}_{metric}_{unit}`

**Examples:**
- `rfsource_requests_total` (counter)
- `rfsource_latency_seconds` (histogram)
- `rfsource_memory_bytes` (gauge)
- `rfsource_cpu_percent` (gauge)

### Retention Policies

| Metric Type | Raw Retention | Aggregated Retention |
|-------------|---------------|----------------------|
| RED Metrics | 30 days | 1 year (hourly) |
| USE Metrics | 30 days | 90 days (hourly) |
| Business Metrics | 90 days | 2 years (daily) |
| Logs | 14 days | N/A |

### Dashboard Access

**Primary Operations Dashboard:** https://grafana.example.com/d/rfsource-ops  
**Capacity Planning Dashboard:** https://grafana.example.com/d/rfsource-capacity  
**Debugging Dashboard:** https://grafana.example.com/d/rfsource-debug

---

**Document Version:** 1.0.0  
**Last Updated:** 2026-05-07  
**Next Review:** 2026-06-07 (or after first major incident)
