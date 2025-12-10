# Requirements Document - Week 2: Multi-Provider + Frontend

## Introduction

Week 2 extends the proxy to support all 4 AI providers (OpenAI, Anthropic, Google AI, Qwen) with request transformation and streaming support. Additionally, the frontend landing page and user dashboard skeleton are built.

**Sprint Duration**: Dec 16-22, 2024
**Goal**: All providers working + Landing page + Dashboard skeleton

## Glossary

| Term | Definition |
|------|------------|
| Webrana_Proxy | The backend component that forwards requests to AI providers with format transformation |
| Webrana_Frontend | The Next.js 15 application serving landing page and user dashboard |
| SSE | Server-Sent Events - protocol for streaming responses from AI providers |
| Request_Transformer | Component that converts unified Webrana format to provider-specific formats |
| Usage_Logger | Component that records request metadata (tokens, latency, cost) for analytics |
| Landing_Page | The public-facing marketing page at webrana.id root URL |
| Dashboard_Layout | The authenticated user interface container with sidebar navigation |
| API_Keys_Page | The dashboard page for managing provider and proxy API keys |
| Auth_Forms | Login and registration form components with validation |
| Auth_Pages | The /login and /register pages for user authentication |

## Requirements

### Requirement 1: Anthropic (Claude) Proxy Support

**User Story:** As a developer, I want to access Claude models through Webrana, so that I can use my Anthropic API key via the unified endpoint.

#### Acceptance Criteria

1. WHEN a user sends a request to `/v1/chat/completions` with `model: "claude-3-opus"` or `model: "claude-3-sonnet"`, THE Webrana_Proxy SHALL transform and forward to Anthropic's Messages API
2. WHEN transforming a request, THE Request_Transformer SHALL convert OpenAI-style messages format to Anthropic's format (system message as separate parameter)
3. WHEN transforming a request, THE Request_Transformer SHALL map `max_tokens` parameter correctly for Anthropic (required field)
4. WHEN Anthropic returns a response, THE Webrana_Proxy SHALL transform it back to OpenAI-compatible format within 50ms
5. IF the user's Anthropic API key is not configured, THEN THE Webrana_Proxy SHALL return HTTP 400 with message "Anthropic API key not configured"

---

### Requirement 2: Google AI (Gemini) Proxy Support

**User Story:** As a developer, I want to access Gemini models through Webrana, so that I can use my Google AI API key via the unified endpoint.

#### Acceptance Criteria

1. WHEN a user sends a request with `model: "gemini-pro"` or `model: "gemini-1.5-pro"`, THE Webrana_Proxy SHALL transform and forward to Google's Generative AI API
2. WHEN transforming a request, THE Request_Transformer SHALL convert OpenAI-style messages to Google's `contents` format with `parts` array
3. WHEN transforming a request, THE Request_Transformer SHALL map `temperature`, `top_p`, and `max_tokens` to Google's parameter names
4. WHEN Google AI returns a response, THE Webrana_Proxy SHALL transform it back to OpenAI-compatible format within 50ms
5. IF the user's Google AI API key is not configured, THEN THE Webrana_Proxy SHALL return HTTP 400 with message "Google AI API key not configured"

---

### Requirement 3: Alibaba Qwen Proxy Support

**User Story:** As a developer, I want to access Qwen models through Webrana, so that I can use my Alibaba API key via the unified endpoint.

#### Acceptance Criteria

1. WHEN a user sends a request with `model: "qwen-turbo"` or `model: "qwen-plus"`, THE Webrana_Proxy SHALL transform and forward to Alibaba's DashScope API
2. WHEN transforming a request, THE Request_Transformer SHALL convert OpenAI-style messages to Qwen's input format
3. WHEN transforming a request, THE Request_Transformer SHALL handle Qwen-specific parameters (enable_search, result_format)
4. WHEN Qwen returns a response, THE Webrana_Proxy SHALL transform it back to OpenAI-compatible format within 50ms
5. IF the user's Qwen API key is not configured, THEN THE Webrana_Proxy SHALL return HTTP 400 with message "Qwen API key not configured"

---

### Requirement 4: Streaming Support (SSE)

**User Story:** As a developer, I want to receive streaming responses, so that I can display AI responses in real-time to my users.

#### Acceptance Criteria

1. WHEN a user sends a request with `stream: true`, THE Webrana_Proxy SHALL establish an SSE connection and stream chunks as they arrive from the provider
2. WHILE streaming, THE Webrana_Proxy SHALL transform each streaming chunk to OpenAI-compatible SSE format (`data: {...}\n\n`)
3. WHEN the stream completes, THE Webrana_Proxy SHALL send `data: [DONE]\n\n`
4. IF the upstream provider connection drops during streaming, THEN THE Webrana_Proxy SHALL close the client connection with appropriate error
5. WHEN streaming is requested, THE Webrana_Proxy SHALL support streaming for all 4 providers (OpenAI, Anthropic, Google, Qwen)

---

### Requirement 5: Usage Logging

**User Story:** As a user, I want my API usage logged, so that I can view analytics and track costs.

#### Acceptance Criteria

1. WHEN a proxy request completes, THE Usage_Logger SHALL record: user_id, provider, model, input_tokens, output_tokens, latency_ms, timestamp
2. WHEN logging a request, THE Usage_Logger SHALL calculate estimated cost based on provider pricing (stored in configuration)
3. WHEN logging a request, THE Usage_Logger SHALL write logs asynchronously to avoid adding latency to responses
4. WHEN storing logs, THE Usage_Logger SHALL store logs in the `proxy_requests` table with indexes on user_id and timestamp
5. IF token counting fails, THEN THE Usage_Logger SHALL estimate tokens using character count / 4 approximation

---

### Requirement 6: Landing Page

**User Story:** As a visitor, I want to see a professional landing page, so that I can understand what Webrana offers and sign up.

#### Acceptance Criteria

1. WHEN a visitor accesses `webrana.id`, THE Webrana_Frontend SHALL display the landing page within 2 seconds (LCP)
2. WHEN displaying content, THE Landing_Page SHALL include: hero section, feature highlights, pricing table (Rp 0/49K/99K/299K), and call-to-action buttons
3. WHEN displaying pricing, THE Landing_Page SHALL display pricing in Indonesian Rupiah (Rp) with proper thousand separators (e.g., Rp 49.000)
4. WHEN displaying features, THE Landing_Page SHALL include supported provider logos (OpenAI, Anthropic, Google, Alibaba)
5. WHEN rendering on different devices, THE Landing_Page SHALL be responsive (mobile, tablet, desktop) using Tailwind CSS breakpoints
6. WHEN displaying content, THE Landing_Page SHALL include ID/EN language toggle (default: ID)

---

### Requirement 7: User Dashboard Layout

**User Story:** As a logged-in user, I want a dashboard interface, so that I can manage my account and API keys.

#### Acceptance Criteria

1. WHEN an authenticated user accesses `/dashboard`, THE Webrana_Frontend SHALL display the dashboard layout with sidebar navigation
2. WHEN displaying navigation, THE Dashboard_Layout SHALL include navigation items: Overview, API Keys, Usage, Billing, Settings
3. WHEN displaying the header, THE Dashboard_Layout SHALL display the user's current plan tier and usage summary
4. IF a user is not authenticated, THEN THE Webrana_Frontend SHALL redirect to `/login` with return URL preserved
5. WHEN rendering components, THE Dashboard_Layout SHALL use shadcn/ui components for consistent styling

---

### Requirement 8: API Key Management UI

**User Story:** As a user, I want to manage my API keys through the dashboard, so that I can add, view, and delete provider keys.

#### Acceptance Criteria

1. WHEN a user accesses `/dashboard/api-keys`, THE Webrana_Frontend SHALL display a list of configured provider API keys (masked)
2. WHEN adding keys, THE API_Keys_Page SHALL provide forms to add new API keys for each supported provider (OpenAI, Anthropic, Google, Qwen)
3. WHEN a user adds an API key, THE Webrana_Frontend SHALL validate the key format before submission
4. WHEN displaying a key, THE API_Keys_Page SHALL provide a "Test Connection" button that verifies the API key works with the provider
5. WHEN a user clicks "Delete" on an API key, THE Webrana_Frontend SHALL show a confirmation dialog before deletion
6. WHEN displaying Proxy API keys, THE API_Keys_Page SHALL provide copy-to-clipboard functionality

---

### Requirement 9: Authentication UI

**User Story:** As a visitor, I want to sign up and log in through the web interface, so that I can access my dashboard.

#### Acceptance Criteria

1. WHEN a visitor accesses `/login`, THE Webrana_Frontend SHALL display a login form with email and password fields
2. WHEN a visitor accesses `/register`, THE Webrana_Frontend SHALL display a registration form with email, password, and password confirmation
3. WHEN validation fails, THE Auth_Forms SHALL display validation errors inline (email format, password requirements)
4. WHEN login succeeds, THE Webrana_Frontend SHALL redirect to `/dashboard/overview`
5. WHEN displaying the login form, THE Auth_Pages SHALL include "Forgot Password" link (implementation in Week 3)
6. WHEN validating input, THE Auth_Forms SHALL use Zod for client-side validation matching backend requirements
