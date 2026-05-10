---
doc_id: DOC-OPS-RFSOURCE-CHECKLIST-001
title: RFSource Production Deployment Checklist
status: active
version: 1.0.0
created_at: 2026-05-07
related_docs:
  - DOC-OPS-RFSOURCE-RUNBOOK-001
  - DOC-OPS-RFSOURCE-MONITORING-001
  - DOC-TEST-RFSOURCE-EXTENDED-001
---

# RFSource Production Deployment Checklist

**Version:** 1.0.0  
**Load Test Validated:** ✅ 500 users, 5-minute sustained load

---

## Pre-Deployment Checklist

### Infrastructure Readiness

- [ ] **Compute Resources Provisioned**
  - CPU: Minimum 4 cores
  - Memory: Minimum 2 GB (500 MB for service + overhead)
  - Network: 1 Gbps minimum

- [ ] **Storage Configured**
  - Filesystem: ext4, XFS, or similar (NOT FAT32)
  - Capacity: Minimum 50 GB available
  - IOPS: 1,000+ for production workloads
  - Permissions: Service user has read/write access

- [ ] **Network Configuration**
  - Port 8080 accessible for service
  - Port 9090 accessible for metrics (Prometheus)
  - Firewall rules configured
  - Load balancer configured (if using)

- [ ] **Dependencies Installed**
  - Rust toolchain (if building from source)
  - systemd (for service management)
  - Monitoring agents (Prometheus, Datadog, etc.)

### Configuration Readiness

- [ ] **Environment Variables Set**
  - `RFSOURCE_DATA_DIR` - Data storage location
  - `RFSOURCE_LOG_LEVEL` - Logging level (info/debug)
  - `RFSOURCE_MAX_MEMORY_MB` - Memory limit (500 MB recommended)
  - See [PRODUCTION-CONFIG-RFSOURCE.md](#file-production-config-rfsource) for full list

- [ ] **Service Files Created**
  - systemd service unit file at `/etc/systemd/system/rfsource.service`
  - Environment file at `/opt/rfsource/config/rfsource.env`
  - Binary at `/opt/rfsource/bin/rfsource`

- [ ] **Monitoring Infrastructure Ready**
  - Prometheus scraping configured
  - Grafana dashboards imported
  - AlertManager rules deployed
  - PagerDuty integration configured

### Testing Readiness

- [ ] **Load Test Results Reviewed**
  - Baseline metrics documented (see DOC-TEST-RFSOURCE-EXTENDED-001)
  - Alert thresholds configured based on baseline
  - Capacity limits understood (500 users validated, 2,000 safe, 5,000 max)

- [ ] **Smoke Tests Prepared**
  - Smoke test script at `/scripts/smoke_test_rfsource.sh`
  - Test data prepared
  - Expected results documented

- [ ] **Rollback Plan Documented**
  - Previous version binary backed up
  - Previous configuration backed up
  - Rollback procedures reviewed (see runbook)

### Team Readiness

- [ ] **On-Call Engineer Identified**
  - Primary: ___________________
  - Secondary: ___________________
  - PagerDuty schedule configured

- [ ] **Deployment Window Scheduled**
  - Date/Time: ___________________
  - Duration: ___________ (recommended: 2 hours)
  - Stakeholders notified

- [ ] **Communication Plan Ready**
  - Status page prepared
  - Slack channels configured (`#rfsource-incidents`, `#rfsource-ops`)
  - Email distribution list ready

- [ ] **Rollback Criteria Agreed**
  - Error rate > 5% for 2 minutes → Immediate rollback
  - Service down for > 1 minute → Immediate rollback
  - Critical functionality broken → Immediate rollback

---

## Deployment Steps

### Step 1: Pre-Deployment Verification

- [ ] Run pre-deployment checks
  ```bash
  # Verify infrastructure
  df -h /opt/rfsource/data  # Check disk space
  systemctl status rfsource  # Should not exist yet (or stopped)
  
  # Verify configuration
  cat /opt/rfsource/config/rfsource.env  # Review settings
  ```

- [ ] Backup current system state (if updating)
  ```bash
  # Backup previous version
  cp /opt/rfsource/bin/rfsource /opt/rfsource/bin/rfsource.previous
  cp /opt/rfsource/config/rfsource.env /opt/rfsource/config/rfsource.env.previous
  ```

- [ ] Notify stakeholders deployment starting
  - Post in `#rfsource-ops`: "RFSource deployment starting"
  - Update status page: "Maintenance in progress"

### Step 2: Deploy Service

- [ ] Deploy binary (choose option A or B from runbook)
  - [ ] Option A: Binary deployment (recommended)
  - [ ] Option B: Source build deployment

- [ ] Configure systemd service
  ```bash
  sudo systemctl daemon-reload
  sudo systemctl enable rfsource
  ```

- [ ] Set permissions
  ```bash
  sudo chown -R rfsource-service:rfsource-service /opt/rfsource
  sudo chmod +x /opt/rfsource/bin/rfsource
  ```

### Step 3: Start Service

- [ ] Start service
  ```bash
  sudo systemctl start rfsource
  ```

- [ ] Verify service is running
  ```bash
  sudo systemctl status rfsource
  # Expected: "active (running)"
  ```

- [ ] Check logs for errors
  ```bash
  sudo journalctl -u rfsource -n 50
  # Look for: "Service started successfully" or similar
  ```

### Step 4: Smoke Testing

- [ ] Run health check
  ```bash
  curl http://localhost:8080/health
  # Expected: {"status":"healthy","version":"1.0.0"}
  ```

- [ ] Run smoke test script
  ```bash
  /opt/rfsource/scripts/smoke_test_rfsource.sh
  # Expected: "✅ All smoke tests passed"
  ```

- [ ] Verify basic operations
  - [ ] Write operation successful
  - [ ] Read operation successful
  - [ ] List operation successful

### Step 5: Monitoring Verification

- [ ] Verify metrics are being collected
  ```bash
  curl http://localhost:8080/metrics
  # Expected: Prometheus format metrics
  ```

- [ ] Check Prometheus scraping
  - Visit Prometheus UI → Targets
  - Verify rfsource target is "UP"

- [ ] Check Grafana dashboards
  - Open Primary Operations Dashboard
  - Verify panels populating with data

- [ ] Verify alerts are configured
  - Check AlertManager rules loaded
  - Test alert (optional): trigger test alert

---

## Post-Deployment Validation

### Immediate Validation (0-15 minutes)

- [ ] **Service Health** (0-5 minutes)
  - [ ] Health endpoint returning 200 OK
  - [ ] No errors in logs
  - [ ] Process running (check with `ps aux | grep rfsource`)

- [ ] **Basic Functionality** (5-10 minutes)
  - [ ] Write test successful
  - [ ] Read test successful
  - [ ] Branch operation successful

- [ ] **Resource Usage Normal** (10-15 minutes)
  - [ ] CPU < 10% (baseline: 4.6%)
  - [ ] Memory < 50 MB (baseline: 27 MB)
  - [ ] No spike in error rate

### Short-Term Monitoring (15 minutes - 1 hour)

- [ ] **Latency Metrics** (Check every 15 minutes)
  - [ ] Read p95 < 200ms (baseline: 119ms)
  - [ ] Write p95 < 50ms (baseline: 15ms)
  - [ ] No latency spikes

- [ ] **Error Rate** (Check every 15 minutes)
  - [ ] Error rate < 1% (baseline: 0.35%)
  - [ ] No unexpected error types in logs

- [ ] **Throughput** (Check every 15 minutes)
  - [ ] Throughput consistent with traffic level
  - [ ] No throughput degradation

- [ ] **Alerts** (Monitor continuously)
  - [ ] No critical alerts fired
  - [ ] No warning alerts fired

### Extended Monitoring (1-24 hours)

- [ ] **Hour 1-2:** Monitor every 15 minutes
  - Check all metrics remain within baseline

- [ ] **Hour 2-4:** Monitor every 30 minutes
  - Look for slow memory growth
  - Verify throughput scales with traffic

- [ ] **Hour 4-24:** Monitor every hour
  - Compare to load test 24-hour projections
  - Memory should be < 256 MB
  - CPU should remain < 10%

---

## Rollback Triggers

**Immediately rollback if any of these occur:**

- [ ] **Service Down**
  - Health check failing for > 1 minute
  - Cannot restart service

- [ ] **High Error Rate**
  - Error rate > 5% for 2 minutes
  - Critical functionality broken

- [ ] **Extreme Performance Degradation**
  - Read p95 > 500ms for 5 minutes
  - Write p95 > 200ms for 5 minutes

- [ ] **Resource Exhaustion**
  - Memory > 1 GB
  - CPU > 50% sustained
  - Disk full

**Execute rollback procedure from runbook immediately.**

---

## Post-Deployment Actions

### Communication

- [ ] Update stakeholders
  - Post in `#rfsource-ops`: "RFSource deployed successfully"
  - Update status page: "All systems operational"

- [ ] Document deployment
  - Record deployment time
  - Record version deployed
  - Note any issues encountered

### Monitoring Setup

- [ ] Configure baseline alerts
  - Set thresholds per monitoring document
  - Enable PagerDuty notifications

- [ ] Schedule daily check-ins
  - Review metrics daily for first week
  - Weekly thereafter

### Next Steps

- [ ] Schedule 1-week review
  - Review metrics vs. baseline
  - Adjust alert thresholds if needed
  - Identify optimization opportunities

- [ ] Update documentation if needed
  - Document any deployment variations
  - Update runbook with lessons learned

---

## Sign-Off

**Deployment Team:**

- [ ] Deployer: _______________ Date: _____ Signature: ___________
- [ ] Reviewer: _______________ Date: _____ Signature: ___________
- [ ] On-Call: _______________ Date: _____ Signature: ___________

**Approval:**

- [ ] Engineering Lead: _____________ Date: _____ Signature: _______
- [ ] Operations Manager: ____________ Date: _____ Signature: _______

---

**Checklist Version:** 1.0.0  
**Last Updated:** 2026-05-07
