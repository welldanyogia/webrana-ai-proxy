---
name: Proxy Logic Review
description: Reviews AI proxy routing and transformation logic for correctness
version: 1
trigger:
  event: onFileSave
  filePattern: "src/domains/proxy/**/*.ts"
---

🔄 **Proxy Logic Review**

Proxy-related code was modified. Verify against API Key MVP spec:

**Provider Routing**:
1. Model detection correct?
   - `gpt-*` → OpenAI
   - `claude-*` → Anthropic
   - `gemini-*` → Google AI
   - `qwen-*` → Alibaba

**Request Transformation**:
2. OpenAI format → Anthropic format correct?
3. OpenAI format → Google Gemini format correct?
4. OpenAI format → Qwen format correct?

**Response Transformation**:
5. All responses normalized to OpenAI format?
6. Streaming (SSE) passthrough working?
7. Error responses properly formatted?

**Usage Tracking**:
8. Tokens counted accurately (input + output)?
9. Latency measured correctly?
10. Cost estimation based on provider pricing?

**Rate Limiting**:
11. Per-plan limits enforced?
    - Free: 1K/month
    - Starter: 10K/month
    - Pro: 50K/month
    - Team: 200K/month
12. Redis-backed distributed limiting?

**Performance**:
13. Proxy overhead <100ms?
14. Connection pooling configured?

Flag any issues that could affect user experience.
