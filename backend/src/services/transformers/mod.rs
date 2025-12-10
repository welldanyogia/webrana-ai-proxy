//! Request/Response transformers for AI providers.
//!
//! This module provides transformers that convert between Webrana's unified
//! OpenAI-compatible format and provider-specific formats.

pub mod anthropic;
pub mod google;
pub mod qwen;

#[cfg(test)]
mod property_tests;

use serde::{Deserialize, Serialize};

/// Unified chat completion request (OpenAI-compatible format)
#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Unified chat completion response (OpenAI-compatible format)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

/// AI Provider enum for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Google,
    Qwen,
}

impl Provider {
    /// Determine provider from model name
    /// Requirements: 1.1, 2.1, 3.1
    pub fn from_model(model: &str) -> Option<Self> {
        if model.starts_with("gpt-") || model.starts_with("o1-") {
            Some(Provider::OpenAI)
        } else if model.starts_with("claude-") {
            Some(Provider::Anthropic)
        } else if model.starts_with("gemini-") {
            Some(Provider::Google)
        } else if model.starts_with("qwen-") || model.starts_with("qwen2-") {
            Some(Provider::Qwen)
        } else {
            None
        }
    }

    /// Get provider name for display
    pub fn name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::Google => "Google",
            Provider::Qwen => "Qwen",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Property Test 5: Model Routing Correctness
    // **Feature: week2-multi-provider, Property 5: Model Routing Correctness**
    // **Validates: Requirements 1.1, 2.1, 3.1**
    // ============================================================

    #[test]
    fn test_provider_from_model_openai() {
        assert_eq!(Provider::from_model("gpt-4"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_model("gpt-4-turbo"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_model("gpt-3.5-turbo"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_model("o1-preview"), Some(Provider::OpenAI));
        assert_eq!(Provider::from_model("o1-mini"), Some(Provider::OpenAI));
    }

    #[test]
    fn test_provider_from_model_anthropic() {
        assert_eq!(Provider::from_model("claude-3-opus"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_model("claude-3-sonnet"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_model("claude-3-haiku"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_model("claude-2.1"), Some(Provider::Anthropic));
    }

    #[test]
    fn test_provider_from_model_google() {
        assert_eq!(Provider::from_model("gemini-pro"), Some(Provider::Google));
        assert_eq!(Provider::from_model("gemini-1.5-pro"), Some(Provider::Google));
        assert_eq!(Provider::from_model("gemini-1.5-flash"), Some(Provider::Google));
    }

    #[test]
    fn test_provider_from_model_qwen() {
        assert_eq!(Provider::from_model("qwen-turbo"), Some(Provider::Qwen));
        assert_eq!(Provider::from_model("qwen-plus"), Some(Provider::Qwen));
        assert_eq!(Provider::from_model("qwen-max"), Some(Provider::Qwen));
    }

    #[test]
    fn test_provider_from_model_unknown() {
        assert_eq!(Provider::from_model("llama-2"), None);
        assert_eq!(Provider::from_model("mistral-7b"), None);
        assert_eq!(Provider::from_model("unknown-model"), None);
    }

    #[test]
    fn test_provider_name() {
        assert_eq!(Provider::OpenAI.name(), "OpenAI");
        assert_eq!(Provider::Anthropic.name(), "Anthropic");
        assert_eq!(Provider::Google.name(), "Google");
        assert_eq!(Provider::Qwen.name(), "Qwen");
    }
}
