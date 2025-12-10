# Load Test Guide - Week 3 Task 22

> **Untuk: DevOps Team**  
> **Tanggal: December 2024**  
> **Status: Menunggu Eksekusi**

## Overview

Task 22 memerlukan load testing untuk memvalidasi performa Webrana AI Proxy dengan 100 concurrent users selama 10 menit.

---

## Requirements

### Performance Targets (Requirements 9.1-9.4)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Concurrent Users | 100 | k6 VUs |
| Test Duration | 10 minutes | k6 stages |
| p95 Latency | < 500ms | http_req_duration |
| Error Rate | 0% | errors metric |

---

## Prerequisites

### Install k6

**macOS:**
```bash
brew install k6
```

**Ubuntu/Debian:**
```bash
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

**Windows:**
```powershell
choco install k6
# or
winget install k6
```

**Docker:**
```bash
docker pull grafana/k6
```

---

## Running the Load Test

### Basic Run

```bash
# From project root
k6 run infrastructure/scripts/load-test.js
```

### With Custom Base URL

```bash
# Against staging
k6 run -e BASE_URL=https://staging-api.webrana.id infrastructure/scripts/load-test.js

# Against production
k6 run -e BASE_URL=https://api.webrana.id infrastructure/scripts/load-test.js
```

### With Test User Credentials

```bash
k6 run \
  -e BASE_URL=https://api.webrana.id \
  -e TEST_EMAIL=loadtest@webrana.id \
  -e TEST_PASSWORD=SecurePassword123! \
  infrastructure/scripts/load-test.js
```

### Using Docker

```bash
docker run --rm -i grafana/k6 run - < infrastructure/scripts/load-test.js
```

---

## Test Stages

The load test runs in 3 stages:

```
Stage 1: Ramp Up (2 minutes)
├── 0 → 100 VUs gradually
└── Simulates traffic increase

Stage 2: Sustained Load (10 minutes)
├── 100 VUs constant
└── Main test period

Stage 3: Ramp Down (1 minute)
├── 100 → 0 VUs gradually
└── Graceful shutdown
```

---

## Endpoints Tested

| Endpoint | Weight | Auth Required |
|----------|--------|---------------|
| GET /auth/me | 30% | Yes |
| GET /usage | 15% | Yes |
| GET /usage/summary | 15% | Yes |
| GET /billing/subscription | 10% | Yes |
| GET /billing/invoices | 10% | Yes |
| GET /health | 20% | No |

---

## Interpreting Results

### Success Criteria

```
✓ PASS if:
  - http_req_duration p95 < 500ms
  - errors rate < 1%
  - All thresholds pass

✗ FAIL if:
  - Any threshold exceeded
  - Server errors (5xx) > 0
  - Connection failures
```

### Sample Output

```
==========================================================
LOAD TEST SUMMARY - Webrana AI Proxy
==========================================================

Duration: 780s
Max VUs: 100
Total Requests: 45,230

LATENCY:
  p50: 45ms
  p95: 187ms
  p99: 342ms

ERROR RATE:
  0.00%

THRESHOLDS:
  p95 < 500ms: ✓ PASS
  Error rate < 1%: ✓ PASS

==========================================================
```

---

## Troubleshooting

### High Latency

**Possible causes:**
1. Database connection pool exhausted
2. Redis connection issues
3. Insufficient server resources
4. Network latency

**Solutions:**
```bash
# Check database connections
SELECT count(*) FROM pg_stat_activity;

# Check Redis connections
redis-cli INFO clients

# Monitor server resources
htop
docker stats
```

### High Error Rate

**Possible causes:**
1. Rate limiting triggered
2. Authentication failures
3. Server overload
4. Database deadlocks

**Solutions:**
```bash
# Check application logs
docker logs webrana-backend --tail 1000 | grep ERROR

# Check rate limit keys in Redis
redis-cli KEYS "rate:*"

# Check for database locks
SELECT * FROM pg_locks WHERE NOT granted;
```

### Connection Refused

**Possible causes:**
1. Server not running
2. Firewall blocking
3. Wrong port/URL

**Solutions:**
```bash
# Verify server is running
curl -v http://localhost:3000/health

# Check firewall
sudo ufw status

# Check Docker network
docker network ls
docker network inspect webrana_network
```

---

## Monitoring During Test

### Real-time Metrics

```bash
# Watch server resources
watch -n 1 'docker stats --no-stream'

# Watch database connections
watch -n 5 'docker exec webrana-postgres psql -U webrana -c "SELECT count(*) FROM pg_stat_activity;"'

# Watch Redis
watch -n 5 'docker exec webrana-redis redis-cli INFO stats | grep instantaneous'
```

### Grafana Dashboard (if available)

1. Open Grafana: http://localhost:3001
2. Navigate to "Webrana Performance" dashboard
3. Set time range to "Last 15 minutes"
4. Watch metrics during test

---

## Reporting

### Generate Report

After test completion, results are saved to:
- `load-test-results.json` - Machine-readable summary
- Console output - Human-readable summary

### Report Template

```markdown
## Load Test Report - Webrana AI Proxy

**Date:** [DATE]
**Tester:** [NAME]
**Environment:** [staging/production]
**Duration:** 13 minutes (2m ramp-up + 10m sustained + 1m ramp-down)

### Configuration
- Max VUs: 100
- Base URL: [URL]
- Test Script: infrastructure/scripts/load-test.js

### Results

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| p95 Latency | < 500ms | [X]ms | ✓/✗ |
| Error Rate | < 1% | [X]% | ✓/✗ |
| Total Requests | - | [X] | - |
| Requests/sec | - | [X] | - |

### Latency Distribution
- p50: [X]ms
- p95: [X]ms
- p99: [X]ms
- max: [X]ms

### Observations
[Notes about any issues, anomalies, or recommendations]

### Conclusion
[PASS/FAIL] - [Summary statement]
```

---

## Next Steps After Testing

1. **If PASS:**
   - Document results
   - Archive test artifacts
   - Proceed to production deployment

2. **If FAIL:**
   - Identify bottlenecks
   - Implement optimizations
   - Re-run tests
   - Repeat until passing

---

## Contact

Jika ada pertanyaan:
- Backend Team: [contact]
- DevOps Lead: [contact]
- Performance Engineer: [contact]
