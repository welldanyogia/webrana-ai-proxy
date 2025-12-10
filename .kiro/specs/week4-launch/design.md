# Design Document - Week 4: Launch

## Overview

Week 4 focuses on public beta launch preparation, ProductHunt submission, documentation, and user onboarding. This sprint is primarily operational with minimal new code development. The focus is on monitoring, support readiness, and rapid response to user feedback.

## Architecture

```mermaid
graph TD
    subgraph "Launch Infrastructure"
        A[Production K8s] --> B[3-5 Nodes Auto-scale]
        C[Cloudflare] --> D[DDoS Protection]
        E[Grafana] --> F[Real-time Dashboards]
    end
    
    subgraph "Monitoring Stack"
        G[Prometheus] --> H[Metrics Collection]
        I[Alertmanager] --> J[Slack Notifications]
        K[Sentry] --> L[Error Tracking]
    end
    
    subgraph "User Onboarding"
        M[Signup] --> N[Welcome Email]
        N --> O[Onboarding Checklist]
        O --> P[First API Key]
        P --> Q[First Request]
        Q --> R[Activation Complete]
    end
    
    subgraph "Support Channels"
        S[Email] --> T[support@webrana.id]
        U[Discord] --> V[Community Server]
        W[GitHub] --> X[Issues]
    end
```

## Components and Interfaces

### Component 1: OnboardingService

**Purpose:** Tracks user onboarding progress and triggers engagement emails.

**Interface:**
```rust
pub trait OnboardingService {
    async fn get_onboarding_status(&self, user_id: Uuid) -> Result<OnboardingStatus, OnboardingError>;
    async fn mark_step_complete(&self, user_id: Uuid, step: OnboardingStep) -> Result<(), OnboardingError>;
    async fn check_inactive_users(&self) -> Result<Vec<InactiveUser>, OnboardingError>;
}

pub struct OnboardingStatus {
    pub user_id: Uuid,
    pub steps_completed: Vec<OnboardingStep>,
    pub completion_percent: u8,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

pub enum OnboardingStep {
    AccountCreated,
    ApiKeyAdded,
    FirstRequestMade,
    UsageDashboardViewed,
}
```

### Component 2: AnalyticsService

**Purpose:** Tracks user acquisition, activation, and retention metrics.

**Interface:**
```rust
pub trait AnalyticsService {
    async fn track_event(&self, event: AnalyticsEvent) -> Result<(), AnalyticsError>;
    async fn get_acquisition_stats(&self, period: DateRange) -> Result<AcquisitionStats, AnalyticsError>;
    async fn get_activation_funnel(&self) -> Result<ActivationFunnel, AnalyticsError>;
    async fn get_retention_cohorts(&self) -> Result<Vec<RetentionCohort>, AnalyticsError>;
    async fn identify_churn_risk(&self) -> Result<Vec<ChurnRiskUser>, AnalyticsError>;
}

pub struct AnalyticsEvent {
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub properties: HashMap<String, Value>,
    pub source: String,  // producthunt, organic, referral
    pub timestamp: DateTime<Utc>,
}
```

### Component 3: StatusPageService

**Purpose:** Manages system status and incident communication.

**Interface:**
```rust
pub trait StatusPageService {
    async fn get_current_status(&self) -> Result<SystemStatus, StatusError>;
    async fn create_incident(&self, incident: Incident) -> Result<IncidentId, StatusError>;
    async fn update_incident(&self, id: IncidentId, update: IncidentUpdate) -> Result<(), StatusError>;
    async fn resolve_incident(&self, id: IncidentId) -> Result<(), StatusError>;
}

pub struct SystemStatus {
    pub overall: StatusLevel,  // Operational, Degraded, Outage
    pub components: Vec<ComponentStatus>,
    pub active_incidents: Vec<Incident>,
}
```

## Data Models

### Entity: OnboardingProgress

```rust
pub struct OnboardingProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_created_at: DateTime<Utc>,
    pub api_key_added_at: Option<DateTime<Utc>>,
    pub first_request_at: Option<DateTime<Utc>>,
    pub dashboard_viewed_at: Option<DateTime<Utc>>,
    pub reminder_sent_at: Option<DateTime<Utc>>,
}
```

### Entity: AnalyticsEvent

```rust
pub struct AnalyticsEventRecord {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub properties: JsonValue,
    pub source: String,
    pub created_at: DateTime<Utc>,
}
```

## Database Schema Addition

```sql
-- Onboarding progress tracking
CREATE TABLE onboarding_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    api_key_added_at TIMESTAMP WITH TIME ZONE,
    first_request_at TIMESTAMP WITH TIME ZONE,
    dashboard_viewed_at TIMESTAMP WITH TIME ZONE,
    reminder_sent_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_onboarding_user_id ON onboarding_progress(user_id);

-- Analytics events
CREATE TABLE analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}',
    source VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_analytics_events_type ON analytics_events(event_type);
CREATE INDEX idx_analytics_events_created ON analytics_events(created_at);
CREATE INDEX idx_analytics_events_user ON analytics_events(user_id);
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Onboarding Step Progression
*For any* user, onboarding steps SHALL be recorded in chronological order (account_created → api_key_added → first_request), and completion percentage SHALL accurately reflect completed steps.
**Validates: Requirements 5.6**

### Property 2: Inactive User Detection
*For any* user who signed up more than 24 hours ago without adding an API key, the user SHALL be identified by the inactive user check.
**Validates: Requirements 5.5**

### Property 3: Analytics Event Integrity
*For any* tracked event, the event record SHALL contain: event_type, timestamp, and source, with optional user_id for authenticated events.
**Validates: Requirements 9.1**

Note: Week 4 is primarily operational/marketing focused. Most requirements relate to processes, content, and monitoring rather than testable code properties.

## Error Handling

| Error Scenario | Response | HTTP Status |
|----------------|----------|-------------|
| Status page unavailable | Return cached status | 200 |
| Analytics tracking failed | Log error, don't block user | - |
| Onboarding query failed | Return empty status | 200 |

## Testing Strategy

### Unit Tests
- Test onboarding step progression logic
- Test inactive user detection query
- Test analytics event validation

### Property-Based Tests
- Use `proptest` crate for Rust
- Minimum 100 iterations per property
- Tag format: `**Feature: week4-launch, Property {number}: {property_text}**`

**Property tests to implement:**
1. Onboarding step progression (Property 1)
2. Inactive user detection (Property 2)

### Integration Tests
- Test full onboarding flow
- Test analytics event tracking
- Test email trigger for inactive users

### Smoke Tests (Launch Day)
- Health check all endpoints
- Verify database connectivity
- Verify Redis connectivity
- Verify external API connectivity (Midtrans, SendGrid)
- Verify SSL certificates valid
