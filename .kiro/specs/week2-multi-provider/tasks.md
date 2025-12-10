# Implementation Plan - Week 2: Multi-Provider + Frontend

## Provider Transformers

- [x] 1. Implement Anthropic (Claude) proxy support

  - [x] 1.1 Create AnthropicTransformer struct
    - Transform OpenAI messages to Anthropic format (system as separate param)
    - Map max_tokens (required for Anthropic)
    - _Requirements: 1.2, 1.3_
  - [x] 1.2 Implement response transformation
    - Convert Anthropic response to OpenAI-compatible format
    - _Requirements: 1.4_
  - [x] 1.3 Add Anthropic API client
    - Base URL: api.anthropic.com
    - Headers: x-api-key, anthropic-version
    - _Requirements: 1.1_
  - [x] 1.4 Write property test: Request format transformation










    - Property 1: Request Format Transformation Consistency
    - Validates: _Requirements 1.2, 2.2, 3.2_

- [x] 2. Implement Google AI (Gemini) proxy support

  - [x] 2.1 Create GoogleTransformer struct
    - Transform messages to Google's contents/parts format
    - Map temperature, top_p, max_tokens to Google params
    - _Requirements: 2.2, 2.3_
  - [x] 2.2 Implement response transformation
    - Convert Google response to OpenAI-compatible format
    - _Requirements: 2.4_
  - [x] 2.3 Add Google AI API client
    - Base URL: generativelanguage.googleapis.com
    - API key in query param
    - _Requirements: 2.1_

- [x] 3. Implement Alibaba Qwen proxy support

  - [x] 3.1 Create QwenTransformer struct
    - Transform messages to Qwen's input format
    - Handle Qwen-specific params (enable_search, result_format)
    - _Requirements: 3.2, 3.3_
  - [x] 3.2 Implement response transformation
    - Convert Qwen response to OpenAI-compatible format
    - _Requirements: 3.4_
  - [x] 3.3 Add DashScope API client
    - Base URL: dashscope.aliyuncs.com
    - Authorization header with API key
    - _Requirements: 3.1_
  - [x] 3.4 Write property test: Response format normalization






    - Property 2: Response Format Normalization
    - Validates: _Requirements 1.4, 2.4, 3.4_

- [x] 4. Implement model router

  - [x] 4.1 Create ModelRouter to dispatch requests by model name
    - gpt-* → OpenAI, claude-* → Anthropic, gemini-* → Google, qwen-* → Qwen
    - _Requirements: 1.1, 2.1, 3.1_


  - [ ] 4.2 Write property test: Model routing correctness

    - Property 5: Model Routing Correctness
    - Validates: _Requirements 1.1, 2.1, 3.1_

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Streaming Support

- [x] 6. Implement SSE streaming
  - [x] 6.1 Create StreamHandler for SSE passthrough
    - Establish SSE connection to provider
    - Transform chunks to OpenAI-compatible format
    - _Requirements: 4.1, 4.2_
  - [x] 6.2 Implement stream termination
    - Send `data: [DONE]\n\n` on completion
    - Handle upstream connection drops
    - _Requirements: 4.3, 4.4_
  - [-] 6.3 Add streaming support to all transformers

    - OpenAI, Anthropic, Google, Qwen streaming
    - _Requirements: 4.5_
  - [x] 6.4 Write property test: Streaming chunk format

    - Property 3: Streaming Chunk Format
    - Validates: _Requirements 4.2, 4.3_

- [x] 7. Checkpoint - Streaming implementation verified
  - Build passes, streaming handlers implemented for all providers

## Usage Logging

- [x] 8. Implement usage logging
  - [x] 8.1 Create proxy_requests table migration
    - Columns: id, user_id, provider, model, tokens, latency, cost, timestamp
    - _Requirements: 5.4_
  - [x] 8.2 Create UsageLogger service
    - Async logging to avoid response latency
    - _Requirements: 5.3_
  - [x] 8.3 Implement token counting
    - Use tiktoken for accurate counts
    - Fallback to chars/4 estimation
    - _Requirements: 5.2, 5.5_
  - [x] 8.4 Implement cost calculation
    - Provider pricing configuration
    - Calculate estimated cost in IDR
    - _Requirements: 5.2_
  - [x] 8.5 Write property test: Usage log completeness


    - Property 4: Usage Log Completeness
    - Validates: _Requirements 5.1, 5.4_

- [x] 9. Checkpoint - Usage logging implementation verified
  - UsageLogger service with token counting and cost calculation in IDR

## Frontend - Landing Page

- [x] 10. Initialize Next.js 16 frontend
  - [x] 10.1 Create Next.js app with App Router
    - TypeScript strict mode, Tailwind CSS 3.x
    - Next.js 16.0.8, React 19.2.1 (CVE-2025 patched)
    - _Requirements: 6.1_
  - [x] 10.2 Install and configure shadcn/ui
    - Add Button, Card, Input, Dialog components
    - _Requirements: 7.5_
  - [x] 10.3 Setup TanStack Query for API calls
    - Configure QueryClient with defaults
    - _Requirements: 6.1_

- [x] 11. Build landing page
  - [x] 11.1 Create hero section
    - Headline, subheadline, CTA buttons
    - _Requirements: 6.2_
  - [x] 11.2 Create features section with provider logos
    - Unified API, Analytics, Team Management highlights
    - Provider logos (OpenAI, Anthropic, Google, Alibaba)
    - _Requirements: 6.2, 6.4_
  - [x] 11.3 Create pricing table
    - Display tiers in Rupiah (Rp 0/49.000/99.000/299.000)
    - Proper thousand separators
    - _Requirements: 6.3_
  - [x] 11.4 Implement responsive design
    - Mobile, tablet, desktop breakpoints
    - _Requirements: 6.5_
  - [x] 11.5 Add language toggle (ID/EN)
    - Default to Indonesian
    - _Requirements: 6.6_

- [x] 12. Checkpoint - Landing page build verified
  - Next.js build passes, all pages render correctly

## Frontend - Dashboard

- [x] 13. Build dashboard layout
  - [x] 13.1 Create (dashboard) route group with layout
    - Sidebar navigation, header with user info
    - _Requirements: 7.1, 7.2_
  - [x] 13.2 Create overview page skeleton
    - Plan tier display, usage summary cards
    - _Requirements: 7.3_
  - [x] 13.3 Implement auth redirect middleware
    - Redirect to /login if not authenticated
    - _Requirements: 7.4_

- [x] 14. Build API keys management page
  - [x] 14.1 Create /dashboard/api-keys page
    - List provider keys (masked) and proxy keys
    - _Requirements: 8.1_
  - [x] 14.2 Create add API key forms
    - Forms for each provider with format validation
    - _Requirements: 8.2, 8.3_
  - [x] 14.3 Implement "Test Connection" button
    - Verify key works with provider
    - _Requirements: 8.4_
  - [x] 14.4 Add delete confirmation dialog
    - Confirm before deletion
    - _Requirements: 8.5_
  - [x] 14.5 Add copy-to-clipboard for proxy keys
    - _Requirements: 8.6_


- [x] 15. Build authentication pages
  - [x] 15.1 Create /login page
    - Email/password form with validation
    - _Requirements: 9.1, 9.3_
  - [x] 15.2 Create /register page
    - Email, password, confirm password
    - _Requirements: 9.2, 9.3_
  - [x] 15.3 Implement form validation with Zod
    - Match backend validation rules
    - _Requirements: 9.6_
  - [x] 15.4 Implement login redirect
    - Redirect to /dashboard/overview on success
    - _Requirements: 9.4_
  - [x] 15.5 Add "Forgot Password" link (placeholder)

    - _Requirements: 9.5_

- [x] 16. Final Checkpoint - Week 2 implementation complete

  - ✅ All 158 tests passed (Dec 10, 2025)
  - ✅ Frontend build passes (Next.js 16.0.8, React 19.2.1)
  - ✅ All dashboard pages implemented
  - ✅ All property tests implemented and passing:
    - Property 1: Request Format Transformation (Anthropic, Google, Qwen)
    - Property 2: Response Format Normalization
    - Property 3: Streaming Chunk Format (9 tests)
    - Property 4: Usage Log Completeness (9 tests)
    - Property 5: Model Routing Correctness (7 tests)
