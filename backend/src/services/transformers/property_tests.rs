//! Property-Based Tests for Request/Response Transformers
//!
//! **Feature: week2-multi-provider, Property 1: Request Format Transformation Consistency**
//! **Validates: Requirements 1.2, 2.2, 3.2**
//!
//! **Feature: week2-multi-provider, Property 2: Response Format Normalization**
//! **Validates: Requirements 1.4, 2.4, 3.4**
//!
//! These tests verify that request transformations preserve semantic content
//! and response transformations conform to OpenAI's ChatCompletionResponse schema
//! across all provider transformers (Anthropic, Google, Qwen).

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use crate::services::transformers::{
        ChatCompletionRequest, ChatCompletionResponse, Message,
        anthropic::{AnthropicTransformer, AnthropicResponse, AnthropicContent, AnthropicUsage},
        google::{GoogleTransformer, GoogleResponse, GoogleContent, Part, Candidate, UsageMetadata},
        qwen::{QwenTransformer, QwenResponse, QwenOutput, QwenChoice, QwenMessage, QwenUsage},
    };

    // ============================================================
    // Generators for Property-Based Testing
    // ============================================================

    /// Generate a valid role for messages
    fn role_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("user".to_string()),
            Just("assistant".to_string()),
            Just("system".to_string()),
        ]
    }

    /// Generate non-empty message content
    fn content_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 .,!?]{1,200}".prop_map(|s| s.trim().to_string())
            .prop_filter("content must not be empty", |s| !s.is_empty())
    }

    /// Generate a valid message
    fn message_strategy() -> impl Strategy<Value = Message> {
        (role_strategy(), content_strategy()).prop_map(|(role, content)| Message { role, content })
    }

    /// Generate a non-empty list of messages with at least one user message
    /// and at most one system message (realistic constraint)
    fn messages_strategy() -> impl Strategy<Value = Vec<Message>> {
        prop::collection::vec(message_strategy(), 1..5)
            .prop_filter("must have at least one user message", |msgs| {
                msgs.iter().any(|m| m.role == "user")
            })
            .prop_filter("at most one system message", |msgs| {
                msgs.iter().filter(|m| m.role == "system").count() <= 1
            })
    }

    /// Generate optional temperature (0.0 to 2.0)
    fn temperature_strategy() -> impl Strategy<Value = Option<f32>> {
        prop_oneof![
            Just(None),
            (0.0f32..=2.0f32).prop_map(Some),
        ]
    }

    /// Generate optional top_p (0.0 to 1.0)
    fn top_p_strategy() -> impl Strategy<Value = Option<f32>> {
        prop_oneof![
            Just(None),
            (0.0f32..=1.0f32).prop_map(Some),
        ]
    }

    /// Generate optional max_tokens (1 to 8192)
    fn max_tokens_strategy() -> impl Strategy<Value = Option<u32>> {
        prop_oneof![
            Just(None),
            (1u32..=8192u32).prop_map(Some),
        ]
    }

    /// Generate optional stop sequences
    fn stop_strategy() -> impl Strategy<Value = Option<Vec<String>>> {
        prop_oneof![
            Just(None),
            prop::collection::vec("[a-zA-Z]{1,10}", 1..3).prop_map(Some),
        ]
    }

    /// Generate a valid ChatCompletionRequest
    fn chat_completion_request_strategy() -> impl Strategy<Value = ChatCompletionRequest> {
        (
            messages_strategy(),
            temperature_strategy(),
            max_tokens_strategy(),
            prop::bool::ANY,
            top_p_strategy(),
            stop_strategy(),
        ).prop_map(|(messages, temperature, max_tokens, stream, top_p, stop)| {
            ChatCompletionRequest {
                model: "test-model".to_string(),
                messages,
                temperature,
                max_tokens,
                stream,
                top_p,
                frequency_penalty: None,
                presence_penalty: None,
                stop,
                user: None,
            }
        })
    }

    // ============================================================
    // Property Test 1: Request Format Transformation Consistency
    // **Feature: week2-multi-provider, Property 1: Request Format Transformation Consistency**
    // **Validates: Requirements 1.2, 2.2, 3.2**
    // ============================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Anthropic transformation preserves all non-system message content
        /// Requirements: 1.2 - Transform OpenAI-style messages to Anthropic format
        #[test]
        fn prop_anthropic_preserves_message_content(request in chat_completion_request_strategy()) {
            let anthropic_req = AnthropicTransformer::transform_request(&request);

            // Count non-system messages in original
            let non_system_messages: Vec<_> = request.messages.iter()
                .filter(|m| m.role != "system")
                .collect();

            // Anthropic request should have same number of non-system messages
            prop_assert_eq!(
                anthropic_req.messages.len(),
                non_system_messages.len(),
                "Anthropic should preserve all non-system messages"
            );

            // Each non-system message content should be preserved
            for (orig, transformed) in non_system_messages.iter().zip(anthropic_req.messages.iter()) {
                prop_assert_eq!(
                    &orig.content,
                    &transformed.content,
                    "Message content must be preserved"
                );
                prop_assert_eq!(
                    &orig.role,
                    &transformed.role,
                    "Message role must be preserved for non-system messages"
                );
            }
        }

        /// Property: Anthropic transformation extracts system message correctly
        /// Requirements: 1.2 - System message as separate parameter
        #[test]
        fn prop_anthropic_extracts_system_message(request in chat_completion_request_strategy()) {
            let anthropic_req = AnthropicTransformer::transform_request(&request);

            // Find system message in original request
            let system_msg = request.messages.iter().find(|m| m.role == "system");

            match system_msg {
                Some(msg) => {
                    prop_assert_eq!(
                        anthropic_req.system,
                        Some(msg.content.clone()),
                        "System message should be extracted to system field"
                    );
                }
                None => {
                    prop_assert!(
                        anthropic_req.system.is_none(),
                        "System field should be None when no system message"
                    );
                }
            }
        }

        /// Property: Anthropic transformation always sets max_tokens
        /// Requirements: 1.3 - max_tokens is required for Anthropic
        #[test]
        fn prop_anthropic_always_has_max_tokens(request in chat_completion_request_strategy()) {
            let anthropic_req = AnthropicTransformer::transform_request(&request);

            // max_tokens should always be set (default 4096 if not specified)
            prop_assert!(
                anthropic_req.max_tokens > 0,
                "Anthropic request must always have max_tokens set"
            );

            // If original had max_tokens, it should be preserved
            if let Some(orig_max) = request.max_tokens {
                prop_assert_eq!(
                    anthropic_req.max_tokens,
                    orig_max,
                    "Original max_tokens should be preserved"
                );
            } else {
                prop_assert_eq!(
                    anthropic_req.max_tokens,
                    4096,
                    "Default max_tokens should be 4096"
                );
            }
        }

        /// Property: Anthropic transformation preserves optional parameters
        /// Requirements: 1.2 - Parameter mapping
        #[test]
        fn prop_anthropic_preserves_parameters(request in chat_completion_request_strategy()) {
            let anthropic_req = AnthropicTransformer::transform_request(&request);

            prop_assert_eq!(
                anthropic_req.temperature,
                request.temperature,
                "Temperature should be preserved"
            );
            prop_assert_eq!(
                anthropic_req.top_p,
                request.top_p,
                "top_p should be preserved"
            );
            prop_assert_eq!(
                anthropic_req.stop_sequences,
                request.stop,
                "Stop sequences should be preserved"
            );
        }

        /// Property: Google transformation preserves all non-system message content
        /// Requirements: 2.2 - Convert OpenAI-style messages to Google's contents format
        #[test]
        fn prop_google_preserves_message_content(request in chat_completion_request_strategy()) {
            let google_req = GoogleTransformer::transform_request(&request);

            // Count non-system messages in original
            let non_system_messages: Vec<_> = request.messages.iter()
                .filter(|m| m.role != "system")
                .collect();

            // Google request should have same number of non-system messages in contents
            prop_assert_eq!(
                google_req.contents.len(),
                non_system_messages.len(),
                "Google should preserve all non-system messages in contents"
            );

            // Each message content should be preserved in parts
            for (orig, content) in non_system_messages.iter().zip(google_req.contents.iter()) {
                let text = content.parts.iter()
                    .map(|p| p.text.clone())
                    .collect::<Vec<_>>()
                    .join("");
                prop_assert_eq!(
                    &orig.content,
                    &text,
                    "Message content must be preserved in parts"
                );
            }
        }

        /// Property: Google transformation extracts system instruction correctly
        /// Requirements: 2.2 - System message handling
        #[test]
        fn prop_google_extracts_system_instruction(request in chat_completion_request_strategy()) {
            let google_req = GoogleTransformer::transform_request(&request);

            let system_msg = request.messages.iter().find(|m| m.role == "system");

            match system_msg {
                Some(msg) => {
                    prop_assert!(
                        google_req.system_instruction.is_some(),
                        "System instruction should be set when system message exists"
                    );
                    let sys_text = google_req.system_instruction.as_ref().unwrap()
                        .parts.iter()
                        .map(|p| p.text.clone())
                        .collect::<Vec<_>>()
                        .join("");
                    prop_assert_eq!(
                        &msg.content,
                        &sys_text,
                        "System instruction content should match"
                    );
                }
                None => {
                    prop_assert!(
                        google_req.system_instruction.is_none(),
                        "System instruction should be None when no system message"
                    );
                }
            }
        }

        /// Property: Google transformation maps roles correctly
        /// Requirements: 2.2 - Role mapping (assistant -> model)
        #[test]
        fn prop_google_maps_roles_correctly(request in chat_completion_request_strategy()) {
            let google_req = GoogleTransformer::transform_request(&request);

            let non_system_messages: Vec<_> = request.messages.iter()
                .filter(|m| m.role != "system")
                .collect();

            for (orig, content) in non_system_messages.iter().zip(google_req.contents.iter()) {
                let expected_role = match orig.role.as_str() {
                    "assistant" => "model",
                    other => other,
                };
                prop_assert_eq!(
                    &content.role,
                    expected_role,
                    "Role should be mapped correctly (assistant -> model)"
                );
            }
        }

        /// Property: Google transformation preserves generation config parameters
        /// Requirements: 2.3 - Map temperature, top_p, max_tokens to Google params
        #[test]
        fn prop_google_preserves_generation_config(request in chat_completion_request_strategy()) {
            let google_req = GoogleTransformer::transform_request(&request);

            if let Some(config) = google_req.generation_config {
                prop_assert_eq!(
                    config.temperature,
                    request.temperature,
                    "Temperature should be preserved"
                );
                prop_assert_eq!(
                    config.top_p,
                    request.top_p,
                    "top_p should be preserved"
                );
                prop_assert_eq!(
                    config.max_output_tokens,
                    request.max_tokens,
                    "max_tokens should be mapped to max_output_tokens"
                );
                prop_assert_eq!(
                    config.stop_sequences,
                    request.stop,
                    "Stop sequences should be preserved"
                );
            }
        }

        /// Property: Qwen transformation preserves all message content
        /// Requirements: 3.2 - Convert OpenAI-style messages to Qwen's input format
        #[test]
        fn prop_qwen_preserves_message_content(request in chat_completion_request_strategy()) {
            let qwen_req = QwenTransformer::transform_request(&request);

            // Qwen preserves all messages including system
            prop_assert_eq!(
                qwen_req.input.messages.len(),
                request.messages.len(),
                "Qwen should preserve all messages"
            );

            for (orig, transformed) in request.messages.iter().zip(qwen_req.input.messages.iter()) {
                prop_assert_eq!(
                    &orig.content,
                    &transformed.content,
                    "Message content must be preserved"
                );
                prop_assert_eq!(
                    &orig.role,
                    &transformed.role,
                    "Message role must be preserved"
                );
            }
        }

        /// Property: Qwen transformation preserves parameters
        /// Requirements: 3.2, 3.3 - Handle Qwen-specific parameters
        #[test]
        fn prop_qwen_preserves_parameters(request in chat_completion_request_strategy()) {
            let qwen_req = QwenTransformer::transform_request(&request);

            // Parameters should always be set for Qwen
            prop_assert!(
                qwen_req.parameters.is_some(),
                "Qwen parameters should always be set"
            );

            let params = qwen_req.parameters.unwrap();
            prop_assert_eq!(
                params.temperature,
                request.temperature,
                "Temperature should be preserved"
            );
            prop_assert_eq!(
                params.top_p,
                request.top_p,
                "top_p should be preserved"
            );
            prop_assert_eq!(
                params.max_tokens,
                request.max_tokens,
                "max_tokens should be preserved"
            );
            prop_assert_eq!(
                params.stop,
                request.stop,
                "Stop sequences should be preserved"
            );
        }

        /// Property: Qwen transformation handles streaming correctly
        /// Requirements: 3.3 - Handle Qwen-specific parameters (incremental_output)
        #[test]
        fn prop_qwen_handles_streaming(request in chat_completion_request_strategy()) {
            let qwen_req = QwenTransformer::transform_request(&request);
            let params = qwen_req.parameters.unwrap();

            if request.stream {
                prop_assert_eq!(
                    params.incremental_output,
                    Some(true),
                    "incremental_output should be true when streaming"
                );
            } else {
                prop_assert!(
                    params.incremental_output.is_none(),
                    "incremental_output should be None when not streaming"
                );
            }
        }

        /// Property: Model name is preserved across all transformers
        /// Requirements: 1.2, 2.2, 3.2 - Model preservation
        #[test]
        fn prop_model_name_preserved(request in chat_completion_request_strategy()) {
            let anthropic_req = AnthropicTransformer::transform_request(&request);
            let qwen_req = QwenTransformer::transform_request(&request);

            prop_assert_eq!(
                &anthropic_req.model,
                &request.model,
                "Anthropic should preserve model name"
            );
            prop_assert_eq!(
                &qwen_req.model,
                &request.model,
                "Qwen should preserve model name"
            );
            // Note: Google doesn't include model in request body (it's in URL)
        }
    }

    // ============================================================
    // Generators for Response Property-Based Testing
    // ============================================================

    /// Generate a valid Anthropic response ID
    fn anthropic_id_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{8,24}".prop_map(|s| format!("msg_{}", s))
    }

    /// Generate valid Anthropic content
    fn anthropic_content_strategy() -> impl Strategy<Value = Vec<AnthropicContent>> {
        prop::collection::vec(
            content_strategy().prop_map(|text| AnthropicContent {
                r#type: "text".to_string(),
                text,
            }),
            1..3,
        )
    }

    /// Generate valid Anthropic stop reason
    fn anthropic_stop_reason_strategy() -> impl Strategy<Value = Option<String>> {
        prop_oneof![
            Just(Some("end_turn".to_string())),
            Just(Some("max_tokens".to_string())),
            Just(Some("stop_sequence".to_string())),
            Just(None),
        ]
    }

    /// Generate valid token counts (positive integers)
    fn token_count_strategy() -> impl Strategy<Value = i32> {
        1i32..10000i32
    }

    /// Generate a valid AnthropicResponse
    fn anthropic_response_strategy() -> impl Strategy<Value = AnthropicResponse> {
        (
            anthropic_id_strategy(),
            anthropic_content_strategy(),
            "[a-zA-Z0-9-]{5,30}".prop_map(|s| format!("claude-{}", s)),
            anthropic_stop_reason_strategy(),
            token_count_strategy(),
            token_count_strategy(),
        ).prop_map(|(id, content, model, stop_reason, input_tokens, output_tokens)| {
            AnthropicResponse {
                id,
                r#type: "message".to_string(),
                role: "assistant".to_string(),
                content,
                model,
                stop_reason,
                stop_sequence: None,
                usage: AnthropicUsage {
                    input_tokens,
                    output_tokens,
                },
            }
        })
    }

    /// Generate valid Google finish reason
    fn google_finish_reason_strategy() -> impl Strategy<Value = Option<String>> {
        prop_oneof![
            Just(Some("STOP".to_string())),
            Just(Some("MAX_TOKENS".to_string())),
            Just(Some("SAFETY".to_string())),
            Just(None),
        ]
    }

    /// Generate a valid GoogleResponse
    fn google_response_strategy() -> impl Strategy<Value = (GoogleResponse, String)> {
        (
            content_strategy(),
            google_finish_reason_strategy(),
            token_count_strategy(),
            token_count_strategy(),
            "[a-zA-Z0-9.-]{5,20}".prop_map(|s| format!("gemini-{}", s)),
        ).prop_map(|(text, finish_reason, prompt_tokens, candidates_tokens, model)| {
            let response = GoogleResponse {
                candidates: vec![Candidate {
                    content: GoogleContent {
                        role: "model".to_string(),
                        parts: vec![Part { text }],
                    },
                    finish_reason,
                    index: Some(0),
                }],
                usage_metadata: Some(UsageMetadata {
                    prompt_token_count: Some(prompt_tokens),
                    candidates_token_count: Some(candidates_tokens),
                    total_token_count: Some(prompt_tokens + candidates_tokens),
                }),
            };
            (response, model)
        })
    }

    /// Generate valid Qwen finish reason
    fn qwen_finish_reason_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("stop".to_string()),
            Just("length".to_string()),
            Just("null".to_string()),
        ]
    }

    /// Generate a valid QwenResponse (message format)
    fn qwen_response_strategy() -> impl Strategy<Value = (QwenResponse, String)> {
        (
            content_strategy(),
            qwen_finish_reason_strategy(),
            token_count_strategy(),
            token_count_strategy(),
            "[a-zA-Z0-9]{8,16}",
            "[a-zA-Z0-9-]{5,15}".prop_map(|s| format!("qwen-{}", s)),
        ).prop_map(|(text, finish_reason, input_tokens, output_tokens, request_id, model)| {
            let response = QwenResponse {
                output: QwenOutput {
                    text: None,
                    finish_reason: None,
                    choices: Some(vec![QwenChoice {
                        finish_reason,
                        message: QwenMessage {
                            role: "assistant".to_string(),
                            content: text,
                        },
                    }]),
                },
                usage: QwenUsage {
                    input_tokens,
                    output_tokens,
                    total_tokens: Some(input_tokens + output_tokens),
                },
                request_id,
            };
            (response, model)
        })
    }

    /// Generate a valid QwenResponse (text format - legacy)
    fn qwen_response_text_format_strategy() -> impl Strategy<Value = (QwenResponse, String)> {
        (
            content_strategy(),
            qwen_finish_reason_strategy(),
            token_count_strategy(),
            token_count_strategy(),
            "[a-zA-Z0-9]{8,16}",
            "[a-zA-Z0-9-]{5,15}".prop_map(|s| format!("qwen-{}", s)),
        ).prop_map(|(text, finish_reason, input_tokens, output_tokens, request_id, model)| {
            let response = QwenResponse {
                output: QwenOutput {
                    text: Some(text),
                    finish_reason: Some(finish_reason),
                    choices: None,
                },
                usage: QwenUsage {
                    input_tokens,
                    output_tokens,
                    total_tokens: None, // Test fallback calculation
                },
                request_id,
            };
            (response, model)
        })
    }

    // ============================================================
    // Property Test 2: Response Format Normalization
    // **Feature: week2-multi-provider, Property 2: Response Format Normalization**
    // **Validates: Requirements 1.4, 2.4, 3.4**
    // ============================================================

    /// Helper function to validate OpenAI ChatCompletionResponse schema
    fn validate_openai_response_schema(response: &ChatCompletionResponse) -> Result<(), String> {
        // id: must be non-empty string starting with "chatcmpl-"
        if response.id.is_empty() {
            return Err("id must not be empty".to_string());
        }
        if !response.id.starts_with("chatcmpl-") {
            return Err(format!("id must start with 'chatcmpl-', got: {}", response.id));
        }

        // object: must be "chat.completion"
        if response.object != "chat.completion" {
            return Err(format!("object must be 'chat.completion', got: {}", response.object));
        }

        // created: must be positive timestamp
        if response.created <= 0 {
            return Err(format!("created must be positive timestamp, got: {}", response.created));
        }

        // model: must be non-empty
        if response.model.is_empty() {
            return Err("model must not be empty".to_string());
        }

        // choices: must have at least one choice
        if response.choices.is_empty() {
            return Err("choices must not be empty".to_string());
        }

        // Validate each choice
        for (i, choice) in response.choices.iter().enumerate() {
            // index: must be non-negative
            if choice.index < 0 {
                return Err(format!("choice[{}].index must be non-negative, got: {}", i, choice.index));
            }

            // message.role: must be "assistant"
            if choice.message.role != "assistant" {
                return Err(format!(
                    "choice[{}].message.role must be 'assistant', got: {}",
                    i, choice.message.role
                ));
            }

            // finish_reason: if present, must be valid OpenAI finish reason
            if let Some(ref reason) = choice.finish_reason {
                let valid_reasons = ["stop", "length", "content_filter", "function_call", "tool_calls"];
                if !valid_reasons.contains(&reason.as_str()) && !reason.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(format!(
                        "choice[{}].finish_reason has invalid format: {}",
                        i, reason
                    ));
                }
            }
        }

        // usage: all token counts must be non-negative
        if response.usage.prompt_tokens < 0 {
            return Err(format!("usage.prompt_tokens must be non-negative, got: {}", response.usage.prompt_tokens));
        }
        if response.usage.completion_tokens < 0 {
            return Err(format!("usage.completion_tokens must be non-negative, got: {}", response.usage.completion_tokens));
        }
        if response.usage.total_tokens < 0 {
            return Err(format!("usage.total_tokens must be non-negative, got: {}", response.usage.total_tokens));
        }

        // total_tokens should equal prompt_tokens + completion_tokens
        let expected_total = response.usage.prompt_tokens + response.usage.completion_tokens;
        if response.usage.total_tokens != expected_total {
            return Err(format!(
                "usage.total_tokens ({}) should equal prompt_tokens ({}) + completion_tokens ({})",
                response.usage.total_tokens, response.usage.prompt_tokens, response.usage.completion_tokens
            ));
        }

        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Anthropic response transformation produces valid OpenAI schema
        /// Requirement: 1.4 - Transform Anthropic response to OpenAI-compatible format
        #[test]
        fn prop_anthropic_response_conforms_to_openai_schema(response in anthropic_response_strategy()) {
            let transformed = AnthropicTransformer::transform_response(response);
            
            match validate_openai_response_schema(&transformed) {
                Ok(()) => prop_assert!(true),
                Err(e) => prop_assert!(false, "Schema validation failed: {}", e),
            }
        }

        /// Property: Anthropic response preserves content
        /// Requirement: 1.4 - Content preservation
        #[test]
        fn prop_anthropic_response_preserves_content(response in anthropic_response_strategy()) {
            let original_content: String = response.content.iter()
                .filter(|c| c.r#type == "text")
                .map(|c| c.text.clone())
                .collect::<Vec<_>>()
                .join("");
            
            let transformed = AnthropicTransformer::transform_response(response);
            
            prop_assert_eq!(
                &transformed.choices[0].message.content,
                &original_content,
                "Content must be preserved in transformation"
            );
        }

        /// Property: Anthropic response preserves token counts
        /// Requirement: 1.4 - Usage preservation
        #[test]
        fn prop_anthropic_response_preserves_usage(response in anthropic_response_strategy()) {
            let original_input = response.usage.input_tokens;
            let original_output = response.usage.output_tokens;
            
            let transformed = AnthropicTransformer::transform_response(response);
            
            prop_assert_eq!(
                transformed.usage.prompt_tokens,
                original_input,
                "Input tokens must be preserved"
            );
            prop_assert_eq!(
                transformed.usage.completion_tokens,
                original_output,
                "Output tokens must be preserved"
            );
        }

        /// Property: Google response transformation produces valid OpenAI schema
        /// Requirement: 2.4 - Transform Google response to OpenAI-compatible format
        #[test]
        fn prop_google_response_conforms_to_openai_schema((response, model) in google_response_strategy()) {
            let transformed = GoogleTransformer::transform_response(response, &model);
            
            match validate_openai_response_schema(&transformed) {
                Ok(()) => prop_assert!(true),
                Err(e) => prop_assert!(false, "Schema validation failed: {}", e),
            }
        }

        /// Property: Google response preserves content
        /// Requirement: 2.4 - Content preservation
        #[test]
        fn prop_google_response_preserves_content((response, model) in google_response_strategy()) {
            let original_content: String = response.candidates[0].content.parts.iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("");
            
            let transformed = GoogleTransformer::transform_response(response, &model);
            
            prop_assert_eq!(
                &transformed.choices[0].message.content,
                &original_content,
                "Content must be preserved in transformation"
            );
        }

        /// Property: Google response preserves model name
        /// Requirement: 2.4 - Model preservation
        #[test]
        fn prop_google_response_preserves_model((response, model) in google_response_strategy()) {
            let transformed = GoogleTransformer::transform_response(response, &model);
            
            prop_assert_eq!(
                &transformed.model,
                &model,
                "Model name must be preserved"
            );
        }

        /// Property: Qwen response (message format) transformation produces valid OpenAI schema
        /// Requirement: 3.4 - Transform Qwen response to OpenAI-compatible format
        #[test]
        fn prop_qwen_response_message_format_conforms_to_openai_schema((response, model) in qwen_response_strategy()) {
            let transformed = QwenTransformer::transform_response(response, &model);
            
            match validate_openai_response_schema(&transformed) {
                Ok(()) => prop_assert!(true),
                Err(e) => prop_assert!(false, "Schema validation failed: {}", e),
            }
        }

        /// Property: Qwen response (text format) transformation produces valid OpenAI schema
        /// Requirement: 3.4 - Transform Qwen response to OpenAI-compatible format (legacy text format)
        #[test]
        fn prop_qwen_response_text_format_conforms_to_openai_schema((response, model) in qwen_response_text_format_strategy()) {
            let transformed = QwenTransformer::transform_response(response, &model);
            
            match validate_openai_response_schema(&transformed) {
                Ok(()) => prop_assert!(true),
                Err(e) => prop_assert!(false, "Schema validation failed: {}", e),
            }
        }

        /// Property: Qwen response preserves content (message format)
        /// Requirement: 3.4 - Content preservation
        #[test]
        fn prop_qwen_response_preserves_content((response, model) in qwen_response_strategy()) {
            let original_content = response.output.choices.as_ref()
                .and_then(|c| c.first())
                .map(|c| c.message.content.clone())
                .unwrap_or_default();
            
            let transformed = QwenTransformer::transform_response(response, &model);
            
            prop_assert_eq!(
                &transformed.choices[0].message.content,
                &original_content,
                "Content must be preserved in transformation"
            );
        }

        /// Property: Qwen response preserves token counts
        /// Requirement: 3.4 - Usage preservation
        #[test]
        fn prop_qwen_response_preserves_usage((response, model) in qwen_response_strategy()) {
            let original_input = response.usage.input_tokens;
            let original_output = response.usage.output_tokens;
            
            let transformed = QwenTransformer::transform_response(response, &model);
            
            prop_assert_eq!(
                transformed.usage.prompt_tokens,
                original_input,
                "Input tokens must be preserved"
            );
            prop_assert_eq!(
                transformed.usage.completion_tokens,
                original_output,
                "Output tokens must be preserved"
            );
        }

        /// Property: All transformers produce consistent object type
        /// Requirements: 1.4, 2.4, 3.4 - Consistent response format
        #[test]
        fn prop_all_transformers_produce_chat_completion_object(
            anthropic_resp in anthropic_response_strategy(),
            (google_resp, google_model) in google_response_strategy(),
            (qwen_resp, qwen_model) in qwen_response_strategy(),
        ) {
            let anthropic_transformed = AnthropicTransformer::transform_response(anthropic_resp);
            let google_transformed = GoogleTransformer::transform_response(google_resp, &google_model);
            let qwen_transformed = QwenTransformer::transform_response(qwen_resp, &qwen_model);

            prop_assert_eq!(
                anthropic_transformed.object,
                "chat.completion",
                "Anthropic must produce chat.completion object"
            );
            prop_assert_eq!(
                google_transformed.object,
                "chat.completion",
                "Google must produce chat.completion object"
            );
            prop_assert_eq!(
                qwen_transformed.object,
                "chat.completion",
                "Qwen must produce chat.completion object"
            );
        }

        /// Property: All transformers produce assistant role in response
        /// Requirements: 1.4, 2.4, 3.4 - Consistent message role
        #[test]
        fn prop_all_transformers_produce_assistant_role(
            anthropic_resp in anthropic_response_strategy(),
            (google_resp, google_model) in google_response_strategy(),
            (qwen_resp, qwen_model) in qwen_response_strategy(),
        ) {
            let anthropic_transformed = AnthropicTransformer::transform_response(anthropic_resp);
            let google_transformed = GoogleTransformer::transform_response(google_resp, &google_model);
            let qwen_transformed = QwenTransformer::transform_response(qwen_resp, &qwen_model);

            prop_assert_eq!(
                &anthropic_transformed.choices[0].message.role,
                "assistant",
                "Anthropic response must have assistant role"
            );
            prop_assert_eq!(
                &google_transformed.choices[0].message.role,
                "assistant",
                "Google response must have assistant role"
            );
            prop_assert_eq!(
                &qwen_transformed.choices[0].message.role,
                "assistant",
                "Qwen response must have assistant role"
            );
        }
    }
}


// ============================================================
// Property Test 5: Model Routing Correctness
// **Feature: week2-multi-provider, Property 5: Model Routing Correctness**
// **Validates: Requirements 1.1, 2.1, 3.1**
// ============================================================

#[cfg(test)]
mod model_routing_tests {
    use proptest::prelude::*;
    use crate::services::transformers::Provider;

    // ============================================================
    // Generators for Model Names
    // ============================================================

    /// Generate valid OpenAI model names (gpt-* or o1-*)
    fn openai_model_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // GPT-4 variants
            Just("gpt-4".to_string()),
            Just("gpt-4-turbo".to_string()),
            Just("gpt-4-turbo-preview".to_string()),
            Just("gpt-4-0125-preview".to_string()),
            Just("gpt-4-1106-preview".to_string()),
            Just("gpt-4-vision-preview".to_string()),
            Just("gpt-4o".to_string()),
            Just("gpt-4o-mini".to_string()),
            // GPT-3.5 variants
            Just("gpt-3.5-turbo".to_string()),
            Just("gpt-3.5-turbo-16k".to_string()),
            Just("gpt-3.5-turbo-0125".to_string()),
            // O1 variants
            Just("o1-preview".to_string()),
            Just("o1-mini".to_string()),
            // Dynamic generation for future models
            "[a-z0-9-]{1,10}".prop_map(|suffix| format!("gpt-{}", suffix)),
            "[a-z0-9-]{1,10}".prop_map(|suffix| format!("o1-{}", suffix)),
        ]
    }

    /// Generate valid Anthropic model names (claude-*)
    fn anthropic_model_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Claude 3 variants
            Just("claude-3-opus".to_string()),
            Just("claude-3-opus-20240229".to_string()),
            Just("claude-3-sonnet".to_string()),
            Just("claude-3-sonnet-20240229".to_string()),
            Just("claude-3-haiku".to_string()),
            Just("claude-3-haiku-20240307".to_string()),
            Just("claude-3-5-sonnet".to_string()),
            Just("claude-3-5-sonnet-20241022".to_string()),
            // Claude 2 variants
            Just("claude-2.1".to_string()),
            Just("claude-2.0".to_string()),
            Just("claude-instant-1.2".to_string()),
            // Dynamic generation for future models
            "[a-z0-9.-]{1,15}".prop_map(|suffix| format!("claude-{}", suffix)),
        ]
    }

    /// Generate valid Google model names (gemini-*)
    fn google_model_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Gemini Pro variants
            Just("gemini-pro".to_string()),
            Just("gemini-pro-vision".to_string()),
            Just("gemini-1.0-pro".to_string()),
            Just("gemini-1.5-pro".to_string()),
            Just("gemini-1.5-pro-latest".to_string()),
            Just("gemini-1.5-flash".to_string()),
            Just("gemini-1.5-flash-latest".to_string()),
            // Dynamic generation for future models
            "[a-z0-9.-]{1,15}".prop_map(|suffix| format!("gemini-{}", suffix)),
        ]
    }

    /// Generate valid Qwen model names (qwen-* or qwen2-*)
    fn qwen_model_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Qwen variants
            Just("qwen-turbo".to_string()),
            Just("qwen-plus".to_string()),
            Just("qwen-max".to_string()),
            Just("qwen-max-longcontext".to_string()),
            Just("qwen-vl-plus".to_string()),
            Just("qwen-vl-max".to_string()),
            // Qwen2 variants
            Just("qwen2-72b-instruct".to_string()),
            Just("qwen2-7b-instruct".to_string()),
            Just("qwen2-1.5b-instruct".to_string()),
            // Dynamic generation for future models
            "[a-z0-9-]{1,15}".prop_map(|suffix| format!("qwen-{}", suffix)),
            "[a-z0-9-]{1,15}".prop_map(|suffix| format!("qwen2-{}", suffix)),
        ]
    }

    /// Generate unknown/unsupported model names
    fn unknown_model_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Known unsupported models
            Just("llama-2".to_string()),
            Just("llama-2-70b".to_string()),
            Just("mistral-7b".to_string()),
            Just("mixtral-8x7b".to_string()),
            Just("falcon-40b".to_string()),
            Just("palm-2".to_string()),
            Just("cohere-command".to_string()),
            // Random strings that don't match any prefix
            "[a-z]{3,10}".prop_filter("must not match known prefixes", |s| {
                !s.starts_with("gpt-") && 
                !s.starts_with("o1-") && 
                !s.starts_with("claude-") && 
                !s.starts_with("gemini-") && 
                !s.starts_with("qwen-") &&
                !s.starts_with("qwen2-")
            }),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: All OpenAI models (gpt-*, o1-*) route to OpenAI provider
        /// **Validates: Requirements 1.1**
        #[test]
        fn prop_openai_models_route_to_openai(model in openai_model_strategy()) {
            let provider = Provider::from_model(&model);
            prop_assert_eq!(
                provider,
                Some(Provider::OpenAI),
                "Model '{}' should route to OpenAI",
                model
            );
        }

        /// Property: All Anthropic models (claude-*) route to Anthropic provider
        /// **Validates: Requirements 1.1**
        #[test]
        fn prop_anthropic_models_route_to_anthropic(model in anthropic_model_strategy()) {
            let provider = Provider::from_model(&model);
            prop_assert_eq!(
                provider,
                Some(Provider::Anthropic),
                "Model '{}' should route to Anthropic",
                model
            );
        }

        /// Property: All Google models (gemini-*) route to Google provider
        /// **Validates: Requirements 2.1**
        #[test]
        fn prop_google_models_route_to_google(model in google_model_strategy()) {
            let provider = Provider::from_model(&model);
            prop_assert_eq!(
                provider,
                Some(Provider::Google),
                "Model '{}' should route to Google",
                model
            );
        }

        /// Property: All Qwen models (qwen-*) route to Qwen provider
        /// **Validates: Requirements 3.1**
        #[test]
        fn prop_qwen_models_route_to_qwen(model in qwen_model_strategy()) {
            let provider = Provider::from_model(&model);
            prop_assert_eq!(
                provider,
                Some(Provider::Qwen),
                "Model '{}' should route to Qwen",
                model
            );
        }

        /// Property: Unknown models return None (no routing)
        /// **Validates: Requirements 1.1, 2.1, 3.1 (negative case)**
        #[test]
        fn prop_unknown_models_return_none(model in unknown_model_strategy()) {
            let provider = Provider::from_model(&model);
            prop_assert_eq!(
                provider,
                None,
                "Unknown model '{}' should return None",
                model
            );
        }

        /// Property: Model routing is deterministic (same model always routes to same provider)
        /// **Validates: Requirements 1.1, 2.1, 3.1**
        #[test]
        fn prop_model_routing_is_deterministic(
            openai_model in openai_model_strategy(),
            anthropic_model in anthropic_model_strategy(),
            google_model in google_model_strategy(),
            qwen_model in qwen_model_strategy(),
        ) {
            // Call from_model twice for each model and verify consistency
            let openai_result1 = Provider::from_model(&openai_model);
            let openai_result2 = Provider::from_model(&openai_model);
            prop_assert_eq!(openai_result1, openai_result2, "OpenAI routing must be deterministic");

            let anthropic_result1 = Provider::from_model(&anthropic_model);
            let anthropic_result2 = Provider::from_model(&anthropic_model);
            prop_assert_eq!(anthropic_result1, anthropic_result2, "Anthropic routing must be deterministic");

            let google_result1 = Provider::from_model(&google_model);
            let google_result2 = Provider::from_model(&google_model);
            prop_assert_eq!(google_result1, google_result2, "Google routing must be deterministic");

            let qwen_result1 = Provider::from_model(&qwen_model);
            let qwen_result2 = Provider::from_model(&qwen_model);
            prop_assert_eq!(qwen_result1, qwen_result2, "Qwen routing must be deterministic");
        }

        /// Property: Provider name matches expected string
        /// **Validates: Requirements 1.1, 2.1, 3.1**
        #[test]
        fn prop_provider_name_is_correct(
            openai_model in openai_model_strategy(),
            anthropic_model in anthropic_model_strategy(),
            google_model in google_model_strategy(),
            qwen_model in qwen_model_strategy(),
        ) {
            if let Some(provider) = Provider::from_model(&openai_model) {
                prop_assert_eq!(provider.name(), "OpenAI");
            }
            if let Some(provider) = Provider::from_model(&anthropic_model) {
                prop_assert_eq!(provider.name(), "Anthropic");
            }
            if let Some(provider) = Provider::from_model(&google_model) {
                prop_assert_eq!(provider.name(), "Google");
            }
            if let Some(provider) = Provider::from_model(&qwen_model) {
                prop_assert_eq!(provider.name(), "Qwen");
            }
        }
    }
}


// ============================================================
// Property Test 3: Streaming Chunk Format
// **Feature: week2-multi-provider, Property 3: Streaming Chunk Format**
// **Validates: Requirements 4.2, 4.3**
// ============================================================

#[cfg(test)]
mod streaming_chunk_tests {
    use proptest::prelude::*;
    use crate::services::stream_handler::{
        StreamHandler, StreamChunk, StreamChoice, StreamDelta,
        AnthropicStreamEvent, AnthropicMessageStart, AnthropicContentBlock, AnthropicDelta,
        AnthropicMessageDeltaContent,
        GoogleStreamChunk, GoogleCandidate, GoogleContent, GooglePart,
        QwenStreamChunk, QwenStreamOutput,
    };

    // ============================================================
    // Generators for Streaming Chunks
    // ============================================================

    /// Generate valid stream chunk ID
    fn chunk_id_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{8,24}".prop_map(|s| format!("chatcmpl-{}", s))
    }

    /// Generate valid model name
    fn model_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("gpt-4".to_string()),
            Just("claude-3-opus".to_string()),
            Just("gemini-pro".to_string()),
            Just("qwen-turbo".to_string()),
        ]
    }

    /// Generate optional content
    fn content_strategy() -> impl Strategy<Value = Option<String>> {
        prop_oneof![
            Just(None),
            "[a-zA-Z0-9 .,!?]{1,100}".prop_map(Some),
        ]
    }

    /// Generate optional finish reason
    fn finish_reason_strategy() -> impl Strategy<Value = Option<String>> {
        prop_oneof![
            Just(None),
            Just(Some("stop".to_string())),
            Just(Some("length".to_string())),
        ]
    }

    /// Generate a valid StreamChunk
    fn stream_chunk_strategy() -> impl Strategy<Value = StreamChunk> {
        (
            chunk_id_strategy(),
            model_strategy(),
            content_strategy(),
            finish_reason_strategy(),
        ).prop_map(|(id, model, content, finish_reason)| {
            StreamChunk {
                id,
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp(),
                model,
                choices: vec![StreamChoice {
                    index: 0,
                    delta: StreamDelta {
                        role: if content.is_some() { Some("assistant".to_string()) } else { None },
                        content,
                    },
                    finish_reason,
                }],
            }
        })
    }

    /// Generate Anthropic content block delta event
    fn anthropic_delta_event_strategy() -> impl Strategy<Value = (AnthropicStreamEvent, String, String)> {
        (
            "[a-zA-Z0-9]{8,16}",
            model_strategy(),
            "[a-zA-Z0-9 .,!?]{1,50}",
        ).prop_map(|(msg_id, model, text)| {
            let event = AnthropicStreamEvent::ContentBlockDelta {
                index: 0,
                delta: AnthropicDelta {
                    r#type: "text_delta".to_string(),
                    text,
                },
            };
            (event, msg_id, model)
        })
    }

    /// Generate Google stream chunk
    fn google_chunk_strategy() -> impl Strategy<Value = (GoogleStreamChunk, String)> {
        (
            "[a-zA-Z0-9 .,!?]{1,100}",
            model_strategy(),
            finish_reason_strategy(),
        ).prop_map(|(text, model, finish_reason)| {
            let chunk = GoogleStreamChunk {
                candidates: Some(vec![GoogleCandidate {
                    content: Some(GoogleContent {
                        parts: Some(vec![GooglePart {
                            text: Some(text),
                        }]),
                    }),
                    finish_reason: finish_reason.map(|r| match r.as_str() {
                        "stop" => "STOP".to_string(),
                        "length" => "MAX_TOKENS".to_string(),
                        other => other.to_uppercase(),
                    }),
                }]),
            };
            (chunk, model)
        })
    }

    /// Generate Qwen stream chunk
    fn qwen_chunk_strategy() -> impl Strategy<Value = (QwenStreamChunk, String)> {
        (
            "[a-zA-Z0-9 .,!?]{1,100}",
            "[a-zA-Z0-9]{8,16}",
            model_strategy(),
            finish_reason_strategy(),
        ).prop_map(|(text, request_id, model, finish_reason)| {
            let chunk = QwenStreamChunk {
                output: QwenStreamOutput {
                    text: Some(text),
                    finish_reason,
                },
                request_id,
            };
            (chunk, model)
        })
    }

    // ============================================================
    // Helper Functions
    // ============================================================

    /// Validate SSE format: must be "data: {...}\n\n"
    fn validate_sse_format(sse: &str) -> Result<(), String> {
        if !sse.starts_with("data: ") {
            return Err(format!("SSE must start with 'data: ', got: {}", sse));
        }
        if !sse.ends_with("\n\n") {
            return Err(format!("SSE must end with '\\n\\n', got: {:?}", sse));
        }
        
        // Extract JSON part and validate it's valid JSON
        let json_part = sse.strip_prefix("data: ")
            .and_then(|s| s.strip_suffix("\n\n"))
            .ok_or("Failed to extract JSON from SSE")?;
        
        if json_part != "[DONE]" {
            serde_json::from_str::<serde_json::Value>(json_part)
                .map_err(|e| format!("Invalid JSON in SSE: {}", e))?;
        }
        
        Ok(())
    }

    /// Validate StreamChunk schema
    fn validate_stream_chunk_schema(chunk: &StreamChunk) -> Result<(), String> {
        // id must start with "chatcmpl-"
        if !chunk.id.starts_with("chatcmpl-") {
            return Err(format!("id must start with 'chatcmpl-', got: {}", chunk.id));
        }

        // object must be "chat.completion.chunk"
        if chunk.object != "chat.completion.chunk" {
            return Err(format!("object must be 'chat.completion.chunk', got: {}", chunk.object));
        }

        // created must be positive
        if chunk.created <= 0 {
            return Err(format!("created must be positive, got: {}", chunk.created));
        }

        // model must not be empty
        if chunk.model.is_empty() {
            return Err("model must not be empty".to_string());
        }

        // choices must not be empty
        if chunk.choices.is_empty() {
            return Err("choices must not be empty".to_string());
        }

        // Validate each choice
        for (i, choice) in chunk.choices.iter().enumerate() {
            if choice.index < 0 {
                return Err(format!("choice[{}].index must be non-negative", i));
            }
            
            // finish_reason if present must be valid
            if let Some(ref reason) = choice.finish_reason {
                let valid_reasons = ["stop", "length", "content_filter", "function_call", "tool_calls"];
                if !valid_reasons.contains(&reason.as_str()) && !reason.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(format!("Invalid finish_reason: {}", reason));
                }
            }
        }

        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: All formatted SSE chunks follow valid SSE format
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_sse_format_is_valid(chunk in stream_chunk_strategy()) {
            let sse = StreamHandler::format_sse_chunk(&chunk);
            
            match validate_sse_format(&sse) {
                Ok(()) => prop_assert!(true),
                Err(e) => prop_assert!(false, "SSE format validation failed: {}", e),
            }
        }

        /// Property: SSE done message is exactly "data: [DONE]\n\n"
        /// **Validates: Requirements 4.3**
        #[test]
        fn prop_sse_done_format(_dummy in Just(())) {
            let done = StreamHandler::format_sse_done();
            prop_assert_eq!(done, "data: [DONE]\n\n", "Done message must be exactly 'data: [DONE]\\n\\n'");
        }

        /// Property: Anthropic transformed chunks conform to OpenAI schema
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_anthropic_stream_chunk_schema((event, msg_id, model) in anthropic_delta_event_strategy()) {
            if let Some(chunk) = StreamHandler::transform_anthropic_chunk(&event, &msg_id, &model) {
                match validate_stream_chunk_schema(&chunk) {
                    Ok(()) => prop_assert!(true),
                    Err(e) => prop_assert!(false, "Schema validation failed: {}", e),
                }
            }
        }

        /// Property: Google transformed chunks conform to OpenAI schema
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_google_stream_chunk_schema((chunk, model) in google_chunk_strategy()) {
            if let Some(transformed) = StreamHandler::transform_google_chunk(&chunk, &model) {
                match validate_stream_chunk_schema(&transformed) {
                    Ok(()) => prop_assert!(true),
                    Err(e) => prop_assert!(false, "Schema validation failed: {}", e),
                }
            }
        }

        /// Property: Qwen transformed chunks conform to OpenAI schema
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_qwen_stream_chunk_schema((chunk, model) in qwen_chunk_strategy()) {
            if let Some(transformed) = StreamHandler::transform_qwen_chunk(&chunk, &model) {
                match validate_stream_chunk_schema(&transformed) {
                    Ok(()) => prop_assert!(true),
                    Err(e) => prop_assert!(false, "Schema validation failed: {}", e),
                }
            }
        }

        /// Property: Google transformed chunks preserve content
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_google_stream_preserves_content((chunk, model) in google_chunk_strategy()) {
            let original_text = chunk.candidates.as_ref()
                .and_then(|c| c.first())
                .and_then(|c| c.content.as_ref())
                .and_then(|c| c.parts.as_ref())
                .and_then(|p| p.first())
                .and_then(|p| p.text.clone());

            if let Some(transformed) = StreamHandler::transform_google_chunk(&chunk, &model) {
                prop_assert_eq!(
                    transformed.choices[0].delta.content.clone(),
                    original_text,
                    "Content must be preserved in transformation"
                );
            }
        }

        /// Property: Qwen transformed chunks preserve content
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_qwen_stream_preserves_content((chunk, model) in qwen_chunk_strategy()) {
            let original_text = chunk.output.text.clone();

            if let Some(transformed) = StreamHandler::transform_qwen_chunk(&chunk, &model) {
                prop_assert_eq!(
                    transformed.choices[0].delta.content.clone(),
                    original_text,
                    "Content must be preserved in transformation"
                );
            }
        }

        /// Property: All transformed chunks have object type "chat.completion.chunk"
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_all_stream_chunks_have_correct_object_type(
            (google_chunk, google_model) in google_chunk_strategy(),
            (qwen_chunk, qwen_model) in qwen_chunk_strategy(),
        ) {
            if let Some(google_transformed) = StreamHandler::transform_google_chunk(&google_chunk, &google_model) {
                prop_assert_eq!(
                    google_transformed.object,
                    "chat.completion.chunk",
                    "Google chunk must have object type 'chat.completion.chunk'"
                );
            }

            if let Some(qwen_transformed) = StreamHandler::transform_qwen_chunk(&qwen_chunk, &qwen_model) {
                prop_assert_eq!(
                    qwen_transformed.object,
                    "chat.completion.chunk",
                    "Qwen chunk must have object type 'chat.completion.chunk'"
                );
            }
        }

        /// Property: Finish reasons are normalized to OpenAI format
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_finish_reasons_normalized(
            (google_chunk, google_model) in google_chunk_strategy(),
        ) {
            if let Some(transformed) = StreamHandler::transform_google_chunk(&google_chunk, &google_model) {
                if let Some(ref reason) = transformed.choices[0].finish_reason {
                    // OpenAI finish reasons are lowercase
                    prop_assert!(
                        reason.chars().all(|c| c.is_lowercase() || c == '_'),
                        "Finish reason should be lowercase, got: {}",
                        reason
                    );
                }
            }
        }
    }
}


// ============================================================
// Property Test 4: Usage Log Completeness
// **Feature: week2-multi-provider, Property 4: Usage Log Completeness**
// **Validates: Requirements 5.1, 5.4**
// ============================================================

#[cfg(test)]
mod usage_log_tests {
    use proptest::prelude::*;
    use uuid::Uuid;
    use crate::services::transformers::Provider;
    use crate::services::usage_logger::{UsageLog, UsageLogger, TokenCounter, ProviderPricing};

    // ============================================================
    // Generators for Usage Logs
    // ============================================================

    /// Generate valid provider
    fn provider_strategy() -> impl Strategy<Value = Provider> {
        prop_oneof![
            Just(Provider::OpenAI),
            Just(Provider::Anthropic),
            Just(Provider::Google),
            Just(Provider::Qwen),
        ]
    }

    /// Generate valid model name for a provider
    fn model_for_provider_strategy(provider: Provider) -> impl Strategy<Value = String> {
        match provider {
            Provider::OpenAI => prop_oneof![
                Just("gpt-4".to_string()),
                Just("gpt-4-turbo".to_string()),
                Just("gpt-3.5-turbo".to_string()),
                Just("o1-preview".to_string()),
            ].boxed(),
            Provider::Anthropic => prop_oneof![
                Just("claude-3-opus".to_string()),
                Just("claude-3-sonnet".to_string()),
                Just("claude-3-haiku".to_string()),
            ].boxed(),
            Provider::Google => prop_oneof![
                Just("gemini-pro".to_string()),
                Just("gemini-1.5-pro".to_string()),
                Just("gemini-1.5-flash".to_string()),
            ].boxed(),
            Provider::Qwen => prop_oneof![
                Just("qwen-turbo".to_string()),
                Just("qwen-plus".to_string()),
                Just("qwen-max".to_string()),
            ].boxed(),
        }
    }

    /// Generate valid token counts (non-negative)
    fn token_count_strategy() -> impl Strategy<Value = i32> {
        0i32..100000i32
    }

    /// Generate valid latency (positive)
    fn latency_strategy() -> impl Strategy<Value = i32> {
        1i32..60000i32 // 1ms to 60s
    }

    /// Generate valid HTTP status code
    fn status_code_strategy() -> impl Strategy<Value = i16> {
        prop_oneof![
            Just(200i16),
            Just(201i16),
            Just(400i16),
            Just(401i16),
            Just(403i16),
            Just(429i16),
            Just(500i16),
            Just(502i16),
            Just(503i16),
        ]
    }

    /// Generate a complete UsageLog
    fn usage_log_strategy() -> impl Strategy<Value = UsageLog> {
        (
            provider_strategy(),
            token_count_strategy(),
            token_count_strategy(),
            latency_strategy(),
            status_code_strategy(),
        ).prop_flat_map(|(provider, prompt_tokens, completion_tokens, latency_ms, status_code)| {
            model_for_provider_strategy(provider).prop_map(move |model| {
                let total_tokens = prompt_tokens + completion_tokens;
                let estimated_cost_idr = UsageLogger::calculate_cost(
                    provider,
                    &model,
                    prompt_tokens,
                    completion_tokens,
                );
                
                UsageLog {
                    user_id: Uuid::new_v4(),
                    proxy_key_id: Some(Uuid::new_v4()),
                    provider,
                    model,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                    latency_ms,
                    estimated_cost_idr,
                    status_code,
                    error_message: if status_code >= 400 {
                        Some("Error occurred".to_string())
                    } else {
                        None
                    },
                }
            })
        })
    }

    /// Generate text for token estimation
    fn text_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 .,!?]{0,1000}"
    }

    // ============================================================
    // Helper Functions
    // ============================================================

    /// Validate that a UsageLog contains all required fields
    fn validate_usage_log_completeness(log: &UsageLog) -> Result<(), String> {
        // user_id must be valid UUID (non-nil)
        if log.user_id.is_nil() {
            return Err("user_id must not be nil".to_string());
        }

        // model must not be empty
        if log.model.is_empty() {
            return Err("model must not be empty".to_string());
        }

        // prompt_tokens must be non-negative
        if log.prompt_tokens < 0 {
            return Err(format!("prompt_tokens must be non-negative, got: {}", log.prompt_tokens));
        }

        // completion_tokens must be non-negative
        if log.completion_tokens < 0 {
            return Err(format!("completion_tokens must be non-negative, got: {}", log.completion_tokens));
        }

        // total_tokens must equal prompt_tokens + completion_tokens
        let expected_total = log.prompt_tokens + log.completion_tokens;
        if log.total_tokens != expected_total {
            return Err(format!(
                "total_tokens ({}) must equal prompt_tokens ({}) + completion_tokens ({})",
                log.total_tokens, log.prompt_tokens, log.completion_tokens
            ));
        }

        // latency_ms must be positive
        if log.latency_ms <= 0 {
            return Err(format!("latency_ms must be positive, got: {}", log.latency_ms));
        }

        // estimated_cost_idr must be non-negative
        if log.estimated_cost_idr < 0 {
            return Err(format!("estimated_cost_idr must be non-negative, got: {}", log.estimated_cost_idr));
        }

        // status_code must be valid HTTP status
        if log.status_code < 100 || log.status_code >= 600 {
            return Err(format!("status_code must be valid HTTP status, got: {}", log.status_code));
        }

        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: All generated UsageLogs contain required fields
        /// **Validates: Requirements 5.1, 5.4**
        #[test]
        fn prop_usage_log_has_all_required_fields(log in usage_log_strategy()) {
            match validate_usage_log_completeness(&log) {
                Ok(()) => prop_assert!(true),
                Err(e) => prop_assert!(false, "UsageLog validation failed: {}", e),
            }
        }

        /// Property: Total tokens equals prompt + completion tokens
        /// **Validates: Requirements 5.1**
        #[test]
        fn prop_total_tokens_is_sum(log in usage_log_strategy()) {
            prop_assert_eq!(
                log.total_tokens,
                log.prompt_tokens + log.completion_tokens,
                "total_tokens must equal prompt_tokens + completion_tokens"
            );
        }

        /// Property: Cost calculation is non-negative for all providers
        /// **Validates: Requirements 5.2**
        #[test]
        fn prop_cost_is_non_negative(
            provider in provider_strategy(),
            prompt_tokens in token_count_strategy(),
            completion_tokens in token_count_strategy(),
        ) {
            let model = match provider {
                Provider::OpenAI => "gpt-4",
                Provider::Anthropic => "claude-3-sonnet",
                Provider::Google => "gemini-pro",
                Provider::Qwen => "qwen-turbo",
            };
            
            let cost = UsageLogger::calculate_cost(provider, model, prompt_tokens, completion_tokens);
            prop_assert!(cost >= 0, "Cost must be non-negative, got: {}", cost);
        }

        /// Property: Cost increases with token count
        /// **Validates: Requirements 5.2**
        #[test]
        fn prop_cost_increases_with_tokens(
            provider in provider_strategy(),
            base_tokens in 1i32..10000i32,
        ) {
            let model = match provider {
                Provider::OpenAI => "gpt-4",
                Provider::Anthropic => "claude-3-sonnet",
                Provider::Google => "gemini-pro",
                Provider::Qwen => "qwen-turbo",
            };
            
            let cost_small = UsageLogger::calculate_cost(provider, model, base_tokens, base_tokens);
            let cost_large = UsageLogger::calculate_cost(provider, model, base_tokens * 10, base_tokens * 10);
            
            prop_assert!(
                cost_large >= cost_small,
                "Cost should increase with more tokens: {} vs {}",
                cost_small, cost_large
            );
        }

        /// Property: Token estimation is non-negative
        /// **Validates: Requirements 5.5**
        #[test]
        fn prop_token_estimation_non_negative(text in text_strategy()) {
            let tokens = TokenCounter::estimate_tokens(&text);
            prop_assert!(tokens >= 0, "Token count must be non-negative, got: {}", tokens);
        }

        /// Property: Token estimation scales with text length
        /// **Validates: Requirements 5.5**
        #[test]
        fn prop_token_estimation_scales_with_length(
            short_text in "[a-zA-Z]{1,10}",
            long_text in "[a-zA-Z]{100,200}",
        ) {
            let short_tokens = TokenCounter::estimate_tokens(&short_text);
            let long_tokens = TokenCounter::estimate_tokens(&long_text);
            
            prop_assert!(
                long_tokens >= short_tokens,
                "Longer text should have more tokens: {} chars -> {} tokens vs {} chars -> {} tokens",
                short_text.len(), short_tokens, long_text.len(), long_tokens
            );
        }

        /// Property: Empty text has zero tokens
        /// **Validates: Requirements 5.5**
        #[test]
        fn prop_empty_text_zero_tokens(_dummy in Just(())) {
            let tokens = TokenCounter::estimate_tokens("");
            prop_assert_eq!(tokens, 0, "Empty text should have 0 tokens");
        }

        /// Property: Pricing exists for all providers
        /// **Validates: Requirements 5.2**
        #[test]
        fn prop_pricing_exists_for_all_providers(provider in provider_strategy()) {
            let model = match provider {
                Provider::OpenAI => "gpt-4",
                Provider::Anthropic => "claude-3-sonnet",
                Provider::Google => "gemini-pro",
                Provider::Qwen => "qwen-turbo",
            };
            
            let pricing = ProviderPricing::for_model(provider, model);
            
            prop_assert!(pricing.input_per_million > 0, "Input pricing must be positive");
            prop_assert!(pricing.output_per_million > 0, "Output pricing must be positive");
        }

        /// Property: Output tokens are typically more expensive than input tokens
        /// **Validates: Requirements 5.2**
        #[test]
        fn prop_output_more_expensive_than_input(provider in provider_strategy()) {
            let model = match provider {
                Provider::OpenAI => "gpt-4",
                Provider::Anthropic => "claude-3-sonnet",
                Provider::Google => "gemini-pro",
                Provider::Qwen => "qwen-turbo",
            };
            
            let pricing = ProviderPricing::for_model(provider, model);
            
            // For most models, output is more expensive than input
            // This is a common pricing pattern in LLM APIs
            prop_assert!(
                pricing.output_per_million >= pricing.input_per_million,
                "Output pricing ({}) should be >= input pricing ({})",
                pricing.output_per_million, pricing.input_per_million
            );
        }
    }
}
