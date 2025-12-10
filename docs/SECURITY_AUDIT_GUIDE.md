# Security Audit Guide - Week 3 Task 21

> **Untuk: DevOps Team**  
> **Tanggal: December 2024**  
> **Status: Menunggu Eksekusi**

## Overview

Task 21 memerlukan security audit menyeluruh untuk Webrana AI Proxy sebelum production deployment. Dokumen ini berisi panduan lengkap untuk menjalankan audit keamanan.

---

## 21.1 OWASP Security Checks

### SQL Injection Testing

**Tools yang direkomendasikan:**
- SQLMap
- OWASP ZAP

**Endpoints untuk ditest:**
```
POST /auth/register
POST /auth/login
GET  /admin/users?search=<payload>
GET  /usage/export?start_date=<payload>&end_date=<payload>
POST /billing/subscribe
```

**Test payloads:**
```sql
' OR '1'='1
'; DROP TABLE users; --
1' AND '1'='1
UNION SELECT * FROM users --
```

**Expected result:** Semua payload harus di-reject atau di-escape dengan benar.

**Catatan implementasi:**
- Backend menggunakan SQLx dengan parameterized queries (`query!` dan `query_as!` macros)
- Tidak ada string concatenation untuk SQL queries
- Semua input di-bind sebagai parameter

### XSS (Cross-Site Scripting) Testing

**Tools:**
- OWASP ZAP
- Burp Suite

**Test locations:**
```
- User name field saat registrasi
- API key name/label
- Search fields di admin dashboard
- Invoice display (HTML rendering)
```

**Test payloads:**
```html
<script>alert('XSS')</script>
<img src=x onerror=alert('XSS')>
javascript:alert('XSS')
<svg onload=alert('XSS')>
```

**Expected result:** Semua payload harus di-escape atau di-sanitize.

### CSRF (Cross-Site Request Forgery) Testing

**Endpoints yang perlu CSRF protection:**
```
POST /auth/logout
POST /billing/subscribe
POST /billing/subscription/cancel
POST /admin/users/{id}/suspend
POST /admin/users/{id}/plan
DELETE /api-keys/{id}
```

**Verification:**
- Pastikan semua state-changing requests memerlukan valid session/token
- Verify Origin/Referer header checking
- Test dengan request dari domain berbeda

---

## 21.2 HTTPS and TLS Configuration

### TLS Version Check

**Command:**
```bash
# Check supported TLS versions
nmap --script ssl-enum-ciphers -p 443 webrana.id

# Or using testssl.sh
./testssl.sh webrana.id
```

**Requirements:**
- [ ] TLS 1.2 minimum (TLS 1.3 preferred)
- [ ] TLS 1.0 dan 1.1 harus disabled
- [ ] SSL 2.0 dan 3.0 harus disabled

### Certificate Validation

**Command:**
```bash
# Check certificate details
openssl s_client -connect webrana.id:443 -servername webrana.id

# Check certificate chain
openssl s_client -connect webrana.id:443 -showcerts
```

**Requirements:**
- [ ] Valid certificate dari trusted CA
- [ ] Certificate tidak expired
- [ ] Certificate chain lengkap
- [ ] Common Name atau SAN match dengan domain

### Cipher Suite Check

**Strong ciphers yang direkomendasikan:**
```
TLS_AES_256_GCM_SHA384
TLS_CHACHA20_POLY1305_SHA256
TLS_AES_128_GCM_SHA256
ECDHE-RSA-AES256-GCM-SHA384
ECDHE-RSA-AES128-GCM-SHA256
```

**Weak ciphers yang harus disabled:**
```
RC4
DES
3DES
MD5
SHA1 (untuk signatures)
```

---

## 21.3 Security Headers

### Required Headers

Tambahkan headers berikut di Nginx/reverse proxy atau Axum middleware:

```nginx
# Nginx configuration
add_header X-Frame-Options "DENY" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
add_header Permissions-Policy "geolocation=(), microphone=(), camera=()" always;

# Content Security Policy
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline' https://app.sandbox.midtrans.com https://app.midtrans.com; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' https://api.webrana.id; frame-src https://app.sandbox.midtrans.com https://app.midtrans.com;" always;

# HSTS (enable after confirming HTTPS works)
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
```

### Verification

**Command:**
```bash
# Check security headers
curl -I https://webrana.id

# Or use online tool
# https://securityheaders.com/?q=webrana.id
```

**Checklist:**
- [ ] X-Frame-Options: DENY
- [ ] X-Content-Type-Options: nosniff
- [ ] X-XSS-Protection: 1; mode=block
- [ ] Content-Security-Policy: configured
- [ ] Strict-Transport-Security: enabled
- [ ] Referrer-Policy: strict-origin-when-cross-origin

---

## 21.4 PII Exposure Audit

### Log Files Review

**Locations to check:**
```
/var/log/webrana/
Docker logs: docker logs <container_id>
Application logs via tracing
```

**PII yang TIDAK boleh ada di logs:**
- [ ] Email addresses (kecuali masked: u***@example.com)
- [ ] Passwords (plain atau hashed)
- [ ] API keys (plain)
- [ ] Credit card numbers
- [ ] Phone numbers
- [ ] Full names (dalam context sensitif)

**Grep commands untuk audit:**
```bash
# Check for email patterns
grep -rE '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}' /var/log/webrana/

# Check for API key patterns
grep -rE 'sk-[a-zA-Z0-9]{20,}' /var/log/webrana/
grep -rE 'AIza[a-zA-Z0-9_-]{35}' /var/log/webrana/

# Check for password fields
grep -ri 'password' /var/log/webrana/
```

### Database Encryption Audit

**Verify encrypted fields:**
```sql
-- Check API keys are encrypted (should be binary/bytea, not plaintext)
SELECT id, LENGTH(encrypted_key) as key_length, 
       encrypted_key NOT LIKE 'sk-%' as is_encrypted
FROM api_keys LIMIT 5;

-- Verify no plaintext passwords
SELECT id, LENGTH(password_hash) as hash_length,
       password_hash LIKE '$argon2%' as is_argon2
FROM users LIMIT 5;
```

### Response Data Review

**Check API responses don't leak sensitive data:**
```bash
# Test user endpoint doesn't return password
curl -H "Authorization: Bearer <token>" https://api.webrana.id/auth/me | jq

# Verify API key list masks keys
curl -H "Authorization: Bearer <token>" https://api.webrana.id/api-keys | jq
```

**Expected masking:**
- API keys: `sk-****...****1234`
- Emails in logs: `u***@example.com`
- Passwords: Never returned

---

## Execution Checklist

### Pre-Audit Setup
- [ ] Setup OWASP ZAP atau Burp Suite
- [ ] Prepare test environment (staging)
- [ ] Backup database sebelum testing
- [ ] Notify team tentang security testing

### Audit Execution
- [ ] 21.1.1 SQL Injection tests completed
- [ ] 21.1.2 XSS tests completed
- [ ] 21.1.3 CSRF tests completed
- [ ] 21.2.1 TLS version verified
- [ ] 21.2.2 Certificate validated
- [ ] 21.2.3 Cipher suites checked
- [ ] 21.3.1 Security headers added
- [ ] 21.3.2 Headers verified
- [ ] 21.4.1 Log files audited
- [ ] 21.4.2 Database encryption verified
- [ ] 21.4.3 API responses checked

### Post-Audit
- [ ] Document all findings
- [ ] Create tickets untuk issues found
- [ ] Implement fixes
- [ ] Re-test after fixes
- [ ] Sign-off dari security lead

---

## Reporting Template

```markdown
## Security Audit Report - Webrana AI Proxy

**Date:** [DATE]
**Auditor:** [NAME]
**Environment:** [staging/production]

### Summary
- Total tests: X
- Passed: X
- Failed: X
- Critical issues: X
- High issues: X
- Medium issues: X
- Low issues: X

### Findings

#### [CRITICAL/HIGH/MEDIUM/LOW] - Issue Title
- **Location:** [endpoint/file]
- **Description:** [what was found]
- **Impact:** [potential damage]
- **Recommendation:** [how to fix]
- **Status:** [open/fixed/accepted risk]

### Conclusion
[Overall assessment and recommendations]
```

---

## Contact

Jika ada pertanyaan atau butuh clarification:
- Backend Team: [contact]
- Security Lead: [contact]
- DevOps Lead: [contact]
