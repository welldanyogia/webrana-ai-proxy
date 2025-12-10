//! Anthropic (Claude) request/response transformer.
//!
//! Requirements: 1.1-1.5 - Anthropic proxy support
//!
//! Transforms between OpenAI-compatible format and Anthropic Messages API format.

use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

use super::{ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Usage};

/// Anthropic Messages API request format
/// https://docs.anthropic.com/en/api/messages
#[derive(Debug, Clone, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

/// Anthropic Messages API response format
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<AnthropicContent>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicContent {
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: i32,
    pub output_tokens: i32,
}

/// Anthropic transformer
/// Requirements: 1.2, 1.3, 1.4
pub struct AnthropicTransformer;

impl AnthropicTransformer {
    /// Transform OpenAI-compatible request to Anthropic format
    /// Requirements: 1.2, 1.3
    pub fn transform_request(request: &ChatCompletionRequest) -> AnthropicRequest {
        // Extract system message if present
        let mut system_message: Option<String> = None;
        let mut messages: Vec<AnthropicMessage> = Vec::new();

        for msg in &request.messages {
            if msg.role == "system" {
                // Anthropic requires system as separate parameter
                system_message = Some(msg.content.clone());
            } else {
                messages.push(AnthropicMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }

        // Requirement 1.3: max_tokens is required for Anthropic
        // Default to 4096 if not specified
        let max_tokens = request.max_tokens.unwrap_or(4096);

        AnthropicRequest {
            model: request.model.clone(),
            max_tokens,
            system: system_message,
            messages,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop.clone(),
            stream: if request.stream { Some(true) } else { None },
        }
    }

    /// Transform Anthropic response to OpenAI-compatible format
    /// Requirement: 1.4
    pub fn transform_response(response: AnthropicResponse) -> ChatCompletionResponse {
        // Combine all content blocks into single message
        let content = response
            .content
            .iter()
            .filter(|c| c.r#type == "text")
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("");

        // Map Anthropic stop_reason to OpenAI finish_reason
        let finish_reason = response.stop_reason.map(|reason| {
            match reason.as_str() {
                "end_turn" => "stop".to_string(),
                "max_tokens" => "length".to_string(),
                "stop_sequence" => "stop".to_string(),
                other => other.to_string(),
            }
        });

        ChatCompletionResponse {
            id: format!("chatcmpl-{}", response.id),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: response.model,
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content,
                },
                finish_reason,
            }],
            usage: Usage {
                prompt_tokens: response.usage.input_tokens,
                completion_tokens: response.usage.output_tokens,
                total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            },
        }
    }

    /// Get Anthropic API base URL
    pub fn base_url() -> &'static str {
        "https://api.anthropic.com/v1/messages"
    }

    /// Get required headers for Anthropic API
    pub fn headers(api_key: &str) -> Vec<(&'static str, String)> {
        vec![
            ("x-api-key", api_key.to_string()),
            ("anthropic-version", "2023-06-01".to_string()),
            ("content-type", "application/json".to_string()),
        ]
    }

    /// Supported Claude models
    pub fn supported_models() -> &'static [&'static str] {
        &[
            "claude-3-opus-20240229",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
            "claude-3-5-sonnet-20241022",
            "claude-2.1",
            "claude-2.0",
            "claude-instant-1.2",
        ]
    }

    /// Check if model is a Claude model
    pub fn is_anthropic_model(model: &str) -> bool {
        model.starts_with("claude-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Unit Tests for Anthropic Transformer (Task 1.1, 1.2)
    // **Validates: Requirements 1.2, 1.3, 1.4**
    // ============================================================

    #[test]
    fn test_transform_request_basic() {
        let request = ChatCompletionRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: "Hello, Claude!".to_string(),
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
        };

        let anthropic_req = AnthropicTransformer::transform_request(&request);

        assert_eq!(anthropic_req.model, "claude-3-sonnet-20240229");
        assert_eq!(anthropic_req.max_tokens, 1000);
        assert_eq!(anthropic_req.messages.len(), 1);
        assert_eq!(anthropic_req.messages[0].role, "user");
        assert_eq!(anthropic_req.messages[0].content, "Hello, Claude!");
        assert!(anthropic_req.system.is_none());
    }

    #[test]
    fn test_transform_request_with_system_message() {
        let request = ChatCompletionRequest {
            model: "claude-3-opus-20240229".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Hello!".to_string(),
                },
            ],
            temperature: None,
            max_tokens: None,
            stream: false,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
        };

        let anthropic_req = AnthropicTransformer::transform_request(&request);

        // System message should be extracted
        assert_eq!(anthropic_req.system, Some("You are a helpful assistant.".to_string()));
        // Only user message should remain in messages array
        assert_eq!(anthropic_req.messages.len(), 1);
        assert_eq!(anthropic_req.messages[0].role, "user");
    }

    #[test]
    fn test_transform_request_default_max_tokens() {
        // Requirement 1.3: max_tokens is required for Anthropic
        let request = ChatCompletionRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: None,
            max_tokens: None, // Not specified
            stream: false,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
        };

        let anthropic_req = AnthropicTransformer::transform_request(&request);

        // Should default to 4096
        assert_eq!(anthropic_req.max_tokens, 4096);
    }

    #[test]
    fn test_transform_response() {
        let anthropic_response = AnthropicResponse {
            id: "msg_123".to_string(),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![AnthropicContent {
                r#type: "text".to_string(),
                text: "Hello! How can I help you today?".to_string(),
            }],
            model: "claude-3-sonnet-20240229".to_string(),
            stop_reason: Some("end_turn".to_string()),
            stop_sequence: None,
            usage: AnthropicUsage {
                input_tokens: 10,
                output_tokens: 20,
            },
        };

        let response = AnthropicTransformer::transform_response(anthropic_response);

        assert!(response.id.starts_with("chatcmpl-"));
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.model, "claude-3-sonnet-20240229");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you today?");
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.usage.completion_tokens, 20);
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[test]
    fn test_transform_response_max_tokens_stop() {
        let anthropic_response = AnthropicResponse {
            id: "msg_456".to_string(),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![AnthropicContent {
                r#type: "text".to_string(),
                text: "Truncated response...".to_string(),
            }],
            model: "claude-3-opus-20240229".to_string(),
            stop_reason: Some("max_tokens".to_string()),
            stop_sequence: None,
            usage: AnthropicUsage {
                input_tokens: 100,
                output_tokens: 4096,
            },
        };

        let response = AnthropicTransformer::transform_response(anthropic_response);

        // max_tokens should map to "length"
        assert_eq!(response.choices[0].finish_reason, Some("length".to_string()));
    }

    #[test]
    fn test_is_anthropic_model() {
        assert!(AnthropicTransformer::is_anthropic_model("claude-3-opus-20240229"));
        assert!(AnthropicTransformer::is_anthropic_model("claude-3-sonnet-20240229"));
        assert!(AnthropicTransformer::is_anthropic_model("claude-2.1"));
        assert!(!AnthropicTransformer::is_anthropic_model("gpt-4"));
        assert!(!AnthropicTransformer::is_anthropic_model("gemini-pro"));
    }

    #[test]
    fn test_headers() {
        let headers = AnthropicTransformer::headers("sk-ant-test-key");
        
        assert_eq!(headers.len(), 3);
        assert!(headers.iter().any(|(k, v)| *k == "x-api-key" && v == "sk-ant-test-key"));
        assert!(headers.iter().any(|(k, v)| *k == "anthropic-version" && v == "2023-06-01"));
    }

    // ============================================================
    // Property Test 1: Request Format Transformation Consistency
    // **Feature: week2-multi-provider, Property 1: Request Format Transformation Consistency**
    // **Validates: Requirements 1.2, 2.2, 3.2**
    // ============================================================

    #[test]
    fn prop_request_preserves_messages() {
        // For any request, transformation should preserve message content
        let messages = vec![
            Message { role: "system".to_string(), content: "System prompt".to_string() },
            Message { role: "user".to_string(), content: "User message".to_string() },
            Message { role: "assistant".to_string(), content: "Assistant reply".to_string() },
            Message { role: "user".to_string(), content: "Follow up".to_string() },
        ];

        let request = ChatCompletionRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: messages.clone(),
            temperature: Some(0.5),
            max_tokens: Some(2000),
            stream: false,
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            stop: Some(vec!["STOP".to_string()]),
            user: None,
        };

        let anthropic_req = AnthropicTransformer::transform_request(&request);

        // System message should be extracted
        assert_eq!(anthropic_req.system, Some("System prompt".to_string()));

        // Non-system messages should be preserved
        assert_eq!(anthropic_req.messages.len(), 3);
        assert_eq!(anthropic_req.messages[0].content, "User message");
        assert_eq!(anthropic_req.messages[1].content, "Assistant reply");
        assert_eq!(anthropic_req.messages[2].content, "Follow up");

        // Parameters should be preserved
        assert_eq!(anthropic_req.temperature, Some(0.5));
        assert_eq!(anthropic_req.top_p, Some(0.9));
        assert_eq!(anthropic_req.stop_sequences, Some(vec!["STOP".to_string()]));
    }
}
