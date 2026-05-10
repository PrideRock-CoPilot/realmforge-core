---
doc_id: DOC-OPS-RFSOURCE-CONFIG-001
title: RFSource Production Configuration
status: active
version: 1.0.0
created_at: 2026-05-07
related_docs:
  - DOC-OPS-RFSOURCE-RUNBOOK-001
  - DOC-TEST-RFSOURCE-EXTENDED-001
---

# RFSource Production Configuration

**Version:** 1.0.0  
**Based on Load Test:** 500 users, 5-minute sustained load

---

## Environment Variables

### Required Configuration

```bash
# Data storage location (REQUIRED)
RFSOURCE_DATA_DIR=/opt/rfsource/data

# Logging configuration (REQUIRED)
RFSOURCE_LOG_LEVEL=info               # Options: debug, info, warn, error
RFSOURCE_LOG_FILE=/opt/rfsource/logs/rfsource.log

# Resource limits (REQUIRED)
RFSOURCE_MAX_MEMORY_MB=500           # Based on load test projection
RFSOURCE_MAX_CONNECTIONS=1000        # Based on 500-user test x2
```

### Optional Configuration

```bash
# Performance tuning
RFSOURCE_CACHE_SIZE_MB=100           # File cache size
RFSOURCE_IO_THREADS=4                # I/O thread pool size
RFSOURCE_WORKER_THREADS=8            # Worker thread pool size

# Service configuration
RFSOURCE_PORT=8080                   # Service port
RFSOURCE_METRICS_PORT=9090           # Metrics endpoint port
RFSOURCE_HEALTH_CHECK_INTERVAL=30    # Health check interval (seconds)

# Security (when auth implemented - REM-007)
RFSOURCE_AUTH_ENABLED=false          # Not yet implemented (F-003)
RFSOURCE_TLS_ENABLED=false           # Deploy behind reverse proxy for TLS
```

### Development/Debug Configuration

```bash
# Only use in non-production environments
RFSOURCE_LOG_LEVEL=debug
RFSOURCE_ENABLE_PROFILING=true
RFSOURCE_PPROF_PORT=6060
```

---

## Resource Limits

### Compute Resources

**Minimum Requirements:**
- CPU: 2 cores
- Memory: 2 GB (500 MB service + 1.5 GB overhead)
- Network: 100 Mbps

**Recommended for 500 Users:**
- CPU: 4 cores (observed usage: 4.6%, ample headroom)
- Memory: 4 GB (observed usage: 27 MB, safe margin)
- Network: 1 Gbps

**Recommended for 2,000 Users (Projected):**
- CPU: 8 cores (projected usage: ~20%)
- Memory: 8 GB (projected usage: ~104 MB)
- Network: 1 Gbps

### Storage Configuration

**Minimum:**
- Capacity: 50 GB available
- IOPS: 1,000 (mixed read/write)
- Latency: < 10ms (p95)

**Recommended:**
- Capacity: 500 GB available (allow for growth)
- IOPS: 5,000+ (SSD recommended)
- Latency: < 5ms (p95)
- Filesystem: ext4, XFS (NOT FAT32 - 2GB file limit)

**Known Limitation:** No explicit file size limit (REM-002). Monitor files approaching 1.5 GB.

---

## Scaling Configuration

### Vertical Scaling Triggers

**Scale up CPU when:**
- Average CPU > 15% sustained for 1 hour
- CPU spikes > 50% regularly

**Scale up Memory when:**
- Memory > 200 MB sustained
- Memory growth trend indicates > 500 MB within 7 days

**Scale up Storage when:**
- Disk usage > 70%
- Less than 14 days until full (based on growth rate)

### Horizontal Scaling Triggers

**Add instances when:**
- Approaching 2,000 concurrent users
- CPU > 25% sustained
- Read latency p95 > 150ms sustained
- Geographic distribution needed

**Known Limitation:** Multi-file repo splitting not implemented (REM-003). May impact horizontal scaling for very large repos.

---

## Configuration by Environment

### Development

```bash
# dev.env
RFSOURCE_DATA_DIR=/tmp/rfsource-dev
RFSOURCE_LOG_LEVEL=debug
RFSOURCE_MAX_MEMORY_MB=100
RFSOURCE_MAX_CONNECTIONS=10
RFSOURCE_PORT=8080
```

### Staging

```bash
# staging.env
RFSOURCE_DATA_DIR=/opt/rfsource-staging/data
RFSOURCE_LOG_LEVEL=info
RFSOURCE_MAX_MEMORY_MB=500
RFSOURCE_MAX_CONNECTIONS=500
RFSOURCE_PORT=8080
RFSOURCE_METRICS_PORT=9090
```

### Production

```bash
# production.env
RFSOURCE_DATA_DIR=/opt/rfsource/data
RFSOURCE_LOG_LEVEL=info
RFSOURCE_LOG_FILE=/opt/rfsource/logs/rfsource.log
RFSOURCE_MAX_MEMORY_MB=500
RFSOURCE_MAX_CONNECTIONS=1000
RFSOURCE_PORT=8080
RFSOURCE_METRICS_PORT=9090
RFSOURCE_HEALTH_CHECK_INTERVAL=30
RFSOURCE_CACHE_SIZE_MB=100
RFSOURCE_IO_THREADS=4
RFSOURCE_WORKER_THREADS=8
```

---

## Security Configuration

### Current Status

**⚠️ WARNING:** Authentication not yet implemented (F-003, REM-007).

**Production Deployment Requirements:**
- Deploy behind VPN or firewall
- Restrict network access to trusted sources
- Use reverse proxy (nginx, Envoy) for TLS termination

### Reverse Proxy Configuration (nginx)

```nginx
upstream rfsource {
    server localhost:8080;
}

server {
    listen 443 ssl http2;
    server_name rfsource.example.com;

    ssl_certificate /etc/ssl/certs/rfsource.crt;
    ssl_certificate_key /etc/ssl/private/rfsource.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://rfsource;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts based on load test latencies
        proxy_connect_timeout 5s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /metrics {
        # Restrict metrics to monitoring systems only
        allow 10.0.0.0/8;  # Internal network
        deny all;
        proxy_pass http://rfsource;
    }
}
```

---

## Backup Configuration

### Backup Strategy

**Frequency:**
- Full backup: Daily at 2 AM (low traffic)
- Incremental backup: Every 6 hours
- Retention: 30 days

**Backup Script Example:**

```bash
#!/bin/bash
# /opt/rfsource/scripts/backup_rfsource.sh

BACKUP_DIR=/backup/rfsource
DATE=$(date +%Y%m%d-%H%M%S)
DATA_DIR=/opt/rfsource/data

# Create backup directory
mkdir -p $BACKUP_DIR

# Stop service (optional - for consistent backup)
# sudo systemctl stop rfsource

# Backup data directory
tar -czf $BACKUP_DIR/rfsource-data-$DATE.tar.gz $DATA_DIR

# Start service (if stopped)
# sudo systemctl start rfsource

# Verify backup
if [ -f "$BACKUP_DIR/rfsource-data-$DATE.tar.gz" ]; then
    echo "Backup successful: rfsource-data-$DATE.tar.gz"
else
    echo "Backup failed!"
    exit 1
fi

# Clean old backups (keep 30 days)
find $BACKUP_DIR -name "rfsource-data-*.tar.gz" -mtime +30 -delete
```

### Restore Procedure

```bash
#!/bin/bash
# Restore from backup

BACKUP_FILE=$1  # Pass backup file as argument
DATA_DIR=/opt/rfsource/data

# Stop service
sudo systemctl stop rfsource

# Backup current data (just in case)
mv $DATA_DIR $DATA_DIR.before-restore

# Extract backup
tar -xzf $BACKUP_FILE -C /opt/rfsource/

# Start service
sudo systemctl start rfsource

# Verify
curl http://localhost:8080/health
```

---

## Logging Configuration

### Log Levels

**Production:** `info`
- Logs: Service start/stop, errors, warnings
- Volume: ~10 MB/day

**Debug:** `debug` (only for troubleshooting)
- Logs: All of the above + request details, internal state
- Volume: ~100+ MB/day (high volume)

### Log Rotation

```bash
# /etc/logrotate.d/rfsource
/opt/rfsource/logs/rfsource.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 rfsource-service rfsource-service
    sharedscripts
    postrotate
        systemctl reload rfsource
    endscript
}
```

### Log Shipping

**To Centralized Logging (Optional):**

```bash
# Filebeat configuration for ELK stack
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /opt/rfsource/logs/rfsource.log
    fields:
      service: rfsource
      environment: production
    json.keys_under_root: true

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "rfsource-%{+yyyy.MM.dd}"
```

---

## Configuration Management

### Version Control

**Store configuration files in Git:**

```bash
# Repository structure
rfsource-config/
├── environments/
│   ├── dev.env
│   ├── staging.env
│   └── production.env
├── systemd/
│   └── rfsource.service
├── nginx/
│   └── rfsource.conf
└── scripts/
    ├── deploy.sh
    └── rollback.sh
```

### Deployment Process

```bash
# Deploy configuration changes

# 1. Pull latest config from Git
cd /opt/rfsource-config
git pull

# 2. Copy environment file
sudo cp environments/production.env /opt/rfsource/config/rfsource.env

# 3. Reload service (if config supports hot reload)
sudo systemctl reload rfsource

# 4. Otherwise, restart
sudo systemctl restart rfsource

# 5. Verify
curl http://localhost:8080/health
```

---

**Document Version:** 1.0.0  
**Last Updated:** 2026-05-07
