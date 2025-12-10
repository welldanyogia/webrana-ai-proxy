//! User model for database operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Plan tier enum matching PostgreSQL enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "plan_tier", rename_all = "lowercase")]
pub enum PlanTier {
    Free,
    Starter,
    Pro,
    Team,
}

impl Default for PlanTier {
    fn default() -> Self {
        PlanTier::Free
    }
}

impl PlanTier {
    /// Get API key limit for this plan
    pub fn api_key_limit(&self) -> Option<u32> {
        match self {
            PlanTier::Free => Some(1),
            PlanTier::Starter => Some(5),
            PlanTier::Pro | PlanTier::Team => None, // Unlimited
        }
    }

    /// Get monthly request limit for this plan
    pub fn request_limit(&self) -> u32 {
        match self {
            PlanTier::Free => 1_000,
            PlanTier::Starter => 10_000,
            PlanTier::Pro => 50_000,
            PlanTier::Team => 200_000,
        }
    }

    /// Get provider limit for this plan
    pub fn provider_limit(&self) -> Option<u32> {
        match self {
            PlanTier::Free => Some(1),
            PlanTier::Starter => Some(2),
            PlanTier::Pro | PlanTier::Team => None, // All providers
        }
    }
}

/// User entity
#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub plan_tier: PlanTier,
    pub is_active: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User creation DTO
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

/// User login DTO
#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

/// User response (safe to return to client)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub plan_tier: PlanTier,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            plan_tier: user.plan_tier,
            is_active: user.is_active,
            email_verified: user.email_verified_at.is_some(),
            created_at: user.created_at,
        }
    }
}
