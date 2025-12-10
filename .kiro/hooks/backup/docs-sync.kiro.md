---
name: Auto-Sync Design Docs
description: Updates design documentation when domain models change
version: 1
trigger:
  event: onFileSave
  filePattern: "src/domains/**/models/*.ts"
---

A domain model file was modified in Webrana AI Proxy. Please:

1. **Review the model changes** - check if it aligns with PRD data models:
   - Users (id, email, password_hash, plan, created_at, email_verified)
   - ApiKeys (id, user_id, provider, label, encrypted_key, iv, is_valid)
   - ProxyApiKeys (id, user_id, key_prefix, key_hash)
   - ProxyRequests (id, user_id, model, provider, tokens_*, latency_ms, status_code)
   - Subscriptions (id, user_id, plan, status, midtrans_subscription_id)

2. **Update design docs** if needed:
   - `.kiro/specs/*/design.md` - Data Models section
   - Check if new fields affect API contracts
   - Verify Zod schemas match TypeScript interfaces

3. **Flag breaking changes**:
   - Database migration needed?
   - API versioning required?
   - Backward compatibility maintained?

Propose specific updates to keep documentation in sync.
