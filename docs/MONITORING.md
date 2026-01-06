# USDFC Terminal Monitoring Guide

## Health Endpoints

### GET /health

Full health check with all service statuses.

**Response:**
```json
{
  "status": "healthy|degraded|unhealthy",
  "version": "0.1.0",
  "uptime_secs": 3600,
  "checks": {
    "rpc": {"status": "ok", "latency_ms": 150},
    "blockscout": {"status": "ok", "latency_ms": 89},
    "subgraph": {"status": "ok", "latency_ms": 201},
    "gecko": {"status": "ok", "latency_ms": 167},
    "database": {"status": "ok", "latency_ms": 1}
  }
}
```

### GET /ready

Quick readiness probe. Returns `ok` if server is accepting requests.

---

## Recommended Alerts

| Alert | Condition | Severity |
|-------|-----------|----------|
| HealthDegraded | /health status != "healthy" for 5m | Warning |
| HealthUnhealthy | /health status == "unhealthy" for 2m | Critical |
| HighLatency | p99 latency > 2s for 5m | Warning |
| APIFailure | Any API check failed for 10m | Critical |
| DiskSpace | DB > 1GB | Warning |

---

## Log Monitoring

```bash
# Watch for errors in real-time
journalctl -u usdfc-terminal -f | grep -E "ERROR|WARN"

# Check last hour of errors
journalctl -u usdfc-terminal --since "1 hour ago" | grep ERROR
```

---

## Database Maintenance

```bash
# Check DB size
du -h data/metrics_history.db

# Vacuum database if size > 500MB
sqlite3 data/metrics_history.db "VACUUM;"

# Prune data older than 30 days
sqlite3 data/metrics_history.db "DELETE FROM metric_snapshots WHERE timestamp < strftime('%s','now','-30 days');"
```

---

## Key Metrics to Monitor

- **Response Time**: Target < 500ms p99
- **Error Rate**: Target < 0.1%
- **Memory Usage**: Target < 512MB (Production: 0.58 GB)
- **DB Size**: Target < 500MB
- **Cache Hit Rate**: Target > 80%

---

## Railway Deployment Metrics

**Production**: https://usdfc-terminal-cleaned-production.up.railway.app/

### Resource Usage

| Metric | Value |
|--------|-------|
| RAM | 0.58 GB |
| CPU | 0.03 vCPU |
| Egress | ~0.00 GB |
| Cost | ~$0.0002 per deployment |

### Railway CLI Commands

```bash
# View logs
railway logs

# Check deployment status
railway status

# View metrics
railway metrics

# Shell access
railway shell
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RPC_TIMEOUT_SECS` | 30 | RPC call timeout |
| `REFRESH_INTERVAL_MS` | 30000 | Auto-refresh interval |
| `TCR_DANGER_THRESHOLD` | 150.0 | TCR danger level |
| `HISTORY_RETENTION_SECS` | 604800 | 7 days of history |
