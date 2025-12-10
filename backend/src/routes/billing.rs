use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::billing_service::{
    BillingError, BillingService, MidtransSnapToken, MidtransWebhook, PlanTier, Subscription,
};
use crate::services::invoice_service::{Invoice, InvoiceService};

/// App state for billing routes
#[derive(Clone)]
pub struct BillingState {
    pub pool: PgPool,
    pub billing_service: std::sync::Arc<BillingService>,
}

/// Create subscription request
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub plan: String,
}

/// Create subscription response
#[derive(Debug, Serialize)]
pub struct CreateSubscriptionResponse {
    pub token: String,
    pub redirect_url: String,
    pub order_id: String,
}

/// Billing routes
pub fn billing_routes(billing_service: std::sync::Arc<BillingService>) -> Router<PgPool> {
    Router::new()
        .route("/subscribe", post(create_subscription))
        .route("/subscription", get(get_subscription))
        .route("/subscription/cancel", post(cancel_subscription))
        .route("/invoices", get(get_invoices))
        .route("/invoices/{id}", get(get_invoice_html))
        .route("/invoices/{id}/download", get(download_invoice))
        .route("/webhook/midtrans", post(handle_midtrans_webhook))
        .with_state(billing_service)
}

/// Create subscription and get Midtrans Snap token
/// POST /billing/subscribe
/// Requirements: 2.1, 2.3
async fn create_subscription(
    State(billing_service): State<std::sync::Arc<BillingService>>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<CreateSubscriptionResponse>, (StatusCode, String)> {
    // TODO: Get user_id and email from auth middleware
    let user_id = Uuid::nil();
    let user_email = "user@example.com";

    let plan = match req.plan.to_lowercase().as_str() {
        "starter" => PlanTier::Starter,
        "pro" => PlanTier::Pro,
        "team" => PlanTier::Team,
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid plan tier".to_string())),
    };

    let snap_token = billing_service
        .create_subscription(user_id, plan, user_email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CreateSubscriptionResponse {
        token: snap_token.token,
        redirect_url: snap_token.redirect_url,
        order_id: snap_token.order_id,
    }))
}


/// Get current subscription
/// GET /billing/subscription
async fn get_subscription(
    State(billing_service): State<std::sync::Arc<BillingService>>,
) -> Result<Json<Option<Subscription>>, (StatusCode, String)> {
    // TODO: Get user_id from auth middleware
    let user_id = Uuid::nil();

    let subscription = billing_service
        .get_subscription(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(subscription))
}

/// Cancel subscription
/// POST /billing/subscription/cancel
/// Requirements: 3.5
async fn cancel_subscription(
    State(billing_service): State<std::sync::Arc<BillingService>>,
) -> Result<StatusCode, (StatusCode, String)> {
    // TODO: Get user_id from auth middleware
    let user_id = Uuid::nil();

    billing_service
        .cancel_subscription(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

/// Handle Midtrans webhook notification
/// POST /billing/webhook/midtrans
/// Requirements: 2.4, 2.5, 2.6
async fn handle_midtrans_webhook(
    State(billing_service): State<std::sync::Arc<BillingService>>,
    Json(webhook): Json<MidtransWebhook>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!(
        order_id = %webhook.order_id,
        status = %webhook.transaction_status,
        "Received Midtrans webhook"
    );

    match billing_service.handle_webhook(webhook).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(BillingError::InvalidSignature) => {
            tracing::warn!("Invalid webhook signature");
            Err((StatusCode::BAD_REQUEST, "Invalid signature".to_string()))
        }
        Err(e) => {
            tracing::error!(error = %e, "Webhook processing failed");
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// Get user's invoices
/// GET /billing/invoices
/// Requirements: 4.6
async fn get_invoices(
    State(billing_service): State<std::sync::Arc<BillingService>>,
) -> Result<Json<Vec<Invoice>>, (StatusCode, String)> {
    // TODO: Get user_id from auth middleware
    let user_id = Uuid::nil();
    
    let pool = billing_service.pool();
    let invoice_service = InvoiceService::new(pool.clone());
    
    let invoices = invoice_service
        .get_user_invoices(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(invoices))
}

/// Get invoice as HTML (for viewing/printing)
/// GET /billing/invoices/:id
/// Requirements: 4.1, 4.6
async fn get_invoice_html(
    State(billing_service): State<std::sync::Arc<BillingService>>,
    Path(invoice_id): Path<Uuid>,
) -> Response {
    let pool = billing_service.pool();
    let invoice_service = InvoiceService::new(pool.clone());
    
    match invoice_service.get_invoice(invoice_id).await {
        Ok(invoice) => {
            let html = InvoiceService::generate_html_invoice(&invoice);
            Html(html).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Invoice not found").into_response(),
    }
}

/// Download invoice as HTML (with Content-Disposition header)
/// GET /billing/invoices/:id/download
/// Requirements: 4.1, 4.6
async fn download_invoice(
    State(billing_service): State<std::sync::Arc<BillingService>>,
    Path(invoice_id): Path<Uuid>,
) -> Response {
    let pool = billing_service.pool();
    let invoice_service = InvoiceService::new(pool.clone());
    
    match invoice_service.get_invoice(invoice_id).await {
        Ok(invoice) => {
            let html = InvoiceService::generate_html_invoice(&invoice);
            let filename = format!("invoice-{}.html", invoice.invoice.invoice_number);
            
            let headers = [
                (axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    &format!("attachment; filename=\"{}\"", filename),
                ),
            ];
            
            (headers, html).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Invoice not found").into_response(),
    }
}
