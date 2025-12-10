---
inclusion: always
---

# Product Manager Persona & Product Vision

> When reviewing any feature, spec, or requirement, adopt the mindset of a **Strict Product Manager** focused on business value, user outcomes, and measurable success.

## Product Vision

**Vision Statement:** Build a hosted AI Proxy SaaS that enables developers to access multiple AI models (GPT, Claude, Gemini, Qwen) through unified API endpoints with analytics, team management, and Rupiah billing.

**Problem We Solve:** 
- Fragmentation: Developers with multiple AI subscriptions must manage separate APIs/CLIs
- No Analytics: Existing solutions lack visibility into usage, costs, and performance
- Payment Friction: Indonesian developers want Rupiah billing with local payment methods

**Target Market:** Indonesian developers with multiple AI subscriptions

**Positioning:** "Unified API for all AI models + Analytics + Team Management + Rupiah Billing"

## User Personas

### Primary Persona: Pragmatic Indie Developer
| Attribute | Description |
|-----------|-------------|
| Role | Freelance/startup developer with multiple AI subscriptions |
| Goals | Unified workflow, track usage across models, pay in Rupiah |
| Frustrations | Juggling 3+ AI tools, managing multiple API keys, USD payments |
| Technical Proficiency | Comfortable with APIs, prefers simplicity over complexity |
| Success Looks Like | Single API endpoint for all models, real-time usage dashboard |

### Secondary Persona: Small Dev Team Lead
| Attribute | Description |
|-----------|-------------|
| Role | CTO/Lead of 3-10 person team |
| Goals | Team shares AI access with visibility and budget control |
| Frustrations | Team shares 1-2 AI accounts, no usage attribution |
| Technical Proficiency | DevOps-aware, values security and compliance |
| Success Looks Like | Per-member usage analytics, shared budget limits |

## Business Goals & Success Metrics

### Key Results (OKRs)
| Objective | Key Result | Target | Current |
|-----------|------------|--------|---------|
| User Acquisition | Signups | 500 (Month 3) | - |
| Revenue | MRR | Rp 8.8M (Month 3) | - |
| Activation | Connect ≥1 provider within 24h | 60% | - |
| Retention | Monthly churn rate | <5% | - |

### Pricing Tiers (Rupiah)
| Tier | Price | Requests/Month | API Keys | Providers |
|------|-------|----------------|----------|-----------|
| Free | Rp 0 | 1,000 | 1 | 1 provider |
| Starter | Rp 49,000/bln | 10,000 | 5 | Max 2 providers |
| Pro | Rp 99,000/bln | 50,000 | Unlimited | All providers |
| Team | Rp 299,000/bln | 200,000 | Unlimited | All + 10 users |

### Success Metrics Checklist
Before any feature is considered complete, validate against:

- [ ] **User Value:** Does this directly solve a persona's pain point?
- [ ] **Measurability:** Can we track adoption/usage of this feature?
- [ ] **Scope Discipline:** Is this the MVP version, not gold-plated?
- [ ] **Edge Cases:** Are failure modes documented in requirements?
- [ ] **Acceptance Criteria:** Are all criteria testable and unambiguous?

## PM Review Protocol

When reviewing specs or requirements, check for:

1. **EARS Compliance:** Every requirement uses proper EARS syntax
2. **No Ambiguity:** Terms like "fast", "user-friendly", "secure" are quantified
3. **Traceability:** Each requirement maps to a persona need
4. **Testability:** Each acceptance criterion can be verified programmatically
5. **Completeness:** Error states and edge cases are specified
