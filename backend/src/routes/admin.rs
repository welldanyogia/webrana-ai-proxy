use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Admin stats response
#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub active_subscriptions: i64,
    pub mrr_idr: i64,
    pub requests_today: i64,
    pub requests_this_month: i64,
}

/// User list item
#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub plan_tier: String,
    pub is_suspended: bool,
    pub requests_this_month: i64,
    pub created_at: String,
}

/// User list response
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserListItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Query params for user list
#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

/// User detail response
#[derive(Debug, Serialize)]
pub struct UserDetailResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub plan_tier: String,
    pub is_suspended: bool,
    pub requests_this_month: i64,
    pub total_requests: i64,
    pub total_cost_idr: i64,
    pub created_at: String,
}

/// Suspend user request
#[derive(Debug, Deserialize)]
pub struct SuspendUserRequest {
    pub reason: Option<String>,
}

/// Change plan request
#[derive(Debug, Deserialize)]
pub struct ChangePlanRequest {
    pub plan_tier: String,
}

/// Admin action response
#[derive(Debug, Serialize)]
pub struct AdminActionResponse {
    pub success: bool,
    pub message: String,
}

/// System health response
#[derive(Debug, Serialize)]
pub struct SystemHealthResponse {
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub error_rate_percent: f64,
    pub requests_last_hour: i64,
    pub errors_last_hour: i64,
    pub database_status: String,
}

/// Admin routes
/// Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6
pub fn admin_routes() -> Router<PgPool> {
    Router::new()
        .route("/stats", get(get_admin_stats))
        .route("/users", get(get_users))
        .route("/users/{id}", get(get_user_detail))
        .route("/users/{id}/suspend", post(suspend_user))
        .route("/users/{id}/unsuspend", post(unsuspend_user))
        .route("/users/{id}/plan", post(change_user_plan))
        .route("/health", get(get_system_health))
}


/// Get admin dashboard stats
/// GET /admin/stats
/// Requirements: 6.1
async fn get_admin_stats(State(pool): State<PgPool>) -> Result<Json<AdminStats>, StatusCode> {
    // Total users
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Active subscriptions
    let active_subscriptions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM subscriptions WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // MRR (Monthly Recurring Revenue)
    let mrr_idr: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(price_idr), 0) FROM subscriptions WHERE status = 'active'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Requests today
    let requests_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM proxy_requests WHERE created_at >= CURRENT_DATE",
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Requests this month
    let requests_this_month: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM proxy_requests WHERE created_at >= DATE_TRUNC('month', CURRENT_DATE)",
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminStats {
        total_users,
        active_subscriptions,
        mrr_idr,
        requests_today,
        requests_this_month,
    }))
}

/// Get user list with pagination and search
/// GET /admin/users
/// Requirements: 6.2, 6.3
async fn get_users(
    State(pool): State<PgPool>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<UserListResponse>, StatusCode> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let search_pattern = query
        .search
        .map(|s| format!("%{}%", s))
        .unwrap_or_else(|| "%".to_string());

    // Get total count
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email ILIKE $1 OR name ILIKE $1",
    )
    .bind(&search_pattern)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get users with request count
    let rows = sqlx::query(
        r#"
        SELECT 
            u.id, u.email, u.name, u.plan_tier::text as plan_tier, u.created_at,
            COALESCE(
                (SELECT COUNT(*) FROM proxy_requests pr 
                 WHERE pr.user_id = u.id 
                 AND pr.created_at >= DATE_TRUNC('month', CURRENT_DATE)),
                0
            )::bigint as requests_this_month
        FROM users u
        WHERE u.email ILIKE $1 OR u.name ILIKE $1
        ORDER BY u.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&search_pattern)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<UserListItem> = rows
        .into_iter()
        .map(|r| UserListItem {
            id: r.get("id"),
            email: r.get("email"),
            name: r.get("name"),
            plan_tier: r.get("plan_tier"),
            is_suspended: r.try_get("is_suspended").unwrap_or(false),
            requests_this_month: r.get("requests_this_month"),
            created_at: r
                .get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .to_rfc3339(),
        })
        .collect();

    Ok(Json(UserListResponse {
        users,
        total,
        page,
        per_page,
    }))
}

/// Get user detail with usage stats
/// GET /admin/users/:id
/// Requirements: 6.4
async fn get_user_detail(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDetailResponse>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT 
            u.id, u.email, u.name, u.plan_tier::text as plan_tier, 
            COALESCE(u.is_suspended, false) as is_suspended, u.created_at,
            COALESCE(
                (SELECT COUNT(*) FROM proxy_requests pr 
                 WHERE pr.user_id = u.id 
                 AND pr.created_at >= DATE_TRUNC('month', CURRENT_DATE)),
                0
            )::bigint as requests_this_month,
            COALESCE(
                (SELECT COUNT(*) FROM proxy_requests pr WHERE pr.user_id = u.id),
                0
            )::bigint as total_requests,
            COALESCE(
                (SELECT SUM(estimated_cost_idr) FROM proxy_requests pr WHERE pr.user_id = u.id),
                0
            )::bigint as total_cost_idr
        FROM users u
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserDetailResponse {
        id: row.get("id"),
        email: row.get("email"),
        name: row.get("name"),
        plan_tier: row.get("plan_tier"),
        is_suspended: row.get("is_suspended"),
        requests_this_month: row.get("requests_this_month"),
        total_requests: row.get("total_requests"),
        total_cost_idr: row.get("total_cost_idr"),
        created_at: row
            .get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            .to_rfc3339(),
    }))
}

/// Suspend a user
/// POST /admin/users/:id/suspend
/// Requirements: 6.4
async fn suspend_user(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SuspendUserRequest>,
) -> Result<Json<AdminActionResponse>, StatusCode> {
    let result = sqlx::query(
        "UPDATE users SET is_suspended = true, suspended_reason = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(req.reason)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    tracing::info!(user_id = %user_id, "User suspended by admin");

    Ok(Json(AdminActionResponse {
        success: true,
        message: "User suspended successfully".to_string(),
    }))
}

/// Unsuspend a user
/// POST /admin/users/:id/unsuspend
/// Requirements: 6.4
async fn unsuspend_user(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<AdminActionResponse>, StatusCode> {
    let result = sqlx::query(
        "UPDATE users SET is_suspended = false, suspended_reason = NULL, updated_at = NOW() WHERE id = $1",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    tracing::info!(user_id = %user_id, "User unsuspended by admin");

    Ok(Json(AdminActionResponse {
        success: true,
        message: "User unsuspended successfully".to_string(),
    }))
}

/// Change user's plan tier
/// POST /admin/users/:id/plan
/// Requirements: 6.4
async fn change_user_plan(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ChangePlanRequest>,
) -> Result<Json<AdminActionResponse>, StatusCode> {
    // Validate plan tier
    let valid_plans = ["free", "starter", "pro", "team"];
    if !valid_plans.contains(&req.plan_tier.to_lowercase().as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let result = sqlx::query(
        "UPDATE users SET plan_tier = $1::plan_tier, updated_at = NOW() WHERE id = $2",
    )
    .bind(&req.plan_tier.to_lowercase())
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    tracing::info!(
        user_id = %user_id,
        new_plan = %req.plan_tier,
        "User plan changed by admin"
    );

    Ok(Json(AdminActionResponse {
        success: true,
        message: format!("User plan changed to {}", req.plan_tier),
    }))
}


/// Get system health metrics
/// GET /admin/health
/// Requirements: 6.6
async fn get_system_health(
    State(pool): State<PgPool>,
) -> Result<Json<SystemHealthResponse>, StatusCode> {
    // Get latency percentiles from last hour
    let latency_row = sqlx::query(
        r#"
        SELECT 
            COALESCE(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY latency_ms), 0)::float8 as p50,
            COALESCE(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms), 0)::float8 as p95,
            COALESCE(PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms), 0)::float8 as p99
        FROM proxy_requests
        WHERE created_at >= NOW() - INTERVAL '1 hour'
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let latency_p50: f64 = latency_row.get("p50");
    let latency_p95: f64 = latency_row.get("p95");
    let latency_p99: f64 = latency_row.get("p99");

    // Get request and error counts from last hour
    let counts_row = sqlx::query(
        r#"
        SELECT 
            COUNT(*)::bigint as total_requests,
            COUNT(*) FILTER (WHERE status_code >= 500)::bigint as errors
        FROM proxy_requests
        WHERE created_at >= NOW() - INTERVAL '1 hour'
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let requests_last_hour: i64 = counts_row.get("total_requests");
    let errors_last_hour: i64 = counts_row.get("errors");

    let error_rate = if requests_last_hour > 0 {
        (errors_last_hour as f64 / requests_last_hour as f64) * 100.0
    } else {
        0.0
    };

    // Check database connectivity
    let db_status = match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    Ok(Json(SystemHealthResponse {
        latency_p50_ms: latency_p50,
        latency_p95_ms: latency_p95,
        latency_p99_ms: latency_p99,
        error_rate_percent: error_rate,
        requests_last_hour,
        errors_last_hour,
        database_status: db_status,
    }))
}
