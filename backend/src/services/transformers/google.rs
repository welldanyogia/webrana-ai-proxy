//! Google AI (Gemini) request/response transformer.
//!
//! Requirements: 2.1-2.5 - Google AI proxy support
//!
//! Transforms between OpenAI-compatible format and Google Generative AI API format.

use serde::{Deserialize, Serialize};
use chrono::Utc;

use super::{ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Usage};

/// Google Generative AI API request format
/// https://ai.google.dev/api/rest/v1beta/models/generateContent
#[derive(Debug, Clone, Serialize)]
pub struct GoogleRequest {
    pub contents: Vec<GoogleContent>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GoogleContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleContent {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(rename = "topP", skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(rename = "maxOutputTokens", skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Google Generative AI API response format
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleResponse {
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Candidate {
    pub content: GoogleContent,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    pub index: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<i32>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<i32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<i32>,
}

/// Google AI transformer
/// Requirements: 2.2, 2.3, 2.4
pub struct GoogleTransformer;

impl GoogleTransformer {
    /// Transform OpenAI-compatible request to Google format
    /// Requirements: 2.2, 2.3
    pub fn transform_request(request: &ChatCompletionRequest) -> GoogleRequest {
        let mut contents: Vec<GoogleContent> = Vec::new();
        let mut system_instruction: Option<GoogleContent> = None;

        for msg in &request.messages {
            if msg.role == "system" {
                // Google uses systemInstruction for system prompts
                system_instruction = Some(GoogleContent {
                    role: "user".to_string(), // System instruction uses user role
                    parts: vec![Part { text: msg.content.clone() }],
                });
            } else {
                // Map OpenAI roles to Google roles
                let role = match msg.role.as_str() {
                    "assistant" => "model",
                    _ => &msg.role,
                };

                contents.push(GoogleContent {
                    role: role.to_string(),
                    parts: vec![Part { text: msg.content.clone() }],
                });
            }
        }

        // Build generation config if any parameters are set
        let generation_config = if request.temperature.is_some()
            || request.top_p.is_some()
            || request.max_tokens.is_some()
            || request.stop.is_some()
        {
            Some(GenerationConfig {
                temperature: request.temperature,
                top_p: request.top_p,
                max_output_tokens: request.max_tokens,
                stop_sequences: request.stop.clone(),
            })
        } else {
            None
        };

        GoogleRequest {
            contents,
            generation_config,
            system_instruction,
        }
    }

    /// Transform Google response to OpenAI-compatible format
    /// Requirement: 2.4
    pub fn transform_response(response: GoogleResponse, model: &str) -> ChatCompletionResponse {
        let choices: Vec<Choice> = response
            .candidates
            .iter()
            .enumerate()
            .map(|(i, candidate)| {
                let content = candidate
                    .content
                    .parts
                    .iter()
                    .map(|p| p.text.clone())
                    .collect::<Vec<_>>()
                    .join("");

                // Map Google finish reasons to OpenAI format
                let finish_reason = candidate.finish_reason.as_ref().map(|reason| {
                    match reason.as_str() {
                        "STOP" => "stop".to_string(),
                        "MAX_TOKENS" => "length".to_string(),
                        "SAFETY" => "content_filter".to_string(),
                        "RECITATION" => "content_filter".to_string(),
                        other => other.to_lowercase(),
                    }
                });

                Choice {
                    index: candidate.index.unwrap_or(i as i32),
                    message: Message {
                        role: "assistant".to_string(),
                        content,
                    },
                    finish_reason,
                }
            })
            .collect();

        let usage = response.usage_metadata.map(|u| Usage {
            prompt_tokens: u.prompt_token_count.unwrap_or(0),
            completion_tokens: u.candidates_token_count.unwrap_or(0),
            total_tokens: u.total_token_count.unwrap_or(0),
        }).unwrap_or(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

        ChatCompletionResponse {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            choices,
            usage,
        }
    }

    /// Get Google AI API URL for a model
    pub fn api_url(model: &str, api_key: &str) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        )
    }

    /// Get required headers for Google AI API
    pub fn headers() -> Vec<(&'static str, String)> {
        vec![
            ("content-type", "application/json".to_string()),
        ]
    }

    /// Supported Gemini models
    pub fn supported_models() -> &'static [&'static str] {
        &[
            "gemini-pro",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
            "gemini-1.0-pro",
        ]
    }

    /// Check if model is a Gemini model
    pub fn is_google_model(model: &str) -> bool {
        model.starts_with("gemini-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Unit Tests for Google Transformer (Task 2.1, 2.2)
    // **Validates: Requirements 2.2, 2.3, 2.4**
    // ============================================================

    #[test]
    fn test_transform_request_basic() {
        let request = ChatCompletionRequest {
            model: "gemini-pro".to_string(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: "Hello, Gemini!".to_string(),
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

        let google_req = GoogleTransformer::transform_request(&request);

        assert_eq!(google_req.contents.len(), 1);
        assert_eq!(google_req.contents[0].role, "user");
        assert_eq!(google_req.contents[0].parts[0].text, "Hello, Gemini!");
        assert!(google_req.generation_config.is_some());
        
        let config = google_req.generation_config.unwrap();
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_output_tokens, Some(1000));
    }

    #[test]
    fn test_transform_request_with_system() {
        let request = ChatCompletionRequest {
            model: "gemini-1.5-pro".to_string(),
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

        let google_req = GoogleTransformer::transform_request(&request);

        // System should be in systemInstruction
        assert!(google_req.system_instruction.is_some());
        assert_eq!(
            google_req.system_instruction.unwrap().parts[0].text,
            "You are a helpful assistant."
        );

        // Only user message in contents
        assert_eq!(google_req.contents.len(), 1);
        assert_eq!(google_req.contents[0].role, "user");
    }

    #[test]
    fn test_transform_request_role_mapping() {
        let request = ChatCompletionRequest {
            model: "gemini-pro".to_string(),
            messages: vec![
                Message { role: "user".to_string(), content: "Hi".to_string() },
                Message { role: "assistant".to_string(), content: "Hello!".to_string() },
                Message { role: "user".to_string(), content: "How are you?".to_string() },
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

        let google_req = GoogleTransformer::transform_request(&request);

        assert_eq!(google_req.contents.len(), 3);
        assert_eq!(google_req.contents[0].role, "user");
        assert_eq!(google_req.contents[1].role, "model"); // assistant -> model
        assert_eq!(google_req.contents[2].role, "user");
    }

    #[test]
    fn test_transform_response() {
        let google_response = GoogleResponse {
            candidates: vec![Candidate {
                content: GoogleContent {
                    role: "model".to_string(),
                    parts: vec![Part {
                        text: "Hello! How can I help you?".to_string(),
                    }],
                },
                finish_reason: Some("STOP".to_string()),
                index: Some(0),
            }],
            usage_metadata: Some(UsageMetadata {
                prompt_token_count: Some(10),
                candidates_token_count: Some(15),
                total_token_count: Some(25),
            }),
        };

        let response = GoogleTransformer::transform_response(google_response, "gemini-pro");

        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.model, "gemini-pro");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you?");
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.usage.completion_tokens, 15);
        assert_eq!(response.usage.total_tokens, 25);
    }

    #[test]
    fn test_is_google_model() {
        assert!(GoogleTransformer::is_google_model("gemini-pro"));
        assert!(GoogleTransformer::is_google_model("gemini-1.5-pro"));
        assert!(GoogleTransformer::is_google_model("gemini-1.5-flash"));
        assert!(!GoogleTransformer::is_google_model("gpt-4"));
        assert!(!GoogleTransformer::is_google_model("claude-3"));
    }

    #[test]
    fn test_api_url() {
        let url = GoogleTransformer::api_url("gemini-pro", "test-api-key");
        assert!(url.contains("gemini-pro"));
        assert!(url.contains("key=test-api-key"));
        assert!(url.contains("generativelanguage.googleapis.com"));
    }
}
