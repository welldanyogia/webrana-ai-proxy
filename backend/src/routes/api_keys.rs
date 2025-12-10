//! API key management routes for provider and proxy keys.
//!
//! Requirements: 3.1, 3.2, 3.4, 3.6, 6.1-6.5

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::auth::AuthUser;
use crate::models::api_key::{AiProvider, CreateApiKey};
use crate::models::proxy_api_key::CreateProxyApiKey;
use crate::models::user::PlanTier;
use crate::services::api_key_service::{ApiKeyError, ApiKeyServiceImpl};
use crate::services::proxy_key_service::{ProxyKeyError, ProxyKeyService};
use crate::AppState;

pub fn router() -> Router {
    Router::new()
        // Provider API keys
        .route("/provider", post(store_provider_key))
        .route("/provider", get(list_provider_keys))
        .route("/provider/{id}", delete(delete_provider_key))
        // Proxy API keys (TODO: Task 11)
        .route("/proxy", post(generate_proxy_key))
        .route("/proxy", get(list_proxy_keys))
        .route("/proxy/{id}", delete(revoke_proxy_key))
}

/// Request body for storing a provider API key
#[derive(Debug, Deserialize)]
pub struct StoreProviderKeyRequest {
    pub provider: AiProvider,
    pub key: String,
    pub name: String,
}

/// Response for stored provider API key
#[derive(Debug, Serialize)]
pub struct StoreProviderKeyResponse {
    pub id: Uuid,
    pub provider: AiProvider,
    pub name: String,
    pub masked_key: String,
    pub created_at: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ApiKeyErrorResponse {
    pub error: String,
    pub code: String,
}

/// POST /api-keys/provider - Store a provider API key
/// Requirements: 3.1, 3.2, 3.6
async fn store_provider_key(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<StoreProviderKeyRequest>,
) -> impl IntoResponse {
    // Initialize service
    let service = match ApiKeyServiceImpl::from_env() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize encryption: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Server configuration error".to_string(),
                    code: "ENCRYPTION_CONFIG_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Create API key input
    let input = CreateApiKey {
        provider: body.provider,
        key: body.key,
        name: body.name,
    };

    // Store the key
    match service.store_provider_key(&state.db, auth_user.user_id, input).await {
        Ok(stored) => (
            StatusCode::CREATED,
            Json(StoreProviderKeyResponse {
                id: stored.id,
                provider: stored.provider,
                name: stored.name,
                masked_key: stored.masked_key,
                created_at: stored.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(ApiKeyError::InvalidKeyFormat(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(ApiKeyErrorResponse {
                error: msg,
                code: "INVALID_KEY_FORMAT".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to store provider key: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Failed to store API key".to_string(),
                    code: "STORAGE_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}


/// GET /api-keys/provider - List provider API keys (masked)
/// Requirement: 3.4
async fn list_provider_keys(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    // Initialize service
    let service = match ApiKeyServiceImpl::from_env() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize encryption: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Server configuration error".to_string(),
                    code: "ENCRYPTION_CONFIG_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    };

    match service.list_provider_keys(&state.db, auth_user.user_id).await {
        Ok(keys) => (StatusCode::OK, Json(keys)).into_response(),
        Err(e) => {
            tracing::error!("Failed to list provider keys: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Failed to list API keys".to_string(),
                    code: "LIST_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// DELETE /api-keys/provider/:id - Delete a provider API key
/// Requirement: 3.1
async fn delete_provider_key(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Initialize service
    let service = match ApiKeyServiceImpl::from_env() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize encryption: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Server configuration error".to_string(),
                    code: "ENCRYPTION_CONFIG_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    };

    match service.delete_provider_key(&state.db, auth_user.user_id, id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(ApiKeyError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(ApiKeyErrorResponse {
                error: "API key not found".to_string(),
                code: "NOT_FOUND".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete provider key: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Failed to delete API key".to_string(),
                    code: "DELETE_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

// ============================================================
// Proxy API Keys (Task 11)
// ============================================================

/// Request body for generating a proxy API key
#[derive(Debug, Deserialize)]
pub struct GenerateProxyKeyRequest {
    pub name: String,
}

/// POST /api-keys/proxy - Generate a new proxy API key
/// Requirements: 6.1, 6.2, 6.3, 6.5
async fn generate_proxy_key(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<GenerateProxyKeyRequest>,
) -> impl IntoResponse {
    // Parse plan tier from auth user
    let plan: PlanTier = match auth_user.plan.as_str() {
        "free" => PlanTier::Free,
        "starter" => PlanTier::Starter,
        "pro" => PlanTier::Pro,
        "team" => PlanTier::Team,
        _ => PlanTier::Free,
    };

    let input = CreateProxyApiKey { name: body.name };

    match ProxyKeyService::generate_key(&state.db, auth_user.user_id, plan, input).await {
        Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
        Err(ProxyKeyError::KeyLimitReached { limit, .. }) => (
            StatusCode::FORBIDDEN,
            Json(ApiKeyErrorResponse {
                error: format!("API key limit reached for your plan (max: {})", limit),
                code: "KEY_LIMIT_REACHED".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to generate proxy key: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Failed to generate API key".to_string(),
                    code: "GENERATION_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// GET /api-keys/proxy - List proxy API keys (prefix and metadata only)
/// Requirement: 6.4
async fn list_proxy_keys(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match ProxyKeyService::list_keys(&state.db, auth_user.user_id).await {
        Ok(keys) => (StatusCode::OK, Json(keys)).into_response(),
        Err(e) => {
            tracing::error!("Failed to list proxy keys: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Failed to list API keys".to_string(),
                    code: "LIST_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// DELETE /api-keys/proxy/:id - Revoke a proxy API key
/// Requirement: 6.1
async fn revoke_proxy_key(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ProxyKeyService::revoke_key(&state.db, auth_user.user_id, id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(ProxyKeyError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(ApiKeyErrorResponse {
                error: "API key not found".to_string(),
                code: "NOT_FOUND".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to revoke proxy key: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiKeyErrorResponse {
                    error: "Failed to revoke API key".to_string(),
                    code: "REVOKE_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}
