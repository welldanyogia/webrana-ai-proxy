//! Proxy routes for forwarding requests to AI providers.
//!
//! Requirements: 1.1-1.5, 2.1-2.5, 3.1-3.5, 4.1-4.5, 5.1-5.6 - Multi-provider proxy endpoints

use axum::{
    body::Body,
    extract::Extension,
    http::{header, StatusCode},
    response::{IntoResponse, Response, Sse},
    routing::post,
    Json, Router,
};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::convert::Infallible;
use async_stream::stream;
use axum::response::sse::Event;

use crate::middleware::auth::ApiKeyUser;
use crate::models::api_key::AiProvider;
use crate::services::api_key_service::ApiKeyServiceImpl;
use crate::services::stream_handler::{
    StreamHandler, StreamChunk, AnthropicStreamEvent, GoogleStreamChunk, QwenStreamChunk,
};
use crate::services::transformers::{
    anthropic::AnthropicTransformer,
    google::GoogleTransformer,
    qwen::QwenTransformer,
    Provider,
};
use crate::AppState;

pub fn router() -> Router {
    Router::new().route("/chat/completions", post(chat_completions))
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ProxyErrorResponse {
    pub error: ProxyError,
}

#[derive(Debug, Serialize)]
pub struct ProxyError {
    pub message: String,
    pub r#type: String,
    pub code: String,
}

/// Chat completion request (OpenAI-compatible format)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Convert route Message to transformer Message
impl From<Message> for crate::services::transformers::Message {
    fn from(msg: Message) -> Self {
        crate::services::transformers::Message {
            role: msg.role,
            content: msg.content,
        }
    }
}

/// Convert route ChatCompletionRequest to transformer ChatCompletionRequest
impl From<ChatCompletionRequest> for crate::services::transformers::ChatCompletionRequest {
    fn from(req: ChatCompletionRequest) -> Self {
        crate::services::transformers::ChatCompletionRequest {
            model: req.model,
            messages: req.messages.into_iter().map(|m| m.into()).collect(),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: req.stream,
            top_p: req.top_p,
            frequency_penalty: req.frequency_penalty,
            presence_penalty: req.presence_penalty,
            stop: req.stop,
            user: req.user,
        }
    }
}

/// POST /v1/chat/completions - Proxy to AI providers
/// Requirements: 1.1, 2.1, 3.1, 5.1 - Multi-provider routing
async fn chat_completions(
    Extension(state): Extension<Arc<AppState>>,
    Extension(api_key_user): Extension<ApiKeyUser>,
    Json(body): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    // Determine provider from model name
    let provider = match Provider::from_model(&body.model) {
        Some(p) => p,
        None => {
            return proxy_error(
                StatusCode::BAD_REQUEST,
                &format!("Unknown model: {}. Supported prefixes: gpt-*, claude-*, gemini-*, qwen-*", body.model),
                "invalid_model",
                "UNKNOWN_MODEL",
            );
        }
    };

    // Initialize API key service
    let service = match ApiKeyServiceImpl::from_env() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize encryption: {}", e);
            return proxy_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error",
                "server_error",
                "CONFIG_ERROR",
            );
        }
    };

    // Route to appropriate provider
    match provider {
        Provider::OpenAI => forward_to_openai(&state, &service, api_key_user.user_id, body).await,
        Provider::Anthropic => forward_to_anthropic(&state, &service, api_key_user.user_id, body).await,
        Provider::Google => forward_to_google(&state, &service, api_key_user.user_id, body).await,
        Provider::Qwen => forward_to_qwen(&state, &service, api_key_user.user_id, body).await,
    }
}

/// Forward request to OpenAI
/// Requirements: 4.1-4.5, 5.1-5.6
async fn forward_to_openai(
    state: &Arc<AppState>,
    service: &ApiKeyServiceImpl,
    user_id: uuid::Uuid,
    body: ChatCompletionRequest,
) -> Response {
    // Get user's OpenAI API key
    let api_key = match service
        .get_decrypted_key(&state.db, user_id, AiProvider::Openai)
        .await
    {
        Ok(key) => key,
        Err(_) => {
            return proxy_error(
                StatusCode::BAD_REQUEST,
                "OpenAI API key not configured",
                "api_key_missing",
                "OPENAI_KEY_NOT_CONFIGURED",
            );
        }
    };

    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";
    let is_streaming = body.stream;

    let response = match client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to forward request to OpenAI: {}", e);
            return proxy_error(
                StatusCode::BAD_GATEWAY,
                "Failed to connect to OpenAI",
                "upstream_error",
                "OPENAI_CONNECTION_ERROR",
            );
        }
    };

    // For streaming, passthrough OpenAI's SSE directly
    if is_streaming && response.status().is_success() {
        return forward_stream_response(response).await;
    }

    forward_response(response).await
}

/// Forward request to Anthropic
/// Requirements: 1.1-1.5, 4.1-4.5
async fn forward_to_anthropic(
    state: &Arc<AppState>,
    service: &ApiKeyServiceImpl,
    user_id: uuid::Uuid,
    body: ChatCompletionRequest,
) -> Response {
    // Get user's Anthropic API key
    let api_key = match service
        .get_decrypted_key(&state.db, user_id, AiProvider::Anthropic)
        .await
    {
        Ok(key) => key,
        Err(_) => {
            return proxy_error(
                StatusCode::BAD_REQUEST,
                "Anthropic API key not configured",
                "api_key_missing",
                "ANTHROPIC_KEY_NOT_CONFIGURED",
            );
        }
    };

    // Transform request to Anthropic format
    let transformer_request: crate::services::transformers::ChatCompletionRequest = body.clone().into();
    let anthropic_request = AnthropicTransformer::transform_request(&transformer_request);
    let is_streaming = body.stream;
    let model = body.model.clone();

    let client = Client::new();
    let url = "https://api.anthropic.com/v1/messages";

    let response = match client
        .post(url)
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&anthropic_request)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to forward request to Anthropic: {}", e);
            return proxy_error(
                StatusCode::BAD_GATEWAY,
                "Failed to connect to Anthropic",
                "upstream_error",
                "ANTHROPIC_CONNECTION_ERROR",
            );
        }
    };

    // Handle streaming response
    let status = response.status();
    if is_streaming && status.is_success() {
        return forward_anthropic_stream(response, model).await;
    }

    // Transform response back to OpenAI format
    if status.is_success() {
        match response.json::<crate::services::transformers::anthropic::AnthropicResponse>().await {
            Ok(anthropic_resp) => {
                let openai_resp = AnthropicTransformer::transform_response(anthropic_resp);
                (StatusCode::OK, Json(openai_resp)).into_response()
            }
            Err(e) => {
                tracing::error!("Failed to parse Anthropic response: {}", e);
                proxy_error(
                    StatusCode::BAD_GATEWAY,
                    "Failed to parse Anthropic response",
                    "upstream_error",
                    "ANTHROPIC_PARSE_ERROR",
                )
            }
        }
    } else {
        // Forward error response as-is
        forward_response_with_status(response, status).await
    }
}

/// Forward request to Google AI
/// Requirements: 2.1-2.5, 4.1-4.5
async fn forward_to_google(
    state: &Arc<AppState>,
    service: &ApiKeyServiceImpl,
    user_id: uuid::Uuid,
    body: ChatCompletionRequest,
) -> Response {
    // Get user's Google AI API key
    let api_key = match service
        .get_decrypted_key(&state.db, user_id, AiProvider::Google)
        .await
    {
        Ok(key) => key,
        Err(_) => {
            return proxy_error(
                StatusCode::BAD_REQUEST,
                "Google AI API key not configured",
                "api_key_missing",
                "GOOGLE_KEY_NOT_CONFIGURED",
            );
        }
    };

    // Transform request to Google format
    let transformer_request: crate::services::transformers::ChatCompletionRequest = body.clone().into();
    let google_request = GoogleTransformer::transform_request(&transformer_request);
    let is_streaming = body.stream;
    let model = body.model.clone();

    let client = Client::new();
    // Use streaming endpoint if streaming is requested
    let url = if is_streaming {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}&alt=sse",
            model, api_key
        )
    } else {
        GoogleTransformer::api_url(&model, &api_key)
    };

    let response = match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&google_request)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to forward request to Google AI: {}", e);
            return proxy_error(
                StatusCode::BAD_GATEWAY,
                "Failed to connect to Google AI",
                "upstream_error",
                "GOOGLE_CONNECTION_ERROR",
            );
        }
    };

    // Handle streaming response
    let status = response.status();
    if is_streaming && status.is_success() {
        return forward_google_stream(response, model).await;
    }

    // Transform response back to OpenAI format
    if status.is_success() {
        match response.json::<crate::services::transformers::google::GoogleResponse>().await {
            Ok(google_resp) => {
                let openai_resp = GoogleTransformer::transform_response(google_resp, &body.model);
                (StatusCode::OK, Json(openai_resp)).into_response()
            }
            Err(e) => {
                tracing::error!("Failed to parse Google AI response: {}", e);
                proxy_error(
                    StatusCode::BAD_GATEWAY,
                    "Failed to parse Google AI response",
                    "upstream_error",
                    "GOOGLE_PARSE_ERROR",
                )
            }
        }
    } else {
        forward_response_with_status(response, status).await
    }
}

/// Forward request to Qwen (DashScope)
/// Requirements: 3.1-3.5, 4.1-4.5
async fn forward_to_qwen(
    state: &Arc<AppState>,
    service: &ApiKeyServiceImpl,
    user_id: uuid::Uuid,
    body: ChatCompletionRequest,
) -> Response {
    // Get user's Qwen API key
    let api_key = match service
        .get_decrypted_key(&state.db, user_id, AiProvider::Qwen)
        .await
    {
        Ok(key) => key,
        Err(_) => {
            return proxy_error(
                StatusCode::BAD_REQUEST,
                "Qwen API key not configured",
                "api_key_missing",
                "QWEN_KEY_NOT_CONFIGURED",
            );
        }
    };

    // Transform request to Qwen format
    let transformer_request: crate::services::transformers::ChatCompletionRequest = body.clone().into();
    let qwen_request = QwenTransformer::transform_request(&transformer_request);
    let is_streaming = body.stream;
    let model = body.model.clone();

    let client = Client::new();
    let url = "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation";

    // Add SSE header for streaming
    let mut request_builder = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json");
    
    if is_streaming {
        request_builder = request_builder.header("X-DashScope-SSE", "enable");
    }

    let response = match request_builder.json(&qwen_request).send().await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to forward request to Qwen: {}", e);
            return proxy_error(
                StatusCode::BAD_GATEWAY,
                "Failed to connect to Qwen",
                "upstream_error",
                "QWEN_CONNECTION_ERROR",
            );
        }
    };

    // Handle streaming response
    let status = response.status();
    if is_streaming && status.is_success() {
        return forward_qwen_stream(response, model).await;
    }

    // Transform response back to OpenAI format
    if status.is_success() {
        match response.json::<crate::services::transformers::qwen::QwenResponse>().await {
            Ok(qwen_resp) => {
                let openai_resp = QwenTransformer::transform_response(qwen_resp, &body.model);
                (StatusCode::OK, Json(openai_resp)).into_response()
            }
            Err(e) => {
                tracing::error!("Failed to parse Qwen response: {}", e);
                proxy_error(
                    StatusCode::BAD_GATEWAY,
                    "Failed to parse Qwen response",
                    "upstream_error",
                    "QWEN_PARSE_ERROR",
                )
            }
        }
    } else {
        forward_response_with_status(response, status).await
    }
}

/// Forward streaming response (passthrough for OpenAI)
/// Requirements: 4.1-4.3
async fn forward_stream_response(response: reqwest::Response) -> Response {
    let stream = stream! {
        let mut byte_stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // Process complete lines
                    while let Some(pos) = buffer.find("\n\n") {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();
                        
                        if line.starts_with("data: ") {
                            yield Ok::<_, Infallible>(Event::default().data(&line[6..]));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                    break;
                }
            }
        }
        
        // Send [DONE] at the end
        yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}

/// Forward Anthropic streaming response with transformation
/// Requirements: 4.1-4.5
async fn forward_anthropic_stream(response: reqwest::Response, model: String) -> Response {
    let stream = stream! {
        let mut byte_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut message_id = String::new();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // Process complete SSE events
                    while let Some(pos) = buffer.find("\n\n") {
                        let event_block = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();
                        
                        // Parse event type and data
                        let mut event_type = String::new();
                        let mut data = String::new();
                        
                        for line in event_block.lines() {
                            if line.starts_with("event: ") {
                                event_type = line[7..].to_string();
                            } else if line.starts_with("data: ") {
                                data = line[6..].to_string();
                            }
                        }
                        
                        if data.is_empty() {
                            continue;
                        }
                        
                        // Parse and transform Anthropic event
                        if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(&data) {
                            // Extract message ID from message_start
                            if let AnthropicStreamEvent::MessageStart { ref message } = event {
                                message_id = message.id.clone();
                            }
                            
                            if let Some(chunk) = StreamHandler::transform_anthropic_chunk(&event, &message_id, &model) {
                                let sse_data = serde_json::to_string(&chunk).unwrap_or_default();
                                yield Ok::<_, Infallible>(Event::default().data(sse_data));
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Anthropic stream error: {}", e);
                    break;
                }
            }
        }
        
        yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}

/// Forward Google streaming response with transformation
/// Requirements: 4.1-4.5
async fn forward_google_stream(response: reqwest::Response, model: String) -> Response {
    let stream = stream! {
        let mut byte_stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // Process complete lines
                    while let Some(pos) = buffer.find("\n") {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 1..].to_string();
                        
                        if let Some(data) = StreamHandler::parse_sse_line(&line) {
                            if let Ok(google_chunk) = serde_json::from_str::<GoogleStreamChunk>(&data) {
                                if let Some(chunk) = StreamHandler::transform_google_chunk(&google_chunk, &model) {
                                    let sse_data = serde_json::to_string(&chunk).unwrap_or_default();
                                    yield Ok::<_, Infallible>(Event::default().data(sse_data));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Google stream error: {}", e);
                    break;
                }
            }
        }
        
        yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}

/// Forward Qwen streaming response with transformation
/// Requirements: 4.1-4.5
async fn forward_qwen_stream(response: reqwest::Response, model: String) -> Response {
    let stream = stream! {
        let mut byte_stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // Process complete lines
                    while let Some(pos) = buffer.find("\n\n") {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();
                        
                        if let Some(data) = StreamHandler::parse_sse_line(&line) {
                            if let Ok(qwen_chunk) = serde_json::from_str::<QwenStreamChunk>(&data) {
                                if let Some(chunk) = StreamHandler::transform_qwen_chunk(&qwen_chunk, &model) {
                                    let sse_data = serde_json::to_string(&chunk).unwrap_or_default();
                                    yield Ok::<_, Infallible>(Event::default().data(sse_data));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Qwen stream error: {}", e);
                    break;
                }
            }
        }
        
        yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}

/// Forward response from upstream provider
async fn forward_response(response: reqwest::Response) -> Response {
    let status_code = response.status().as_u16();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match response.bytes().await {
        Ok(bytes) => {
            let axum_status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK);
            let mut builder = Response::builder().status(axum_status);

            if let Some(ct) = content_type {
                builder = builder.header("Content-Type", ct);
            }

            builder.body(Body::from(bytes)).unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap()
            })
        }
        Err(e) => {
            tracing::error!("Failed to read upstream response: {}", e);
            proxy_error(
                StatusCode::BAD_GATEWAY,
                "Failed to read response from provider",
                "upstream_error",
                "RESPONSE_READ_ERROR",
            )
        }
    }
}

/// Forward response with specific status
async fn forward_response_with_status(response: reqwest::Response, _status: reqwest::StatusCode) -> Response {
    forward_response(response).await
}

/// Helper function to create proxy error responses
fn proxy_error(status: StatusCode, message: &str, error_type: &str, code: &str) -> Response {
    let body = Json(ProxyErrorResponse {
        error: ProxyError {
            message: message.to_string(),
            r#type: error_type.to_string(),
            code: code.to_string(),
        },
    });

    (status, body).into_response()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::transformers::Provider;

    // ============================================================
    // Unit Tests for Multi-Provider Proxy (Tasks 1-4)
    // **Validates: Requirements 1.1, 2.1, 3.1, 5.1, 5.2**
    // ============================================================

    #[test]
    fn test_provider_routing_openai() {
        assert_eq!(Provider::from_model("gpt-4"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_model("gpt-4-turbo"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_model("gpt-3.5-turbo"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_model("o1-preview"), Some(Provider::OpenAI));
    }

    #[test]
    fn test_provider_routing_anthropic() {
        assert_eq!(Provider::from_model("claude-3-opus"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_model("claude-3-sonnet"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_model("claude-3-haiku"), Some(Provider::Anthropic));
    }

    #[test]
    fn test_provider_routing_google() {
        assert_eq!(Provider::from_model("gemini-pro"), Some(Provider::Google));
        assert_eq!(Provider::from_model("gemini-1.5-pro"), Some(Provider::Google));
        assert_eq!(Provider::from_model("gemini-1.5-flash"), Some(Provider::Google));
    }

    #[test]
    fn test_provider_routing_qwen() {
        assert_eq!(Provider::from_model("qwen-turbo"), Some(Provider::Qwen));
        assert_eq!(Provider::from_model("qwen-plus"), Some(Provider::Qwen));
        assert_eq!(Provider::from_model("qwen-max"), Some(Provider::Qwen));
    }

    #[test]
    fn test_provider_routing_unknown() {
        assert_eq!(Provider::from_model("llama-2"), None);
        assert_eq!(Provider::from_model("mistral-7b"), None);
        assert_eq!(Provider::from_model("unknown"), None);
    }

    #[test]
    fn test_chat_completion_request_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: false,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_message_conversion() {
        let msg = Message {
            role: "user".to_string(),
            content: "Test".to_string(),
        };

        let transformer_msg: crate::services::transformers::Message = msg.into();
        assert_eq!(transformer_msg.role, "user");
        assert_eq!(transformer_msg.content, "Test");
    }

    #[test]
    fn test_proxy_error_struct() {
        let error = ProxyError {
            message: "Test message".to_string(),
            r#type: "test_type".to_string(),
            code: "TEST_CODE".to_string(),
        };

        assert_eq!(error.message, "Test message");
        assert_eq!(error.r#type, "test_type");
        assert_eq!(error.code, "TEST_CODE");
    }

    // Property Test 5: Model Routing Correctness
    // **Feature: week2-multi-provider, Property 5: Model Routing Correctness**
    // **Validates: Requirements 1.1, 2.1, 3.1**
    #[test]
    fn prop_model_routing_correctness() {
        // All gpt-* models should route to OpenAI
        for model in ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo", "gpt-4o"] {
            assert_eq!(Provider::from_model(model), Some(Provider::OpenAI));
        }

        // All claude-* models should route to Anthropic
        for model in ["claude-3-opus", "claude-3-sonnet", "claude-2.1"] {
            assert_eq!(Provider::from_model(model), Some(Provider::Anthropic));
        }

        // All gemini-* models should route to Google
        for model in ["gemini-pro", "gemini-1.5-pro", "gemini-1.5-flash"] {
            assert_eq!(Provider::from_model(model), Some(Provider::Google));
        }

        // All qwen-* models should route to Qwen
        for model in ["qwen-turbo", "qwen-plus", "qwen-max"] {
            assert_eq!(Provider::from_model(model), Some(Provider::Qwen));
        }
    }
}
