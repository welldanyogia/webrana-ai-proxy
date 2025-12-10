//! Alibaba Qwen request/response transformer.
//!
//! Requirements: 3.1-3.5 - Qwen proxy support
//!
//! Transforms between OpenAI-compatible format and Alibaba DashScope API format.

use serde::{Deserialize, Serialize};
use chrono::Utc;

use super::{ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Usage};

/// Alibaba DashScope API request format
/// https://help.aliyun.com/zh/dashscope/developer-reference/api-details
#[derive(Debug, Clone, Serialize)]
pub struct QwenRequest {
    pub model: String,
    pub input: QwenInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<QwenParameters>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QwenInput {
    pub messages: Vec<QwenMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct QwenParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental_output: Option<bool>,
}

/// Alibaba DashScope API response format
#[derive(Debug, Clone, Deserialize)]
pub struct QwenResponse {
    pub output: QwenOutput,
    pub usage: QwenUsage,
    pub request_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QwenOutput {
    pub text: Option<String>,
    pub finish_reason: Option<String>,
    pub choices: Option<Vec<QwenChoice>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QwenChoice {
    pub finish_reason: String,
    pub message: QwenMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QwenUsage {
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub total_tokens: Option<i32>,
}

/// Qwen transformer
/// Requirements: 3.2, 3.3, 3.4
pub struct QwenTransformer;

impl QwenTransformer {
    /// Transform OpenAI-compatible request to Qwen format
    /// Requirements: 3.2, 3.3
    pub fn transform_request(request: &ChatCompletionRequest) -> QwenRequest {
        let messages: Vec<QwenMessage> = request
            .messages
            .iter()
            .map(|msg| QwenMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        // Build parameters if any are set
        let parameters = if request.temperature.is_some()
            || request.top_p.is_some()
            || request.max_tokens.is_some()
            || request.stop.is_some()
            || request.stream
        {
            Some(QwenParameters {
                temperature: request.temperature,
                top_p: request.top_p,
                max_tokens: request.max_tokens,
                stop: request.stop.clone(),
                enable_search: None,
                result_format: Some("message".to_string()), // Use message format for consistency
                incremental_output: if request.stream { Some(true) } else { None },
            })
        } else {
            Some(QwenParameters {
                temperature: None,
                top_p: None,
                max_tokens: None,
                stop: None,
                enable_search: None,
                result_format: Some("message".to_string()),
                incremental_output: None,
            })
        };

        QwenRequest {
            model: request.model.clone(),
            input: QwenInput { messages },
            parameters,
        }
    }

    /// Transform Qwen response to OpenAI-compatible format
    /// Requirement: 3.4
    pub fn transform_response(response: QwenResponse, model: &str) -> ChatCompletionResponse {
        // Handle both text format and message format responses
        let (content, finish_reason) = if let Some(choices) = &response.output.choices {
            // Message format (result_format: "message")
            if let Some(choice) = choices.first() {
                (
                    choice.message.content.clone(),
                    Some(Self::map_finish_reason(&choice.finish_reason)),
                )
            } else {
                (String::new(), None)
            }
        } else {
            // Text format (default)
            (
                response.output.text.unwrap_or_default(),
                response.output.finish_reason.map(|r| Self::map_finish_reason(&r)),
            )
        };

        ChatCompletionResponse {
            id: format!("chatcmpl-{}", response.request_id),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
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
                total_tokens: response.usage.total_tokens
                    .unwrap_or(response.usage.input_tokens + response.usage.output_tokens),
            },
        }
    }

    /// Map Qwen finish reasons to OpenAI format
    fn map_finish_reason(reason: &str) -> String {
        match reason {
            "stop" => "stop".to_string(),
            "length" => "length".to_string(),
            "null" => "stop".to_string(),
            other => other.to_string(),
        }
    }

    /// Get DashScope API URL
    pub fn api_url() -> &'static str {
        "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation"
    }

    /// Get required headers for DashScope API
    pub fn headers(api_key: &str) -> Vec<(&'static str, String)> {
        vec![
            ("Authorization", format!("Bearer {}", api_key)),
            ("Content-Type", "application/json".to_string()),
        ]
    }

    /// Supported Qwen models
    pub fn supported_models() -> &'static [&'static str] {
        &[
            "qwen-turbo",
            "qwen-plus",
            "qwen-max",
            "qwen-max-longcontext",
            "qwen-7b-chat",
            "qwen-14b-chat",
            "qwen-72b-chat",
        ]
    }

    /// Check if model is a Qwen model
    pub fn is_qwen_model(model: &str) -> bool {
        model.starts_with("qwen-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Unit Tests for Qwen Transformer (Task 3.1, 3.2)
    // **Validates: Requirements 3.2, 3.3, 3.4**
    // ============================================================

    #[test]
    fn test_transform_request_basic() {
        let request = ChatCompletionRequest {
            model: "qwen-turbo".to_string(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: "Hello, Qwen!".to_string(),
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

        let qwen_req = QwenTransformer::transform_request(&request);

        assert_eq!(qwen_req.model, "qwen-turbo");
        assert_eq!(qwen_req.input.messages.len(), 1);
        assert_eq!(qwen_req.input.messages[0].role, "user");
        assert_eq!(qwen_req.input.messages[0].content, "Hello, Qwen!");
        
        let params = qwen_req.parameters.unwrap();
        assert_eq!(params.temperature, Some(0.7));
        assert_eq!(params.max_tokens, Some(1000));
        assert_eq!(params.result_format, Some("message".to_string()));
    }

    #[test]
    fn test_transform_request_with_system() {
        let request = ChatCompletionRequest {
            model: "qwen-plus".to_string(),
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

        let qwen_req = QwenTransformer::transform_request(&request);

        // Qwen supports system messages directly
        assert_eq!(qwen_req.input.messages.len(), 2);
        assert_eq!(qwen_req.input.messages[0].role, "system");
        assert_eq!(qwen_req.input.messages[1].role, "user");
    }

    #[test]
    fn test_transform_request_streaming() {
        let request = ChatCompletionRequest {
            model: "qwen-turbo".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: true, // Streaming enabled
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
        };

        let qwen_req = QwenTransformer::transform_request(&request);

        let params = qwen_req.parameters.unwrap();
        assert_eq!(params.incremental_output, Some(true));
    }

    #[test]
    fn test_transform_response_message_format() {
        let qwen_response = QwenResponse {
            output: QwenOutput {
                text: None,
                finish_reason: None,
                choices: Some(vec![QwenChoice {
                    finish_reason: "stop".to_string(),
                    message: QwenMessage {
                        role: "assistant".to_string(),
                        content: "Hello! How can I help you?".to_string(),
                    },
                }]),
            },
            usage: QwenUsage {
                input_tokens: 10,
                output_tokens: 15,
                total_tokens: Some(25),
            },
            request_id: "req-123".to_string(),
        };

        let response = QwenTransformer::transform_response(qwen_response, "qwen-turbo");

        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.model, "qwen-turbo");
        assert!(response.id.contains("req-123"));
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you?");
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.usage.completion_tokens, 15);
        assert_eq!(response.usage.total_tokens, 25);
    }

    #[test]
    fn test_transform_response_text_format() {
        let qwen_response = QwenResponse {
            output: QwenOutput {
                text: Some("This is a text response.".to_string()),
                finish_reason: Some("stop".to_string()),
                choices: None,
            },
            usage: QwenUsage {
                input_tokens: 5,
                output_tokens: 10,
                total_tokens: None,
            },
            request_id: "req-456".to_string(),
        };

        let response = QwenTransformer::transform_response(qwen_response, "qwen-plus");

        assert_eq!(response.choices[0].message.content, "This is a text response.");
        assert_eq!(response.usage.total_tokens, 15); // Calculated from input + output
    }

    #[test]
    fn test_is_qwen_model() {
        assert!(QwenTransformer::is_qwen_model("qwen-turbo"));
        assert!(QwenTransformer::is_qwen_model("qwen-plus"));
        assert!(QwenTransformer::is_qwen_model("qwen-max"));
        assert!(!QwenTransformer::is_qwen_model("gpt-4"));
        assert!(!QwenTransformer::is_qwen_model("claude-3"));
        assert!(!QwenTransformer::is_qwen_model("gemini-pro"));
    }

    #[test]
    fn test_headers() {
        let headers = QwenTransformer::headers("test-api-key");
        
        assert_eq!(headers.len(), 2);
        assert!(headers.iter().any(|(k, v)| *k == "Authorization" && v == "Bearer test-api-key"));
        assert!(headers.iter().any(|(k, v)| *k == "Content-Type" && v == "application/json"));
    }

    // ============================================================
    // Property Test 2: Response Format Normalization
    // **Feature: week2-multi-provider, Property 2: Response Format Normalization**
    // **Validates: Requirements 1.4, 2.4, 3.4**
    // ============================================================

    #[test]
    fn prop_response_has_required_fields() {
        // For any Qwen response, transformed response should have all required OpenAI fields
        let qwen_response = QwenResponse {
            output: QwenOutput {
                text: Some("Test response".to_string()),
                finish_reason: Some("stop".to_string()),
                choices: None,
            },
            usage: QwenUsage {
                input_tokens: 100,
                output_tokens: 200,
                total_tokens: Some(300),
            },
            request_id: "test-req".to_string(),
        };

        let response = QwenTransformer::transform_response(qwen_response, "qwen-turbo");

        // Required fields must be present
        assert!(!response.id.is_empty());
        assert_eq!(response.object, "chat.completion");
        assert!(response.created > 0);
        assert!(!response.model.is_empty());
        assert!(!response.choices.is_empty());
        
        // Choice must have required fields
        let choice = &response.choices[0];
        assert_eq!(choice.message.role, "assistant");
        
        // Usage must have required fields
        assert!(response.usage.prompt_tokens >= 0);
        assert!(response.usage.completion_tokens >= 0);
        assert!(response.usage.total_tokens >= 0);
    }
}
