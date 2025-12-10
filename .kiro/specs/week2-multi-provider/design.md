# Design Document - Week 2: Multi-Provider + Frontend

## Overview

Week 2 extends the proxy to support all 4 AI providers (OpenAI, Anthropic, Google AI, Qwen) with request/response transformation and SSE streaming. The frontend landing page and user dashboard skeleton are built using Next.js 15 with App Router.

## Architecture

```mermaid
graph TD
    subgraph "Frontend (Next.js 15)"
        A[Landing Page] --> B[Auth Pages]
        B --> C[Dashboard Layout]
        C --> D[API Keys Page]
        C --> E[Usage Page]
        C --> F[Settings Page]
    end
    
    subgraph "Backend Proxy"
        G[/v1/chat/completions] --> H{Model Router}
        H -->|gpt-*| I[OpenAI Transformer]
        H -->|claude-*| J[Anthropic Transformer]
        H -->|gemini-*| K[Google Transformer]
        H -->|qwen-*| L[Qwen Transformer]
    end
    
    subgraph "External Providers"
        I --> M[OpenAI API]
        J --> N[Anthropic API]
        K --> O[Google AI API]
        L --> P[DashScope API]
    end
    
    subgraph "Usage Tracking"
        I & J & K & L --> Q[Usage Logger]
        Q --> R[(PostgreSQL)]
    end
```

## Components and Interfaces

### Backend Components

### Component 1: RequestTransformer

**Purpose:** Transforms unified Webrana request format to provider-specific formats.

**Interface:**
```rust
pub trait RequestTransformer {
    fn transform_request(&self, request: &ChatCompletionRequest) -> ProviderRequest;
    fn transform_response(&self, response: ProviderResponse) -> ChatCompletionResponse;
    fn transform_stream_chunk(&self, chunk: ProviderChunk) -> StreamChunk;
}

pub struct OpenAITransformer;
pub struct AnthropicTransformer;
pub struct GoogleTransformer;
pub struct QwenTransformer;
```

### Component 2: UsageLogger

**Purpose:** Asynchronously logs request metadata for analytics.

**Interface:**
```rust
pub trait UsageLogger {
    async fn log_request(&self, log: UsageLog) -> Result<(), LogError>;
    async fn get_usage_stats(&self, user_id: Uuid, period: Period) -> Result<UsageStats, LogError>;
}

pub struct UsageLog {
    pub user_id: Uuid,
    pub provider: Provider,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub latency_ms: u32,
    pub estimated_cost_idr: i64,
    pub timestamp: DateTime<Utc>,
}
```

### Component 3: StreamHandler

**Purpose:** Handles SSE streaming from providers and transforms to unified format.

**Interface:**
```rust
pub trait StreamHandler {
    async fn stream_response(
        &self,
        provider_stream: impl Stream<Item = ProviderChunk>,
    ) -> impl Stream<Item = StreamChunk>;
}

pub struct StreamChunk {
    pub id: String,
    pub choices: Vec<StreamChoice>,
    pub model: String,
}
```

### Frontend Components

### Component 4: LandingPage

**Purpose:** Public-facing marketing page with hero, features, and pricing sections.

**Structure:**
```typescript
// src/app/page.tsx - Landing page route
// Sections: HeroSection, FeaturesSection, PricingSection, FooterSection

interface PricingTier {
  name: string;
  price: number;           // In IDR (0, 49000, 99000, 299000)
  requests: number;
  apiKeys: number | 'unlimited';
  providers: string;
  features: string[];
}

interface LanguageToggle {
  locale: 'id' | 'en';
  setLocale: (locale: 'id' | 'en') => void;
}
```

**Design Decisions:**
- Default language: Indonesian (ID) per Requirement 6.6
- Pricing displayed with thousand separators (Rp 49.000) per Requirement 6.3
- Provider logos: OpenAI, Anthropic, Google, Alibaba per Requirement 6.4
- Responsive breakpoints: mobile (<640px), tablet (640-1024px), desktop (>1024px)

### Component 5: DashboardLayout

**Purpose:** Authenticated user interface container with sidebar navigation.

**Structure:**
```typescript
// src/app/(dashboard)/layout.tsx

interface DashboardLayoutProps {
  children: React.ReactNode;
}

interface NavigationItem {
  label: string;
  href: string;
  icon: React.ComponentType;
}

// Navigation items per Requirement 7.2
const navItems: NavigationItem[] = [
  { label: 'Overview', href: '/dashboard/overview', icon: HomeIcon },
  { label: 'API Keys', href: '/dashboard/api-keys', icon: KeyIcon },
  { label: 'Usage', href: '/dashboard/usage', icon: ChartIcon },
  { label: 'Billing', href: '/dashboard/billing', icon: CreditCardIcon },
  { label: 'Settings', href: '/dashboard/settings', icon: SettingsIcon },
];
```

**Design Decisions:**
- Uses shadcn/ui components per Requirement 7.5
- Header displays user's plan tier and usage summary per Requirement 7.3
- Auth redirect handled by Next.js middleware per Requirement 7.4

### Component 6: ApiKeysPage

**Purpose:** Manage provider API keys and proxy API keys.

**Structure:**
```typescript
// src/app/(dashboard)/dashboard/api-keys/page.tsx

interface ProviderApiKey {
  id: string;
  provider: 'openai' | 'anthropic' | 'google' | 'qwen';
  maskedKey: string;       // e.g., "sk-...abc123"
  createdAt: Date;
  lastUsed?: Date;
}

interface ProxyApiKey {
  id: string;
  name: string;
  maskedKey: string;
  createdAt: Date;
  lastUsed?: Date;
}

// Key format validation patterns per Requirement 8.3
const keyPatterns = {
  openai: /^sk-[a-zA-Z0-9]{48}$/,
  anthropic: /^sk-ant-[a-zA-Z0-9-]{95}$/,
  google: /^[a-zA-Z0-9_-]{39}$/,
  qwen: /^sk-[a-zA-Z0-9]{32}$/,
};
```

**Design Decisions:**
- Keys displayed masked (first 3 + last 6 chars) per Requirement 8.1
- "Test Connection" button validates key with provider per Requirement 8.4
- Delete requires confirmation dialog per Requirement 8.5
- Copy-to-clipboard for proxy keys per Requirement 8.6

### Component 7: AuthForms

**Purpose:** Login and registration forms with Zod validation.

**Structure:**
```typescript
// src/lib/validations/auth.ts

import { z } from 'zod';

export const loginSchema = z.object({
  email: z.string().email('Invalid email format'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
});

export const registerSchema = z.object({
  email: z.string().email('Invalid email format'),
  password: z.string()
    .min(8, 'Password must be at least 8 characters')
    .regex(/[A-Z]/, 'Password must contain uppercase letter')
    .regex(/[0-9]/, 'Password must contain a number'),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});
```

**Design Decisions:**
- Zod validation matches backend requirements per Requirement 9.6
- Inline error display per Requirement 9.3
- Redirect to /dashboard/overview on success per Requirement 9.4
- "Forgot Password" link placeholder for Week 3 per Requirement 9.5

## Data Models

### Backend Data Models

### Entity: UsageLog (proxy_requests table)

```rust
pub struct ProxyRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: Provider,
    pub model: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub latency_ms: i32,
    pub estimated_cost_idr: i64,
    pub status_code: i16,
    pub created_at: DateTime<Utc>,
}
```

### Provider Request/Response Formats

```rust
// Anthropic Messages API
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,  // Required for Anthropic
    pub system: Option<String>,
    pub messages: Vec<AnthropicMessage>,
}

// Google Generative AI API
pub struct GoogleRequest {
    pub contents: Vec<GoogleContent>,
    pub generation_config: GenerationConfig,
}

// Alibaba DashScope API
pub struct QwenRequest {
    pub model: String,
    pub input: QwenInput,
    pub parameters: QwenParameters,
}
```

### Frontend Data Models

```typescript
// User session state
interface UserSession {
  id: string;
  email: string;
  plan: 'free' | 'starter' | 'pro' | 'team';
  usageSummary: {
    requestsUsed: number;
    requestsLimit: number;
    currentPeriodEnd: Date;
  };
}

// API Key management
interface ApiKeyFormData {
  provider: 'openai' | 'anthropic' | 'google' | 'qwen';
  apiKey: string;
}

// Pricing tier display (Requirement 6.3)
interface PricingTierDisplay {
  id: string;
  name: string;
  priceIdr: number;           // Raw value: 0, 49000, 99000, 299000
  priceFormatted: string;     // Formatted: "Rp 0", "Rp 49.000", etc.
  requestsPerMonth: number;
  apiKeysLimit: number | null; // null = unlimited
  providersLimit: number | null;
  features: string[];
  isPopular?: boolean;
}

// Language/i18n state (Requirement 6.6)
type Locale = 'id' | 'en';
interface I18nState {
  locale: Locale;
  translations: Record<string, string>;
}
```

## Database Schema Addition

```sql
-- Usage logs table
CREATE TABLE proxy_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(20) NOT NULL,
    model VARCHAR(50) NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    latency_ms INTEGER NOT NULL,
    estimated_cost_idr BIGINT NOT NULL DEFAULT 0,
    status_code SMALLINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_proxy_requests_user_id ON proxy_requests(user_id);
CREATE INDEX idx_proxy_requests_created_at ON proxy_requests(created_at);
CREATE INDEX idx_proxy_requests_user_provider ON proxy_requests(user_id, provider);
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Request Format Transformation Consistency
*For any* valid ChatCompletionRequest, transforming to provider format and back SHALL preserve the semantic content (messages, model intent, parameters).
**Validates: Requirements 1.2, 2.2, 3.2**

### Property 2: Response Format Normalization
*For any* provider response, the transformed response SHALL conform to OpenAI's ChatCompletionResponse schema.
**Validates: Requirements 1.4, 2.4, 3.4**

### Property 3: Streaming Chunk Format
*For any* streaming response, each chunk SHALL be valid SSE format (`data: {...}\n\n`) and the final chunk SHALL be `data: [DONE]\n\n`.
**Validates: Requirements 4.2, 4.3**

### Property 4: Usage Log Completeness
*For any* completed proxy request, the usage log SHALL contain: user_id, provider, model, input_tokens, output_tokens, latency_ms, and timestamp.
**Validates: Requirements 5.1, 5.4**

### Property 5: Model Routing Correctness
*For any* request with a model name, the router SHALL forward to the correct provider (gpt-* → OpenAI, claude-* → Anthropic, gemini-* → Google, qwen-* → Qwen).
**Validates: Requirements 1.1, 2.1, 3.1**

## Error Handling

### Backend Error Handling

| Error Scenario | Response | HTTP Status |
|----------------|----------|-------------|
| Unknown model | `{"error": "Unknown model: {model}"}` | 400 |
| Provider key not configured | `{"error": "{Provider} API key not configured"}` | 400 |
| Provider API error | Forward provider's error | 4xx/5xx |
| Stream connection dropped | Close client connection | - |
| Token counting failed | Use estimate (chars/4) | - |

### Frontend Error Handling

| Error Scenario | User Feedback | Action |
|----------------|---------------|--------|
| Not authenticated | Redirect to /login | Preserve return URL per Req 7.4 |
| API key validation failed | Inline error message | Highlight invalid field per Req 9.3 |
| API key test connection failed | Toast notification | Show provider error message per Req 8.4 |
| Form validation error | Inline field errors | Display Zod validation messages per Req 9.3 |
| Network error | Toast notification | Retry option with exponential backoff |
| Page load >2s | Loading skeleton | Show shimmer UI per Req 6.1 |

## Testing Strategy

### Unit Tests
- Test each transformer's request/response conversion
- Test model routing logic
- Test SSE chunk formatting
- Test token counting/estimation

### Property-Based Tests
- Use `proptest` crate for Rust
- Minimum 100 iterations per property
- Tag format: `**Feature: week2-multi-provider, Property {number}: {property_text}**`

**Property tests to implement:**
1. Request format transformation (Property 1)
2. Response format normalization (Property 2)
3. Streaming chunk format (Property 3)
4. Model routing correctness (Property 5)

### Integration Tests
- Test full proxy flow for each provider (mock provider APIs)
- Test streaming with mock SSE responses
- Test usage logging after requests
