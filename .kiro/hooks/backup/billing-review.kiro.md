---
name: Billing Integration Review
description: Reviews Midtrans payment integration changes for correctness and security
version: 1
trigger:
  event: onFileSave
  filePattern: "src/domains/billing/**/*.ts"
---

💳 **Billing Integration Review** (Midtrans)

A billing-related file was modified. Verify against PRD pricing:

**Pricing Tiers (Rupiah)**:
- Free: Rp 0 (1K requests, 1 provider, 1 account)
- Starter: Rp 49,000/bln (10K requests, 2 providers, 5 accounts)
- Pro: Rp 99,000/bln (50K requests, all providers, unlimited)
- Team: Rp 299,000/bln (200K requests, 10 users)

**Payment Methods Check**:
1. QRIS enabled? (GoPay, OVO, Dana, LinkAja, ShopeePay)
2. Virtual Account enabled? (BCA, BNI, BRI, Mandiri, Permata)
3. Credit/Debit Card enabled? (Visa, Mastercard, JCB)

**Webhook Security**:
4. Midtrans signature verification implemented?
5. Idempotency handling (duplicate webhooks)?
6. Proper error handling for failed payments?

**Invoice Generation**:
7. PPN 11% tax calculation correct?
8. PDF generation working?
9. Email delivery configured?

**Subscription Lifecycle**:
10. Upgrade/downgrade prorated correctly?
11. Cancellation at end of period?
12. Failed payment retry logic?

Report any discrepancies with PRD.
