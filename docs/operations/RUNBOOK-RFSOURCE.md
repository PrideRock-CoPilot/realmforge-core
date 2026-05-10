---
doc_id: DOC-OPS-RFSOURCE-RUNBOOK-001
title: RFSource Production Runbook
status: active
version: 1.0.0
created_at: 2026-05-07
last_updated: 2026-05-07
owner: operations_team
related_docs:
  - DOC-TEST-RFSOURCE-LOAD-500-001
  - DOC-TEST-RFSOURCE-EXTENDED-001
  - DOC-AUDIT-RFSOURCE-001
---

# RFSource Production Runbook

**Version:** 1.0.0  
**Last Updated:** 2026-05-07  
**Service:** rfsource (8-crate file storage system)  
**Load Test Validated:** ✅ 500 concurrent users, 5-minute sustained load

---

## Table of Contents

1. [Service Overview](#service-overview)
2. [Deployment Procedures](#deployment-procedures)
3. [Operational Procedures](#operational-procedures)
4. [Monitoring & Health Checks](#monitoring--health-checks)
5. [Troubleshooting](#troubleshooting)
6. [Incident Response](#incident-response)
7. [Capacity Planning](#capacity-planning)
8. [Known Limitations](#known-limitations)
9. [Rollback Procedures](#rollback-procedures)
10. [Contacts & Escalation](#contacts--escalation)

---

## Service Overview

### Architecture

RFSource is an 8-crate modular file storage system:

| Crate | Purpose | Critical Path |
|-------|---------|---------------|
| `rfsource-core` | Core types, traits, storage interface | ✅ Yes |
| `rfsource-format` | Binary format serialization | ✅ Yes |
| `rfsource-index` | File indexing, content addressing | ✅ Yes |
| `rfsource-store` | Storage backend implementation | ✅ Yes |
| `rfsource-governance` | History, proposals, time travel | ⚠️ Partial |
| `rfsource-catalog` | Metadata catalog | ⚠️ Partial |
| `rfsource-query` | Query engine | ❌ No |
| `rfsource-materialize` | Content materialization | ⚠️ Partial |
| `rfsource-service` | Service wrapper | ✅ Yes |

### Performance Baseline

**Validated with 500 concurrent users (5-minute sustained load):**

| Metric | Value | Alert Threshold |
|--------|-------|-----------------|
| **Read Latency (p95)** | 119ms | >200ms |
| **Write Latency (p95)** | 15ms | >50ms |
| **Throughput** | 1,282 ops/sec | <1,000 ops/sec |
| **Error Rate** | 0.35% | >1% |
| **Peak Memory** | 27 MB | >100 MB |
| **Average CPU** | 4.6% | >25% |

### Service Limits

| Limit | Value | Status |
|-------|-------|--------|
| **Max Concurrent Users** | 500 (validated) | ✅ Tested |
| **Safe Operating Capacity** | 2,000 users | 📊 Projected |
| **Maximum Capacity** | 5,000 users | ⚠️ Needs validation |
| **Max File Size** | ~2 GB (OS limit) | 🔴 Not enforced |
| **Multi-File Repos** | Not supported | 🔴 Single file only |

---

## Deployment Procedures

### Pre-Deployment Checklist

**Infrastructure:**
- [ ] Target environment provisioned (compute, storage, network)
- [ ] Resource limits configured (memory, CPU)
- [ ] Dependencies installed (Rust toolchain if building from source)
- [ ] Storage backend accessible (filesystem permissions)
- [ ] Monitoring infrastructure ready

**Configuration:**
- [ ] Environment variables set (see Production Configuration)
- [ ] File size limits documented (OS-level)
- [ ] Backup strategy defined
- [ ] Log aggregation configured

**Testing:**
- [ ] Load test results reviewed (baseline metrics documented)
- [ ] Smoke tests prepared
- [ ] Rollback plan documented

**Team:**
- [ ] On-call engineer identified
- [ ] Deployment window scheduled
- [ ] Rollback criteria agreed upon
- [ ] Communication plan ready

### Deployment Steps

#### Option A: Binary Deployment (Recommended for Production)

```bash
# 1. Download pre-built binary
curl -L https://github.com/yourorg/rfsource/releases/download/v1.0.0/rfsource -o rfsource
chmod +x rfsource

# 2. Verify checksum
sha256sum rfsource
# Compare with published checksum

# 3. Create service directory structure
mkdir -p /opt/rfsource/{bin,data,logs,config}
mv rfsource /opt/rfsource/bin/

# 4. Create service user (security best practice)
sudo useradd -r -s /bin/false rfsource-service
sudo chown -R rfsource-service:rfsource-service /opt/rfsource

# 5. Configure service (see Production Configuration)
cat > /opt/rfsource/config/rfsource.env << EOF
RFSOURCE_DATA_DIR=/opt/rfsource/data
RFSOURCE_LOG_LEVEL=info
RFSOURCE_MAX_MEMORY_MB=500
EOF

# 6. Create systemd service
sudo cat > /etc/systemd/system/rfsource.service << EOF
[Unit]
Description=RFSource File Storage Service
After=network.target

[Service]
Type=simple
User=rfsource-service
Group=rfsource-service
EnvironmentFile=/opt/rfsource/config/rfsource.env
ExecStart=/opt/rfsource/bin/rfsource
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
EOF

# 7. Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable rfsource
sudo systemctl start rfsource

# 8. Verify service is running
sudo systemctl status rfsource
```

#### Option B: Source Build Deployment

```bash
# 1. Clone repository
git clone https://github.com/yourorg/rfsource.git /opt/rfsource-build
cd /opt/rfsource-build

# 2. Checkout specific version
git checkout v1.0.0

# 3. Build release binary
cargo build --release --workspace

# 4. Run tests
cargo test --workspace

# 5. Install binary
sudo cp target/release/rfsource /opt/rfsource/bin/

# Follow steps 4-8 from Option A
```

### Post-Deployment Verification

```bash
# 1. Check service status
sudo systemctl status rfsource

# 2. Verify logs for startup errors
sudo journalctl -u rfsource -n 50

# 3. Check resource usage
ps aux | grep rfsource
top -p $(pgrep rfsource)

# 4. Run smoke test (see Smoke Tests section)
./scripts/smoke_test_rfsource.sh

# 5. Verify monitoring dashboards populate
# Check Grafana/Datadog/etc for metrics

# 6. Test basic operations
# READ test
curl http://localhost:8080/api/v1/read/test-file

# WRITE test
curl -X POST http://localhost:8080/api/v1/write   -H "Content-Type: application/json"   -d '{"file": "test", "content": "hello"}'
```

---

## Operational Procedures

### Starting the Service

```bash
# Start via systemd
sudo systemctl start rfsource

# Verify startup
sudo systemctl status rfsource
sudo journalctl -u rfsource -f  # Follow logs
```

### Stopping the Service

```bash
# Graceful stop (allows in-flight requests to complete)
sudo systemctl stop rfsource

# Verify shutdown
sudo systemctl status rfsource

# Check for lingering processes
ps aux | grep rfsource
```

### Restarting the Service

```bash
# Graceful restart
sudo systemctl restart rfsource

# Verify restart
sudo systemctl status rfsource
tail -f /opt/rfsource/logs/rfsource.log
```

### Viewing Logs

```bash
# System logs (journalctl)
sudo journalctl -u rfsource -f          # Follow
sudo journalctl -u rfsource -n 100      # Last 100 lines
sudo journalctl -u rfsource --since "1 hour ago"

# Application logs (if file-based)
tail -f /opt/rfsource/logs/rfsource.log
grep ERROR /opt/rfsource/logs/rfsource.log
```

### Configuration Reload

```bash
# If service supports SIGHUP for config reload
sudo systemctl reload rfsource

# Otherwise, restart required
sudo systemctl restart rfsource
```

---

## Monitoring & Health Checks

### Key Metrics to Monitor

#### Performance Metrics

| Metric | Normal Range | Warning | Critical | Action |
|--------|--------------|---------|----------|--------|
| **Read Latency (p95)** | 100-120ms | 150-200ms | >200ms | Investigate slow queries |
| **Write Latency (p95)** | 12-15ms | 30-50ms | >50ms | Check disk I/O |
| **Throughput** | 1,200-1,400 ops/s | 1,000-1,200 ops/s | <1,000 ops/s | Scale up or investigate |
| **Error Rate** | 0.2-0.5% | 0.5-1% | >1% | Check logs immediately |

#### Resource Metrics

| Metric | Normal Range | Warning | Critical | Action |
|--------|--------------|---------|----------|--------|
| **Memory Usage** | 20-40 MB | 100-200 MB | >500 MB | Possible memory leak |
| **CPU Usage** | 3-7% | 15-25% | >25% | Scale or optimize |
| **Disk Usage** | <70% | 70-85% | >85% | Add capacity |
| **File Descriptors** | <1,000 | 1,000-5,000 | >5,000 | Check for leaks |

### Health Check Endpoints

```bash
# Basic health check
curl http://localhost:8080/health
# Expected: {"status": "healthy", "version": "1.0.0"}

# Detailed health with metrics
curl http://localhost:8080/health/detailed
# Expected: JSON with memory, CPU, active connections

# Readiness check (ready to serve traffic)
curl http://localhost:8080/ready
# Expected: 200 OK if ready, 503 if not

# Liveness check (process is alive)
curl http://localhost:8080/alive
# Expected: 200 OK always (unless process dead)
```

### Monitoring Tools Setup

#### Prometheus Metrics Endpoint

```bash
# Scrape metrics (if implemented)
curl http://localhost:8080/metrics

# Expected format:
# rfsource_requests_total{method="read",status="success"} 12345
# rfsource_latency_seconds{method="read",quantile="0.95"} 0.119
# rfsource_memory_bytes 27000000
```

#### Grafana Dashboard

**Recommended Panels:**
1. Request Rate (ops/sec)
2. Latency Distribution (p50/p95/p99)
3. Error Rate (%)
4. Memory Usage (MB)
5. CPU Usage (%)
6. Active Connections
7. Disk I/O (read/write bytes)

#### Alert Rules

```yaml
# Example Prometheus alert rules
groups:
  - name: rfsource_alerts
    interval: 30s
    rules:
      - alert: RFSourceHighLatency
        expr: rfsource_latency_seconds{quantile="0.95"} > 0.2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "RFSource read latency p95 exceeds 200ms"

      - alert: RFSourceHighErrorRate
        expr: rate(rfsource_requests_total{status="error"}[5m]) > 0.01
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "RFSource error rate exceeds 1%"

      - alert: RFSourceMemoryLeak
        expr: rfsource_memory_bytes > 500000000
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "RFSource memory usage exceeds 500MB"

      - alert: RFSourceDown
        expr: up{job="rfsource"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "RFSource service is down"
```

---

## Troubleshooting

### Common Issues

#### Issue: High Read Latency (p95 > 200ms)

**Symptoms:**
- Slow response times for read operations
- User complaints about performance
- Read latency metric exceeds 200ms

**Diagnosis:**
```bash
# Check disk I/O
iostat -x 1 5

# Check file system cache
free -h

# Check for slow queries in logs
grep "slow_query" /opt/rfsource/logs/rfsource.log
```

**Possible Causes:**
1. Disk I/O contention
2. Cold cache (after restart)
3. Large files being read
4. High concurrent read load

**Resolution:**
```bash
# 1. Check if disk is bottleneck
sudo iotop

# 2. Increase system cache (if available memory)
# Restart service to reload cache warmup

# 3. Scale horizontally (add read replicas if supported)

# 4. Optimize queries (check for full-file reads vs. partial)
```

#### Issue: Write Operation Failures

**Symptoms:**
- Write operations returning errors
- Error rate spike in monitoring
- Logs show write failures

**Diagnosis:**
```bash
# Check disk space
df -h /opt/rfsource/data

# Check disk write errors
dmesg | grep -i error

# Check file permissions
ls -la /opt/rfsource/data

# Check for file locks
lsof /opt/rfsource/data/.rfsource
```

**Possible Causes:**
1. Disk full (>85% usage)
2. Permission issues
3. File lock contention
4. Filesystem errors

**Resolution:**
```bash
# 1. Free disk space
# Delete old logs, temp files

# 2. Fix permissions
sudo chown -R rfsource-service:rfsource-service /opt/rfsource/data

# 3. Check for deadlocks in logs
grep "deadlock\|timeout" /opt/rfsource/logs/rfsource.log

# 4. Restart service if locks stuck
sudo systemctl restart rfsource
```

#### Issue: Memory Usage Growing (Possible Leak)

**Symptoms:**
- Memory usage exceeds 100 MB and growing
- OOM killer activated
- Service crashes with memory errors

**Diagnosis:**
```bash
# Monitor memory over time
watch -n 1 'ps aux | grep rfsource'

# Check for memory leak patterns
cat /proc/$(pgrep rfsource)/status | grep -i mem

# Review load test projections
# Baseline: 27 MB for 500 users
# 24-hour projection: ~256 MB
```

**Possible Causes:**
1. Actual memory leak (code bug)
2. Unbounded cache growth
3. Connections not being closed
4. Large files held in memory

**Resolution:**
```bash
# 1. Compare against load test baseline
# If < 500 MB after 24 hours: likely normal

# 2. If > 500 MB: investigate
# Check for connection leaks
netstat -an | grep :8080 | wc -l

# 3. Restart service as temporary fix
sudo systemctl restart rfsource

# 4. Escalate to engineering for code review
```

#### Issue: Service Won't Start

**Symptoms:**
- `systemctl start rfsource` fails
- Service status shows "failed"
- No process running

**Diagnosis:**
```bash
# Check service status
sudo systemctl status rfsource

# View recent logs
sudo journalctl -u rfsource -n 50

# Check if port already in use
sudo lsof -i :8080

# Verify binary exists and is executable
ls -la /opt/rfsource/bin/rfsource
```

**Possible Causes:**
1. Port already in use
2. Configuration error
3. Missing dependencies
4. Permission issues

**Resolution:**
```bash
# 1. Kill process using port
sudo kill $(sudo lsof -t -i :8080)

# 2. Fix configuration
sudo nano /opt/rfsource/config/rfsource.env

# 3. Check permissions
sudo chown rfsource-service:rfsource-service /opt/rfsource/bin/rfsource
sudo chmod +x /opt/rfsource/bin/rfsource

# 4. Try manual start for detailed error
sudo -u rfsource-service /opt/rfsource/bin/rfsource
```

#### Issue: Concurrent Write Conflicts (Error Rate 0.5-1%)

**Symptoms:**
- Error rate elevated but < 1%
- Logs show "conflict" or "concurrent write" errors
- Users report occasional write failures

**Diagnosis:**
```bash
# Check error patterns
grep "conflict\|concurrent" /opt/rfsource/logs/rfsource.log | wc -l

# Check write concurrency
# Expected: 0.35-0.5% error rate is NORMAL for 500 users
```

**Possible Causes:**
1. **EXPECTED BEHAVIOR** - concurrent writes to same resource
2. High write contention on popular files

**Resolution:**
- **0.2-0.5% error rate:** NORMAL, no action needed
- **0.5-1% error rate:** Acceptable, monitor
- **>1% error rate:** Investigate file contention patterns

**Known Limitation:**
- Concurrent write safety gaps (F-010 in QA cert)
- This is expected behavior, not a critical issue

---

## Incident Response

### Severity Levels

| Level | Definition | Response Time | Escalation |
|-------|------------|---------------|------------|
| **SEV1** | Service down, data loss risk | Immediate | Page on-call + manager |
| **SEV2** | Degraded performance, elevated errors | 15 minutes | On-call engineer |
| **SEV3** | Minor issues, non-critical | 1 hour | Queue for next business day |

### SEV1: Service Down

**Indicators:**
- Health check failing
- Service not responding
- CPU/Memory exhausted

**Immediate Actions:**
1. **Acknowledge incident** (PagerDuty, Slack, etc.)
2. **Check service status:** `sudo systemctl status rfsource`
3. **Attempt restart:** `sudo systemctl restart rfsource`
4. **If restart fails:** Check logs for errors
5. **Notify stakeholders:** Service degradation communication

**Investigation:**
```bash
# 1. View recent logs
sudo journalctl -u rfsource -n 200 > /tmp/rfsource-incident-logs.txt

# 2. Check resource exhaustion
df -h                    # Disk space
free -h                  # Memory
top                      # CPU

# 3. Check for crashes
dmesg | tail -n 50       # Kernel messages
```

**Recovery:**
```bash
# Option 1: Restart
sudo systemctl restart rfsource

# Option 2: Rollback (if recent deployment)
# See Rollback Procedures section

# Option 3: Restore from backup
# See Backup/Restore section
```

### SEV2: Performance Degradation

**Indicators:**
- Latency > 200ms (p95)
- Error rate > 1%
- CPU > 25%

**Response:**
1. Check monitoring dashboards
2. Identify affected operation type (read/write)
3. Review recent changes (deployments, config)
4. Scale resources if needed

**Mitigation:**
```bash
# Temporary: Increase resources
# Permanent: Identify root cause

# If disk I/O bottleneck:
# Scale to faster storage

# If CPU bottleneck:
# Scale horizontally (add instances)

# If memory bottleneck:
# Investigate leak, restart if needed
```

### Incident Communication Template

```
**INCIDENT ALERT: RFSource Service Degradation**

Severity: [SEV1/SEV2/SEV3]
Status: [Investigating/Identified/Monitoring/Resolved]
Started: [YYYY-MM-DD HH:MM UTC]

Impact:
- [Describe user impact]
- [Affected operations: read/write/both]

Current Actions:
- [What's being done right now]

Next Update: [Time of next status update]

Updates:
[Chronological log of actions taken]
```

---

## Capacity Planning

### Current Capacity

**Validated (Load Test):**
- 500 concurrent users
- 1,282 ops/sec sustained
- 27 MB memory, 4.6% CPU

**Projected Safe Capacity:**
- 2,000 concurrent users
- ~5,000 ops/sec
- ~104 MB memory, ~20% CPU

**Projected Maximum Capacity:**
- 5,000 concurrent users
- ~12,500 ops/sec
- ~260 MB memory, ~50% CPU

### Scaling Triggers

**Scale Up (Vertical) When:**
- CPU > 15% sustained for 10 minutes
- Memory > 200 MB sustained
- Latency p95 > 150ms for 5 minutes

**Scale Out (Horizontal) When:**
- Approaching 5,000 concurrent users
- CPU > 50% sustained
- Need geographic distribution

### Scaling Procedures

#### Vertical Scaling (Increase Resources)

```bash
# 1. Increase memory limit in service config
sudo nano /opt/rfsource/config/rfsource.env
# RFSOURCE_MAX_MEMORY_MB=1000

# 2. Increase CPU allocation (if containerized)
# Update resource limits in orchestrator (K8s, ECS, etc.)

# 3. Restart service
sudo systemctl restart rfsource
```

#### Horizontal Scaling (Add Instances)

**Requirements:**
- Load balancer configured
- Shared storage backend (or replication)
- Health check endpoints exposed

**Procedure:**
1. Deploy new instance following deployment procedures
2. Add to load balancer pool
3. Monitor metrics for even distribution
4. Adjust load balancer weights as needed

**Known Limitation:**
- Multi-file repo splitting not yet implemented (REM-003)
- May impact horizontal scaling for very large repos

---

## Known Limitations

### Critical Limitations (Documented in Audit)

1. **No Explicit File Size Limit (REM-002)**
   - **Impact:** Repos can grow until OS limit (~2GB)
   - **Risk:** Crash or corruption when limit reached
   - **Workaround:** Monitor file sizes, alert at 1.5 GB
   - **Timeline:** Fix required, 1 day effort

2. **Single-File Repos Only (REM-003)**
   - **Impact:** Cannot split large repos across multiple files
   - **Risk:** Performance degradation for very large repos
   - **Workaround:** None currently
   - **Timeline:** Design Council session required, 2 days

3. **Concurrent Write Safety Gaps (F-010)**
   - **Impact:** 0.35-0.5% error rate expected under load
   - **Risk:** Write conflicts on highly contended files
   - **Workaround:** Application-level retry logic
   - **Timeline:** Phase 2 fix

4. **No Authentication (F-003)**
   - **Impact:** Service assumes trusted network
   - **Risk:** Unauthorized access if exposed publicly
   - **Workaround:** Deploy behind VPN/firewall
   - **Timeline:** Requirements doc needed (REM-007)

5. **Proposal Atomicity Gaps (F-007)**
   - **Impact:** Partial failures during complex operations
   - **Risk:** Inconsistent state in edge cases
   - **Workaround:** Monitor for partial failures, manual recovery
   - **Timeline:** Phase 2 fix

### Operational Limitations

1. **No Built-In Backup/Restore**
   - Must use filesystem-level backups
   - No point-in-time recovery

2. **No Replication**
   - Single point of failure (file storage)
   - No automatic failover

3. **No Query Optimization**
   - All queries are full scans (rfsource-query not fully implemented)

---

## Rollback Procedures

### When to Rollback

**Trigger Conditions:**
- SEV1 incident within 30 minutes of deployment
- Error rate > 5% post-deployment
- Data corruption detected
- Critical functionality broken

**Decision Matrix:**
| Time Since Deploy | Error Rate | Decision |
|-------------------|------------|----------|
| < 30 minutes | >1% | **Rollback** |
| < 1 hour | >5% | **Rollback** |
| > 1 hour | <1% | **Monitor** |
| > 1 hour | >5% | **Investigate, then rollback if needed** |

### Rollback Procedure

```bash
# 1. Stop current service
sudo systemctl stop rfsource

# 2. Backup current version (for forensics)
sudo cp /opt/rfsource/bin/rfsource /opt/rfsource/bin/rfsource.failed

# 3. Restore previous version
sudo cp /opt/rfsource/bin/rfsource.previous /opt/rfsource/bin/rfsource

# 4. Restore previous configuration (if changed)
sudo cp /opt/rfsource/config/rfsource.env.previous /opt/rfsource/config/rfsource.env

# 5. Start service
sudo systemctl start rfsource

# 6. Verify rollback
sudo systemctl status rfsource
curl http://localhost:8080/health

# 7. Run smoke tests
./scripts/smoke_test_rfsource.sh

# 8. Monitor for 15 minutes
# Check metrics return to baseline

# 9. Notify stakeholders
# Incident communication: Rolled back to previous version
```

### Post-Rollback Actions

1. **Root Cause Analysis**
   - Review logs from failed deployment
   - Identify what changed
   - Document findings

2. **Fix Forward Plan**
   - Document fix required
   - Test fix in staging
   - Schedule re-deployment

3. **Update Runbook**
   - Add new known issues
   - Update rollback procedures if needed

---

## Contacts & Escalation

### Escalation Path

| Level | Contact | When to Escalate |
|-------|---------|------------------|
| **L1** | On-Call Engineer | First responder, all incidents |
| **L2** | Engineering Lead | SEV1, or L1 can't resolve in 30 min |
| **L3** | System Architect | Data loss risk, architecture decisions |
| **L4** | VP Engineering | Multi-hour outage, major incident |

### Contact Information

**On-Call Rotation:**
- PagerDuty: `rfsource-oncall`
- Slack: `#rfsource-incidents`

**Key Contacts:**
- Engineering Lead: [Name] - [Email] - [Phone]
- System Architect: [Name] - [Email] - [Phone]
- Operations Manager: [Name] - [Email] - [Phone]

### Communication Channels

- **Incidents:** `#rfsource-incidents` (Slack)
- **Operations:** `#rfsource-ops` (Slack)
- **Status Page:** [URL to status page]
- **Documentation:** [URL to wiki/docs]

---

## Appendix

### Smoke Test Script

```bash
#!/bin/bash
# File: scripts/smoke_test_rfsource.sh

set -e

echo "Starting RFSource smoke tests..."

# Test 1: Health check
echo "Test 1: Health check"
curl -f http://localhost:8080/health || exit 1

# Test 2: Write operation
echo "Test 2: Write operation"
curl -f -X POST http://localhost:8080/api/v1/write   -H "Content-Type: application/json"   -d '{"file": "smoke-test", "content": "hello"}' || exit 1

# Test 3: Read operation
echo "Test 3: Read operation"
curl -f http://localhost:8080/api/v1/read/smoke-test || exit 1

# Test 4: List operation
echo "Test 4: List operation"
curl -f http://localhost:8080/api/v1/list || exit 1

echo "✅ All smoke tests passed"
```

### Load Test Baseline

**Reference:** DOC-TEST-RFSOURCE-EXTENDED-001

| Metric | 500 Users (5 min) |
|--------|-------------------|
| Total Operations | 385,444 |
| Throughput | 1,282 ops/sec |
| Read p50 | 110.10 ms |
| Read p95 | 119.10 ms |
| Write p50 | 12.59 ms |
| Write p95 | 14.84 ms |
| Error Rate | 0.35% |
| Peak Memory | 26.9 MB |
| Avg CPU | 4.6% |

### Configuration Reference

**Environment Variables:**
```bash
# Data storage location
RFSOURCE_DATA_DIR=/opt/rfsource/data

# Logging
RFSOURCE_LOG_LEVEL=info  # debug, info, warn, error
RFSOURCE_LOG_FILE=/opt/rfsource/logs/rfsource.log

# Resource limits
RFSOURCE_MAX_MEMORY_MB=500
RFSOURCE_MAX_CONNECTIONS=1000

# Performance tuning
RFSOURCE_CACHE_SIZE_MB=100
RFSOURCE_IO_THREADS=4
```

---

**Document Version:** 1.0.0  
**Last Updated:** 2026-05-07  
**Next Review:** 2026-06-07 (or after first major incident)
