//! SSE Stream Handler for AI provider responses.
//!
//! Requirements: 4.1-4.5 - Streaming support for all providers
//!
//! Handles Server-Sent Events (SSE) streaming from AI providers and transforms
//! chunks to OpenAI-compatible format.

use async_stream::stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::services::transformers::Provider;

/// OpenAI-compatible streaming chunk format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: i32,
    pub delta: StreamDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Anthropic streaming event types
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: AnthropicMessageStart },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: i32, content_block: AnthropicContentBlock },

    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: i32, delta: AnthropicDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: i32 },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: AnthropicMessageDeltaContent, usage: Option<AnthropicUsageDelta> },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: AnthropicError },
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicMessageStart {
    pub id: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicContentBlock {
    pub r#type: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicDelta {
    pub r#type: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicMessageDeltaContent {
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicUsageDelta {
    pub output_tokens: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicError {
    pub r#type: String,
    pub message: String,
}

/// Google streaming response
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleStreamChunk {
    pub candidates: Option<Vec<GoogleCandidate>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleCandidate {
    pub content: Option<GoogleContent>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleContent {
    pub parts: Option<Vec<GooglePart>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GooglePart {
    pub text: Option<String>,
}

/// Qwen streaming response
#[derive(Debug, Clone, Deserialize)]
pub struct QwenStreamChunk {
    pub output: QwenStreamOutput,
    pub request_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QwenStreamOutput {
    pub text: Option<String>,
    pub finish_reason: Option<String>,
}

/// Stream handler for transforming provider SSE to OpenAI format
pub struct StreamHandler;

impl StreamHandler {
    /// Parse SSE line and extract data
    pub fn parse_sse_line(line: &str) -> Option<String> {
        if line.starts_with("data: ") {
            let data = line.strip_prefix("data: ")?;
            if data == "[DONE]" {
                return None;
            }
            Some(data.to_string())
        } else {
            None
        }
    }

    /// Transform Anthropic stream event to OpenAI chunk
    pub fn transform_anthropic_chunk(
        event: &AnthropicStreamEvent,
        message_id: &str,
        model: &str,
    ) -> Option<StreamChunk> {
        match event {
            AnthropicStreamEvent::ContentBlockStart { index, content_block } => {
                // First chunk with role
                Some(StreamChunk {
                    id: format!("chatcmpl-{}", message_id),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: model.to_string(),
                    choices: vec![StreamChoice {
                        index: *index,
                        delta: StreamDelta {
                            role: Some("assistant".to_string()),
                            content: if content_block.text.is_empty() {
                                None
                            } else {
                                Some(content_block.text.clone())
                            },
                        },
                        finish_reason: None,
                    }],
                })
            }
            AnthropicStreamEvent::ContentBlockDelta { index, delta } => {
                if delta.text.is_empty() {
                    return None;
                }
                Some(StreamChunk {
                    id: format!("chatcmpl-{}", message_id),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: model.to_string(),
                    choices: vec![StreamChoice {
                        index: *index,
                        delta: StreamDelta {
                            role: None,
                            content: Some(delta.text.clone()),
                        },
                        finish_reason: None,
                    }],
                })
            }
            AnthropicStreamEvent::MessageDelta { delta, .. } => {
                let finish_reason = delta.stop_reason.as_ref().map(|r| {
                    match r.as_str() {
                        "end_turn" => "stop".to_string(),
                        "max_tokens" => "length".to_string(),
                        other => other.to_string(),
                    }
                });
                Some(StreamChunk {
                    id: format!("chatcmpl-{}", message_id),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: model.to_string(),
                    choices: vec![StreamChoice {
                        index: 0,
                        delta: StreamDelta {
                            role: None,
                            content: None,
                        },
                        finish_reason,
                    }],
                })
            }
            _ => None,
        }
    }

    /// Transform Google stream chunk to OpenAI format
    pub fn transform_google_chunk(chunk: &GoogleStreamChunk, model: &str) -> Option<StreamChunk> {
        let candidates = chunk.candidates.as_ref()?;
        let candidate = candidates.first()?;
        
        let content = candidate.content.as_ref()
            .and_then(|c| c.parts.as_ref())
            .and_then(|p| p.first())
            .and_then(|p| p.text.clone());

        let finish_reason = candidate.finish_reason.as_ref().map(|r| {
            match r.as_str() {
                "STOP" => "stop".to_string(),
                "MAX_TOKENS" => "length".to_string(),
                other => other.to_lowercase(),
            }
        });

        Some(StreamChunk {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![StreamChoice {
                index: 0,
                delta: StreamDelta {
                    role: if content.is_some() { Some("assistant".to_string()) } else { None },
                    content,
                },
                finish_reason,
            }],
        })
    }

    /// Transform Qwen stream chunk to OpenAI format
    pub fn transform_qwen_chunk(chunk: &QwenStreamChunk, model: &str) -> Option<StreamChunk> {
        let finish_reason = chunk.output.finish_reason.as_ref().map(|r| {
            match r.as_str() {
                "stop" | "null" => "stop".to_string(),
                "length" => "length".to_string(),
                other => other.to_string(),
            }
        });

        Some(StreamChunk {
            id: format!("chatcmpl-{}", chunk.request_id),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![StreamChoice {
                index: 0,
                delta: StreamDelta {
                    role: Some("assistant".to_string()),
                    content: chunk.output.text.clone(),
                },
                finish_reason,
            }],
        })
    }

    /// Format chunk as SSE data line
    pub fn format_sse_chunk(chunk: &StreamChunk) -> String {
        format!("data: {}\n\n", serde_json::to_string(chunk).unwrap_or_default())
    }

    /// Format SSE done message
    pub fn format_sse_done() -> String {
        "data: [DONE]\n\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line_data() {
        let line = "data: {\"test\": true}";
        assert_eq!(StreamHandler::parse_sse_line(line), Some("{\"test\": true}".to_string()));
    }

    #[test]
    fn test_parse_sse_line_done() {
        let line = "data: [DONE]";
        assert_eq!(StreamHandler::parse_sse_line(line), None);
    }

    #[test]
    fn test_parse_sse_line_other() {
        let line = "event: message";
        assert_eq!(StreamHandler::parse_sse_line(line), None);
    }

    #[test]
    fn test_format_sse_chunk() {
        let chunk = StreamChunk {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![StreamChoice {
                index: 0,
                delta: StreamDelta {
                    role: None,
                    content: Some("Hello".to_string()),
                },
                finish_reason: None,
            }],
        };

        let sse = StreamHandler::format_sse_chunk(&chunk);
        assert!(sse.starts_with("data: "));
        assert!(sse.ends_with("\n\n"));
        assert!(sse.contains("Hello"));
    }

    #[test]
    fn test_format_sse_done() {
        assert_eq!(StreamHandler::format_sse_done(), "data: [DONE]\n\n");
    }

    #[test]
    fn test_transform_google_chunk() {
        let chunk = GoogleStreamChunk {
            candidates: Some(vec![GoogleCandidate {
                content: Some(GoogleContent {
                    parts: Some(vec![GooglePart {
                        text: Some("Hello world".to_string()),
                    }]),
                }),
                finish_reason: None,
            }]),
        };

        let result = StreamHandler::transform_google_chunk(&chunk, "gemini-pro");
        assert!(result.is_some());
        let stream_chunk = result.unwrap();
        assert_eq!(stream_chunk.model, "gemini-pro");
        assert_eq!(stream_chunk.choices[0].delta.content, Some("Hello world".to_string()));
    }

    #[test]
    fn test_transform_qwen_chunk() {
        let chunk = QwenStreamChunk {
            output: QwenStreamOutput {
                text: Some("Test response".to_string()),
                finish_reason: None,
            },
            request_id: "req-123".to_string(),
        };

        let result = StreamHandler::transform_qwen_chunk(&chunk, "qwen-turbo");
        assert!(result.is_some());
        let stream_chunk = result.unwrap();
        assert!(stream_chunk.id.contains("req-123"));
        assert_eq!(stream_chunk.choices[0].delta.content, Some("Test response".to_string()));
    }
}
